use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::time::timeout;

use crate::manifest::{ManifestError, ScriptManifest};

const META_TIMEOUT: Duration = Duration::from_secs(5);

/// A fully validated, ready-to-run script entry.
#[derive(Debug, Clone)]
pub struct ScriptEntry {
    /// Path to the script on disk.
    pub path: PathBuf,
    /// Parsed and validated manifest.
    pub manifest: ScriptManifest,
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
async fn load_script(path: &Path) -> Result<ScriptEntry, String> {
    let raw_json = fetch_meta(path).await?;

    let manifest = ScriptManifest::from_json(&raw_json).map_err(|e: ManifestError| match e {
        ManifestError::Json(_) => "manifest JSON inválido".into(),
        other => other.to_string(),
    })?;

    Ok(ScriptEntry {
        path: path.to_path_buf(),
        manifest,
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
// Integration tests live in agrr/tests/discovery.rs
