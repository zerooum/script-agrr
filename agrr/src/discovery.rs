use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::time::timeout;

use crate::manifest::{ManifestError, ScriptManifest};
use crate::runtime::{self, ResolvedRuntime, RuntimeNotFound};

const META_TIMEOUT: Duration = Duration::from_secs(5);

/// A fully validated, ready-to-run script entry.
#[derive(Debug, Clone)]
pub struct ScriptEntry {
    /// Path to the script on disk.
    pub path: PathBuf,
    /// Parsed and validated manifest.
    pub manifest: ScriptManifest,
    /// Resolved runtime (executable + version). Native binaries have source == Native.
    pub resolved_runtime: ResolvedRuntime,
}

/// A script candidate that failed to load, with a human-readable reason.
#[derive(Debug, Clone)]
pub struct LoadWarning {
    /// File name (not full path, for display).
    pub filename: String,
    pub reason: String,
}

impl std::fmt::Display for LoadWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Script {} não carregado. Motivo: {}",
            self.filename, self.reason
        )
    }
}

/// A resolved script candidate ready for loading.
struct ScriptCandidate {
    /// Path to the entry-point file (`main.*` for folder scripts, the file itself for single-file scripts).
    path: PathBuf,
    /// Human-readable name used in warnings (folder name or file name).
    display_name: String,
}

/// The result of the discovery phase.
pub struct ScriptRegistry {
    pub scripts: Vec<ScriptEntry>,
    pub warnings: Vec<LoadWarning>,
}

/// Scan the `scripts/` directory, invoke `--agrr-meta` on each candidate,
/// validate the manifest, and check runtime availability.
///
/// Non-fatal errors are collected as warnings — the registry always returns,
/// even if every script fails to load.
pub async fn discover(scripts_dir: &Path) -> ScriptRegistry {
    let mut registry = ScriptRegistry {
        scripts: Vec::new(),
        warnings: Vec::new(),
    };

    let (candidates, folder_warnings) = match collect_candidates(scripts_dir) {
        Ok(c) => c,
        Err(_) => {
            // Directory doesn't exist or isn't readable
            return registry;
        }
    };

    registry.warnings.extend(folder_warnings);

    for candidate in candidates {
        match load_script(&candidate.path).await {
            Ok(entry) => registry.scripts.push(entry),
            Err(reason) => registry.warnings.push(LoadWarning {
                filename: candidate.display_name,
                reason,
            }),
        }
    }

    // Sort scripts alphabetically within each group (stable sort preserves group order)
    registry.scripts.sort_by(|a, b| {
        a.manifest
            .group
            .cmp(&b.manifest.group)
            .then(a.manifest.name.cmp(&b.manifest.name))
    });

    registry
}

/// Attempt to load a single script candidate:
/// 1. Invoke `--agrr-meta` with timeout
/// 2. Parse & validate manifest
/// 3. Resolve runtime
async fn load_script(path: &Path) -> Result<ScriptEntry, String> {
    let raw_json = fetch_meta(path).await?;

    let manifest = ScriptManifest::from_json(&raw_json).map_err(|e: ManifestError| match e {
        ManifestError::Json(_) => "manifest JSON inválido".into(),
        other => other.to_string(),
    })?;

    // Scripts interpretados (.py, .js, .mjs) devem declarar o campo `runtime`.
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if matches!(ext.as_str(), "py" | "js" | "mjs") && manifest.runtime.is_none() {
        return Err(format!(
            "campo `runtime` obrigatório para scripts .{} — declare language e min_version no manifest",
            ext
        ));
    }

    let resolved_runtime =
        runtime::resolve(manifest.runtime.as_ref()).map_err(|e: RuntimeNotFound| e.to_string())?;

    Ok(ScriptEntry {
        path: path.to_path_buf(),
        manifest,
        resolved_runtime,
    })
}

/// Invoke the script with `--agrr-meta` and return stdout, bounded by timeout.
async fn fetch_meta(path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    let task = tokio::task::spawn_blocking(move || invoke_meta(&path));

    match timeout(META_TIMEOUT, task).await {
        Ok(Ok(result)) => result,
        Ok(Err(join_err)) => Err(format!("thread panic: {join_err}")),
        Err(_) => Err("timeout na leitura do manifest (>5s)".into()),
    }
}

/// Blocking helper: spawn the script process and capture stdout.
fn invoke_meta(path: &Path) -> Result<String, String> {
    let mut cmd = build_command(path)?;
    cmd.arg("--agrr-meta");

    let output = cmd
        .output()
        .map_err(|e| format!("falha ao executar: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "saiu com código {} ao chamar --agrr-meta{}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {}", stderr.trim())
            }
        ));
    }

    String::from_utf8(output.stdout).map_err(|_| "stdout não é UTF-8 válido".into())
}

/// Build an `std::process::Command` for a script, selecting the right interpreter.
fn build_command(path: &Path) -> Result<std::process::Command, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" => {
            // We don't know the runtime yet — use system python to fetch meta first.
            // After manifest parsing we use the resolved runtime for --agrr-run.
            let python = which::which("python3")
                .or_else(|_| which::which("python"))
                .map_err(|_| "Python não encontrado no PATH para leitura de manifest".to_string())?;
            let mut cmd = std::process::Command::new(python);
            cmd.arg(path);
            Ok(cmd)
        }
        "js" | "mjs" => {
            let node = which::which("node")
                .map_err(|_| "Node não encontrado no PATH para leitura de manifest".to_string())?;
            let mut cmd = std::process::Command::new(node);
            cmd.arg(path);
            Ok(cmd)
        }
        "" => {
            // Assume compiled binary — invoke directly
            if !path.is_file() {
                return Err("arquivo não encontrado".into());
            }
            Ok(std::process::Command::new(path))
        }
        other => Err(format!("extensão '{}' não suportada", other)),
    }
}

/// Collect script candidates from the scripts directory.
///
/// Returns a tuple of:
/// - Valid candidates (single files or folders with a valid `main` entry point)
/// - Warnings for folders that were found but have no valid `main` file
fn collect_candidates(dir: &Path) -> std::io::Result<(Vec<ScriptCandidate>, Vec<LoadWarning>)> {
    let mut candidates = Vec::new();
    let mut folder_warnings = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if path.is_dir() {
            // Skip directories prefixed with `_` (support/example folders)
            if name.starts_with('_') {
                continue;
            }
            match find_main_in_folder(&path) {
                Some(main_path) => {
                    candidates.push(ScriptCandidate {
                        path: main_path,
                        display_name: name,
                    });
                }
                None => {
                    folder_warnings.push(LoadWarning {
                        filename: name,
                        reason: "arquivo main não encontrado (esperado: main.py, main.js, main.mjs ou main)".into(),
                    });
                }
            }
        } else if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let is_candidate = matches!(ext.as_str(), "py" | "js" | "mjs")
                || (ext.is_empty() && is_executable(&path));

            if is_candidate {
                candidates.push(ScriptCandidate {
                    path,
                    display_name: name,
                });
            }
        }
    }

    candidates.sort_by(|a, b| a.path.cmp(&b.path)); // deterministic ordering
    Ok((candidates, folder_warnings))
}

/// Look for a valid entry-point file inside a folder script.
///
/// Searches for `main.py`, `main.js`, `main.mjs`, and `main` (executable binary)
/// in that priority order.
fn find_main_in_folder(dir: &Path) -> Option<PathBuf> {
    for name in &["main.py", "main.js", "main.mjs"] {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // Check for executable binary named `main`
    let main_bin = dir.join("main");
    if main_bin.is_file() && is_executable(&main_bin) {
        return Some(main_bin);
    }
    None
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    // On Windows, any file without extension that exists is treated as a candidate.
    // Actual executability is determined at runtime.
    path.exists()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// Minimal valid manifest JSON for a native (compiled) script — no runtime needed.
    fn valid_manifest_json() -> &'static str {
        r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","args":[],"requires_auth":[]}"#
    }

    /// Manifest JSON for a Python script — includes required runtime field.
    fn valid_python_manifest_json() -> &'static str {
        r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","runtime":{"language":"python","min_version":"3.8"},"args":[],"requires_auth":[]}"#
    }

    /// Manifest JSON for a JS script — includes required runtime field.
    fn valid_js_manifest_json() -> &'static str {
        r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","runtime":{"language":"node","min_version":"18"},"args":[],"requires_auth":[]}"#
    }

    /// Write a Python helper that outputs the given text to stdout when called
    /// with `--agrr-meta`, and exits 0.
    fn write_python_stub(dir: &Path, name: &str, meta_output: &str) -> PathBuf {
        let path = dir.join(format!("{}.py", name));
        let body = format!(
            r#"import sys
if '--agrr-meta' in sys.argv:
    print({:?})
    sys.exit(0)
"#,
            meta_output
        );
        fs::write(&path, body).unwrap();
        path
    }

    #[tokio::test]
    async fn nonexistent_dir_returns_empty_registry() {
        let dir = Path::new("/tmp/_agrr_integration_nonexistent_xyz_999");
        let result = discover(dir).await;
        assert_eq!(result.scripts.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    #[tokio::test]
    async fn empty_dir_returns_empty_registry() {
        let dir = std::env::temp_dir().join("agrr_test_empty_dir");
        fs::create_dir_all(&dir).unwrap();
        let result = discover(&dir).await;
        assert_eq!(result.scripts.len(), 0);
        assert_eq!(result.warnings.len(), 0);
        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn valid_python_stub_loaded_without_warnings() {
        let dir = std::env::temp_dir().join("agrr_test_valid_stub");
        fs::create_dir_all(&dir).unwrap();
        write_python_stub(&dir, "my_script", valid_python_manifest_json());

        let result = discover(&dir).await;

        fs::remove_dir_all(&dir).ok();

        // If python3/python is not available, skip gracefully.
        if result.scripts.is_empty() && !result.warnings.is_empty() {
            let reason = &result.warnings[0].reason;
            if reason.contains("Python") || reason.contains("não encontrado") {
                return; // python3 not on PATH in this environment
            }
        }
        assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
        assert_eq!(result.scripts.len(), 1);
        assert_eq!(result.scripts[0].manifest.name, "test_script");
    }

    #[tokio::test]
    async fn invalid_json_stub_produces_warning() {
        let dir = std::env::temp_dir().join("agrr_test_invalid_json_stub");
        fs::create_dir_all(&dir).unwrap();
        write_python_stub(&dir, "bad_script", "this is not json at all");

        let result = discover(&dir).await;

        fs::remove_dir_all(&dir).ok();

        // Skip if python not available
        if result.scripts.is_empty() && result.warnings.is_empty() {
            return;
        }
        if !result.warnings.is_empty() && result.warnings[0].reason.contains("Python") {
            return;
        }
        assert_eq!(result.scripts.len(), 0);
        assert_eq!(result.warnings.len(), 1, "Expected warning for invalid JSON");
    }

    #[tokio::test]
    async fn unsupported_extension_is_not_a_candidate() {
        let dir = std::env::temp_dir().join("agrr_test_unsupported_ext");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("script.rb"), "# ruby script").unwrap();

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        assert_eq!(result.scripts.len(), 0);
        assert_eq!(result.warnings.len(), 0);
    }

    /// Write a JS helper that outputs `meta_output` to stdout when called with `--agrr-meta`.
    fn write_js_stub(dir: &Path, name: &str, meta_output: &str) -> PathBuf {
        let path = dir.join(format!("{}.js", name));
        // Use a template literal so double-quotes in meta_output are safe.
        let body = format!(
            "if (process.argv.includes('--agrr-meta')) {{\n  process.stdout.write(`{meta_output}` + '\\n');\n  process.exit(0);\n}}\n",
            meta_output = meta_output
        );
        fs::write(&path, body).unwrap();
        path
    }

    // ── 3.1 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn folder_with_main_py_is_loaded() {
        let dir = std::env::temp_dir().join("agrr_test_folder_main_py");
        let script_dir = dir.join("my_script");
        fs::create_dir_all(&script_dir).unwrap();
        write_python_stub(&script_dir, "main", valid_python_manifest_json());

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        // Skip gracefully when python3 is not on PATH.
        if result.scripts.is_empty() && !result.warnings.is_empty() {
            let reason = &result.warnings[0].reason;
            if reason.contains("Python") || reason.contains("não encontrado") {
                return;
            }
        }
        assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
        assert_eq!(result.scripts.len(), 1);
        assert_eq!(
            result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
            "main.py"
        );
    }

    // ── 3.2 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn folder_with_main_js_is_loaded() {
        let dir = std::env::temp_dir().join("agrr_test_folder_main_js");
        let script_dir = dir.join("js_script");
        fs::create_dir_all(&script_dir).unwrap();
        write_js_stub(&script_dir, "main", valid_js_manifest_json());

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        // Skip gracefully when node is not on PATH.
        if result.scripts.is_empty() && !result.warnings.is_empty() {
            let reason = &result.warnings[0].reason;
            if reason.contains("Node") || reason.contains("não encontrado") {
                return;
            }
        }
        assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
        assert_eq!(result.scripts.len(), 1);
        assert_eq!(
            result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
            "main.js"
        );
    }

    // ── 3.3 ──────────────────────────────────────────────────────────────────

    #[cfg(unix)]
    #[tokio::test]
    async fn folder_with_binary_main_is_loaded() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join("agrr_test_folder_binary_main");
        let script_dir = dir.join("binary_script");
        fs::create_dir_all(&script_dir).unwrap();

        let main_path = script_dir.join("main");
        let body = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--agrr-meta\" ]; then\n  printf '%s\\n' '{}'\n  exit 0\nfi\n",
            valid_manifest_json()
        );
        fs::write(&main_path, body).unwrap();
        let mut perms = fs::metadata(&main_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&main_path, perms).unwrap();

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
        assert_eq!(result.scripts.len(), 1);
        assert_eq!(
            result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
            "main"
        );
    }

    // ── 3.4 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn folder_without_main_emits_warning_with_folder_name() {
        let dir = std::env::temp_dir().join("agrr_test_folder_no_main");
        let script_dir = dir.join("orphan_folder");
        fs::create_dir_all(&script_dir).unwrap();
        // No main file — just an unrelated file.
        fs::write(script_dir.join("helper.py"), "# helper").unwrap();

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        assert_eq!(result.scripts.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].filename, "orphan_folder");
        assert!(
            result.warnings[0].reason.contains("main"),
            "Warning should mention 'main', got: {}",
            result.warnings[0].reason
        );
    }

    // ── 3.5 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn underscore_prefixed_folder_is_ignored() {
        let dir = std::env::temp_dir().join("agrr_test_underscore_folder");
        let skip_dir = dir.join("_examples");
        fs::create_dir_all(&skip_dir).unwrap();
        // Even with a valid main.py inside, the folder must be skipped entirely.
        write_python_stub(&skip_dir, "main", valid_manifest_json());

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        assert_eq!(result.scripts.len(), 0, "Underscore folder should not produce scripts");
        assert_eq!(result.warnings.len(), 0, "Underscore folder should not produce warnings");
    }

    // ── 3.6 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn nested_subfolder_not_treated_as_candidate() {
        // Structure: tmp/outer/inner/main.py
        // `outer` has no direct `main.*`, so it should warn.
        // `inner` is not a direct child of the scripts dir, so no warning for it.
        let dir = std::env::temp_dir().join("agrr_test_nested_subfolder");
        let inner = dir.join("outer").join("inner");
        fs::create_dir_all(&inner).unwrap();
        write_python_stub(&inner, "main", valid_manifest_json());

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        assert_eq!(result.scripts.len(), 0);
        let inner_warnings: Vec<_> = result.warnings.iter().filter(|w| w.filename == "inner").collect();
        assert!(inner_warnings.is_empty(), "'inner' must not produce a warning");
        let outer_warnings: Vec<_> = result.warnings.iter().filter(|w| w.filename == "outer").collect();
        assert_eq!(outer_warnings.len(), 1, "'outer' should produce exactly one warning");
    }

    // ── 3.7 ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn single_file_and_folder_coexist_without_conflict() {
        let dir = std::env::temp_dir().join("agrr_test_coexist");
        let folder = dir.join("folder_script");
        fs::create_dir_all(&folder).unwrap();
        write_python_stub(&dir, "single_script", valid_python_manifest_json());
        write_python_stub(&folder, "main", valid_python_manifest_json());

        let result = discover(&dir).await;
        fs::remove_dir_all(&dir).ok();

        // Skip gracefully when python3 is not on PATH.
        if result.scripts.is_empty() && !result.warnings.is_empty() {
            let all_python = result.warnings.iter().all(|w| {
                w.reason.contains("Python") || w.reason.contains("não encontrado")
            });
            if all_python {
                return;
            }
        }
        assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
        assert_eq!(result.scripts.len(), 2);
    }
}
