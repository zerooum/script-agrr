use std::path::PathBuf;
use std::process::Command;

use crate::manifest::{Language, RuntimeRequirement};

/// Result of resolving a runtime for a script.
#[derive(Debug, Clone)]
pub struct ResolvedRuntime {
    /// Absolute path to the interpreter executable.
    pub executable: PathBuf,
    /// Human-readable version string as reported by the runtime.
    pub version: String,
    /// How the runtime was resolved (for display to the user).
    pub source: RuntimeSource,
}

/// Where the runtime was found.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeSource {
    Pyenv,
    Nvm,
    Path,
    /// Native compiled binary — no runtime needed.
    Native,
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeSource::Pyenv => write!(f, "pyenv"),
            RuntimeSource::Nvm => write!(f, "nvm"),
            RuntimeSource::Path => write!(f, "PATH"),
            RuntimeSource::Native => write!(f, "native binary"),
        }
    }
}

/// Error indicating the runtime could not be resolved.
#[derive(Debug)]
pub struct RuntimeNotFound {
    pub language: String,
    pub min_version: String,
    pub detail: String,
}

impl std::fmt::Display for RuntimeNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} >= {} não encontrado ({})",
            capitalize(&self.language),
            self.min_version,
            self.detail
        )
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Resolve the interpreter for a `RuntimeRequirement`.
///
/// Returns `None` if no `runtime` is declared (native binary path).
pub fn resolve(req: Option<&RuntimeRequirement>) -> Result<ResolvedRuntime, RuntimeNotFound> {
    let Some(req) = req else {
        return Ok(ResolvedRuntime {
            executable: PathBuf::new(),
            version: String::new(),
            source: RuntimeSource::Native,
        });
    };

    match req.language {
        Language::Python => resolve_python(&req.min_version),
        Language::Node => resolve_node(&req.min_version),
    }
}

// ─── Python ──────────────────────────────────────────────────────────────────

fn resolve_python(min_version: &str) -> Result<ResolvedRuntime, RuntimeNotFound> {
    let min = parse_version(min_version).unwrap_or((0, 0));

    // 1. Try pyenv
    if let Some(result) = try_pyenv_python(min) {
        return Ok(result);
    }

    // 2. Fallback: PATH candidates
    let (major, minor) = min;
    let candidates: Vec<String> = {
        let mut v = vec![format!("python{}.{}", major, minor), "python3".into(), "python".into()];
        // Also try major-only: python3.12
        if major > 0 {
            v.insert(1, format!("python{}", major));
        }
        v
    };

    for candidate in &candidates {
        if let Some(result) = try_python_binary(candidate, min) {
            return Ok(ResolvedRuntime {
                source: RuntimeSource::Path,
                ..result
            });
        }
    }

    Err(RuntimeNotFound {
        language: "Python".into(),
        min_version: min_version.into(),
        detail: "pyenv e PATH verificados".into(),
    })
}

fn try_pyenv_python(min: (u32, u32)) -> Option<ResolvedRuntime> {
    // Check pyenv is available
    let pyenv = which::which("pyenv").ok()?;

    // List installed versions
    let output = Command::new(&pyenv)
        .args(["versions", "--bare", "--skip-aliases"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut best: Option<(u32, u32, u32, String)> = None; // (major, minor, patch, raw)

    for line in stdout.lines() {
        let ver_str = line.trim().trim_start_matches("* ");
        if let Some(parsed) = parse_version_triple(ver_str) {
            let (ma, mi, pa) = parsed;
            if (ma, mi) >= min {
                let is_better = best
                    .as_ref()
                    .map_or(true, |(bma, bmi, bpa, _)| (ma, mi, pa) > (*bma, *bmi, *bpa));
                if is_better {
                    best = Some((ma, mi, pa, ver_str.to_string()));
                }
            }
        }
    }

    let (_, _, _, raw) = best?;

    // Get the prefix for this version
    let prefix_output = Command::new(&pyenv)
        .args(["prefix", &raw])
        .output()
        .ok()?;

    if !prefix_output.status.success() {
        return None;
    }

    let prefix = String::from_utf8_lossy(&prefix_output.stdout)
        .trim()
        .to_string();
    let exe = PathBuf::from(&prefix).join("bin").join("python");

    Some(ResolvedRuntime {
        executable: exe,
        version: raw,
        source: RuntimeSource::Pyenv,
    })
}

fn try_python_binary(name: &str, min: (u32, u32)) -> Option<ResolvedRuntime> {
    let exe = which::which(name).ok()?;
    let version = python_version(&exe)?;
    let parsed = parse_version(&version)?;
    if parsed >= min {
        Some(ResolvedRuntime {
            executable: exe,
            version,
            source: RuntimeSource::Path,
        })
    } else {
        None
    }
}

fn python_version(exe: &PathBuf) -> Option<String> {
    let out = Command::new(exe).arg("--version").output().ok()?;
    // Python prints to stdout on 3.x, stderr on some 2.x
    let raw = if out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stderr).into_owned()
    } else {
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    // "Python 3.11.9" → "3.11.9"
    raw.split_whitespace().nth(1).map(str::to_string)
}

// ─── Node ─────────────────────────────────────────────────────────────────────

fn resolve_node(min_version: &str) -> Result<ResolvedRuntime, RuntimeNotFound> {
    let min_major = min_version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // 1. Try nvm
    if let Some(result) = try_nvm_node(min_major) {
        return Ok(result);
    }

    // 2. Fallback: node on PATH
    if let Some(exe) = which::which("node").ok() {
        if let Some(version) = node_version(&exe) {
            let actual_major = version
                .trim_start_matches('v')
                .split('.')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            if actual_major >= min_major {
                return Ok(ResolvedRuntime {
                    executable: exe,
                    version,
                    source: RuntimeSource::Path,
                });
            }
        }
    }

    Err(RuntimeNotFound {
        language: "Node".into(),
        min_version: min_version.into(),
        detail: "nvm e PATH verificados".into(),
    })
}

fn try_nvm_node(min_major: u32) -> Option<ResolvedRuntime> {
    // nvm is a shell function, not a binary; check via NVM_DIR env var
    let nvm_dir = std::env::var("NVM_DIR").ok()?;
    let nvm_dir = PathBuf::from(nvm_dir);

    if !nvm_dir.exists() {
        return None;
    }

    // List installed versions by scanning ~/.nvm/versions/node/
    let versions_dir = nvm_dir.join("versions").join("node");
    if !versions_dir.exists() {
        return None;
    }

    let mut best: Option<(u32, u32, u32, PathBuf)> = None;

    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // directory names like "v18.19.0"
            let ver = name.trim_start_matches('v');
            if let Some((ma, mi, pa)) = parse_version_triple(ver) {
                if ma >= min_major {
                    let is_better = best.as_ref().map_or(true, |(bma, bmi, bpa, _)| {
                        (ma, mi, pa) > (*bma, *bmi, *bpa)
                    });
                    if is_better {
                        let bin = entry.path().join("bin").join("node");
                        if bin.exists() {
                            best = Some((ma, mi, pa, bin));
                        }
                    }
                }
            }
        }
    }

    let (ma, mi, pa, exe) = best?;
    Some(ResolvedRuntime {
        executable: exe,
        version: format!("v{}.{}.{}", ma, mi, pa),
        source: RuntimeSource::Nvm,
    })
}

fn node_version(exe: &PathBuf) -> Option<String> {
    let out = Command::new(exe).arg("--version").output().ok()?;
    // node prints "v20.11.0"
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ─── Version parsing helpers ─────────────────────────────────────────────────

/// Parse "3.11" or "3.11.9" → (major, minor). Ignores patch.
pub fn parse_version(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.trim().splitn(3, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    Some((major, minor))
}

/// Parse "3.11.9" → (major, minor, patch).
pub fn parse_version_triple(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.split_whitespace().next()?; // strip trailing noise
    let mut parts = s.splitn(4, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| {
        // may have suffix like "9rc1" — take only leading digits
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse().ok()
    }).unwrap_or(0);
    Some((major, minor, patch))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_major_minor() {
        assert_eq!(parse_version("3.11"), Some((3, 11)));
        assert_eq!(parse_version("3.11.9"), Some((3, 11)));
        assert_eq!(parse_version("18"), Some((18, 0)));
    }

    #[test]
    fn parse_version_triple_full() {
        assert_eq!(parse_version_triple("3.11.9"), Some((3, 11, 9)));
        assert_eq!(parse_version_triple("20.11.0"), Some((20, 11, 0)));
        assert_eq!(parse_version_triple("3.9.0rc1"), Some((3, 9, 0)));
    }

    #[test]
    fn selects_highest_satisfying_version() {
        // Simulates the comparison logic used in try_pyenv_python
        let candidates = vec![(3u32, 9u32, 0u32), (3, 11, 2), (3, 12, 1), (3, 10, 5)];
        let min = (3u32, 11u32);
        let mut best: Option<(u32, u32, u32)> = None;
        for (ma, mi, pa) in candidates {
            if (ma, mi) >= min {
                let is_better = best
                    .as_ref()
                    .map_or(true, |(bma, bmi, bpa)| (ma, mi, pa) > (*bma, *bmi, *bpa));
                if is_better {
                    best = Some((ma, mi, pa));
                }
            }
        }
        assert_eq!(best, Some((3, 12, 1)));
    }

    #[test]
    fn rejects_versions_below_minimum() {
        let candidates = vec![(3u32, 9u32, 0u32), (3, 10, 5)];
        let min = (3u32, 11u32);
        let found: Vec<_> = candidates
            .into_iter()
            .filter(|(ma, mi, _)| (*ma, *mi) >= min)
            .collect();
        assert!(found.is_empty());
    }

    #[test]
    fn native_runtime_returns_native_source() {
        let result = resolve(None).unwrap();
        assert_eq!(result.source, RuntimeSource::Native);
    }

    #[test]
    fn parse_version_invalid_returns_none() {
        assert!(parse_version("not-a-version").is_none());
        assert!(parse_version("").is_none());
    }
}
