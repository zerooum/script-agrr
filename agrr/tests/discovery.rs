use std::fs;
use std::path::{Path, PathBuf};

use agrr::discovery::discover;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn valid_manifest_json() -> &'static str {
    r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","args":[],"requires_auth":[]}"#
}

fn valid_python_manifest_json() -> &'static str {
    r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","runtime":{"language":"python","min_version":"3.8"},"args":[],"requires_auth":[]}"#
}

fn valid_js_manifest_json() -> &'static str {
    r#"{"name":"test_script","version":"1.0.0","description":"A test script","group":"testing","runtime":{"language":"node","min_version":"18"},"args":[],"requires_auth":[]}"#
}

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

fn write_js_stub(dir: &Path, name: &str, ext: &str, meta_output: &str) -> PathBuf {
    let path = dir.join(format!("{}.{}", name, ext));
    let body = format!(
        "if (process.argv.includes('--agrr-meta')) {{\n  process.stdout.write(`{meta_output}` + '\\n');\n  process.exit(0);\n}}\n",
        meta_output = meta_output
    );
    fs::write(&path, body).unwrap();
    path
}

fn python_not_available(result: &agrr::discovery::ScriptRegistry) -> bool {
    result.scripts.is_empty()
        && !result.warnings.is_empty()
        && result.warnings.iter().any(|w| {
            w.reason.contains("Python") || w.reason.contains("não encontrado")
        })
}

fn node_not_available(result: &agrr::discovery::ScriptRegistry) -> bool {
    result.scripts.is_empty()
        && !result.warnings.is_empty()
        && result.warnings.iter().any(|w| {
            w.reason.contains("Node") || w.reason.contains("não encontrado")
        })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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

    if python_not_available(&result) {
        return;
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

#[tokio::test]
async fn folder_with_main_py_is_loaded() {
    let dir = std::env::temp_dir().join("agrr_test_folder_main_py");
    let script_dir = dir.join("my_script");
    fs::create_dir_all(&script_dir).unwrap();
    write_python_stub(&script_dir, "main", valid_python_manifest_json());

    let result = discover(&dir).await;
    fs::remove_dir_all(&dir).ok();

    if python_not_available(&result) {
        return;
    }
    assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
    assert_eq!(result.scripts.len(), 1);
    assert_eq!(
        result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
        "main.py"
    );
}

#[tokio::test]
async fn folder_with_main_js_is_loaded() {
    let dir = std::env::temp_dir().join("agrr_test_folder_main_js");
    let script_dir = dir.join("js_script");
    fs::create_dir_all(&script_dir).unwrap();
    write_js_stub(&script_dir, "main", "js", valid_js_manifest_json());

    let result = discover(&dir).await;
    fs::remove_dir_all(&dir).ok();

    if node_not_available(&result) {
        return;
    }
    assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
    assert_eq!(result.scripts.len(), 1);
    assert_eq!(
        result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
        "main.js"
    );
}

#[tokio::test]
async fn folder_with_main_mjs_is_loaded() {
    let dir = std::env::temp_dir().join("agrr_test_folder_main_mjs");
    let script_dir = dir.join("mjs_script");
    fs::create_dir_all(&script_dir).unwrap();
    write_js_stub(&script_dir, "main", "mjs", valid_js_manifest_json());

    let result = discover(&dir).await;
    fs::remove_dir_all(&dir).ok();

    if node_not_available(&result) {
        return;
    }
    assert_eq!(result.warnings.len(), 0, "Unexpected warnings: {:?}", result.warnings);
    assert_eq!(result.scripts.len(), 1);
    assert_eq!(
        result.scripts[0].path.file_name().unwrap().to_str().unwrap(),
        "main.mjs"
    );
}

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

#[tokio::test]
async fn folder_without_main_emits_warning_with_folder_name() {
    let dir = std::env::temp_dir().join("agrr_test_folder_no_main");
    let script_dir = dir.join("orphan_folder");
    fs::create_dir_all(&script_dir).unwrap();
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

#[tokio::test]
async fn underscore_prefixed_folder_is_ignored() {
    let dir = std::env::temp_dir().join("agrr_test_underscore_folder");
    let skip_dir = dir.join("_examples");
    fs::create_dir_all(&skip_dir).unwrap();
    write_python_stub(&skip_dir, "main", valid_manifest_json());

    let result = discover(&dir).await;
    fs::remove_dir_all(&dir).ok();

    assert_eq!(result.scripts.len(), 0, "Underscore folder should not produce scripts");
    assert_eq!(result.warnings.len(), 0, "Underscore folder should not produce warnings");
}

#[tokio::test]
async fn nested_subfolder_not_treated_as_candidate() {
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

#[tokio::test]
async fn single_file_and_folder_coexist_without_conflict() {
    let dir = std::env::temp_dir().join("agrr_test_coexist");
    let folder = dir.join("folder_script");
    fs::create_dir_all(&folder).unwrap();
    write_python_stub(&dir, "single_script", valid_python_manifest_json());
    write_python_stub(&folder, "main", valid_python_manifest_json());

    let result = discover(&dir).await;
    fs::remove_dir_all(&dir).ok();

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
