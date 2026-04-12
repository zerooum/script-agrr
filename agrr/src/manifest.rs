use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported script languages with minimum version requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Node,
}

/// Runtime requirement declared by a script manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequirement {
    pub language: Language,
    /// Minimum version string, e.g. "3.11" or "18".
    pub min_version: String,
}

/// A single argument the script expects to receive via `AGRR_ARG_*` env vars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    /// Identifier used to build `AGRR_ARG_<NAME>`.
    pub name: String,
    /// Human-readable prompt shown to the user before execution.
    pub prompt: String,
    /// If present, user must choose one of these values.
    #[serde(default)]
    pub options: Vec<String>,
}

/// Full manifest returned by a script when invoked with `--agrr-meta`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptManifest {
    pub name: String,
    pub description: String,
    /// Grouping key shown as a section header in the TUI menu. Kebab-case.
    pub group: String,
    /// Semver string, e.g. "1.0.0".
    pub version: String,
    /// Required for interpreted scripts (Python/Node). Omitted for compiled binaries.
    #[serde(default)]
    pub runtime: Option<RuntimeRequirement>,
    /// Named credential keys injected as `AGRR_CRED_<KEY>` env vars.
    #[serde(default)]
    pub requires_auth: Vec<String>,
    /// Arguments collected from the user before execution.
    #[serde(default)]
    pub args: Vec<ArgSpec>,
    /// If true, the agrr global credentials (CHAVE and SENHA) are collected
    /// and injected as AGRR_CRED_CHAVE / AGRR_CRED_SENHA before execution.
    /// These are shared across all scripts that enable this flag.
    #[serde(default)]
    pub global_auth: bool,
}

/// Errors produced when parsing and validating a raw JSON manifest.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("field `name` must not be empty")]
    EmptyName,
    #[error("field `description` must not be empty")]
    EmptyDescription,
    #[error("field `group` must not be empty")]
    EmptyGroup,
    #[error("field `version` must not be empty")]
    EmptyVersion,
    #[error("runtime.min_version must not be empty")]
    EmptyMinVersion,
    #[error("arg at index {0}: `name` must not be empty")]
    EmptyArgName(usize),
    #[error("arg at index {0}: `prompt` must not be empty")]
    EmptyArgPrompt(usize),
    #[error("requires_auth key at index {0} must not be empty")]
    EmptyAuthKey(usize),
}

impl ScriptManifest {
    /// Parse and validate a manifest from a raw JSON string.
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        let manifest: ScriptManifest = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate business rules that serde cannot enforce.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.trim().is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.description.trim().is_empty() {
            return Err(ManifestError::EmptyDescription);
        }
        if self.group.trim().is_empty() {
            return Err(ManifestError::EmptyGroup);
        }
        if self.version.trim().is_empty() {
            return Err(ManifestError::EmptyVersion);
        }
        if let Some(rt) = &self.runtime {
            if rt.min_version.trim().is_empty() {
                return Err(ManifestError::EmptyMinVersion);
            }
        }
        for (i, key) in self.requires_auth.iter().enumerate() {
            if key.trim().is_empty() {
                return Err(ManifestError::EmptyAuthKey(i));
            }
        }
        for (i, arg) in self.args.iter().enumerate() {
            if arg.name.trim().is_empty() {
                return Err(ManifestError::EmptyArgName(i));
            }
            if arg.prompt.trim().is_empty() {
                return Err(ManifestError::EmptyArgPrompt(i));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_json() -> &'static str {
        r#"{"name":"Deploy","description":"Deploy app","group":"infra","version":"1.0.0"}"#
    }

    #[test]
    fn parses_minimal_valid_manifest() {
        let m = ScriptManifest::from_json(valid_json()).unwrap();
        assert_eq!(m.name, "Deploy");
        assert!(m.runtime.is_none());
        assert!(m.requires_auth.is_empty());
        assert!(m.args.is_empty());
    }

    #[test]
    fn rejects_empty_name() {
        let json = r#"{"name":"","description":"d","group":"g","version":"1.0.0"}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyName)
        ));
    }

    #[test]
    fn rejects_empty_description() {
        let json = r#"{"name":"n","description":"","group":"g","version":"1.0.0"}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyDescription)
        ));
    }

    #[test]
    fn rejects_empty_group() {
        let json = r#"{"name":"n","description":"d","group":"","version":"1.0.0"}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyGroup)
        ));
    }

    #[test]
    fn rejects_empty_version() {
        let json = r#"{"name":"n","description":"d","group":"g","version":""}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyVersion)
        ));
    }

    #[test]
    fn rejects_missing_required_field() {
        // Missing "version" entirely — serde returns Json error
        let json = r#"{"name":"n","description":"d","group":"g"}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::Json(_))
        ));
    }

    #[test]
    fn rejects_empty_auth_key() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","requires_auth":[""]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyAuthKey(0))
        ));
    }

    #[test]
    fn rejects_empty_arg_name() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"","prompt":"p"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyArgName(0))
        ));
    }

    #[test]
    fn rejects_empty_manifest() {
        assert!(ScriptManifest::from_json("{}").is_err());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(
            ScriptManifest::from_json("not json"),
            Err(ManifestError::Json(_))
        ));
    }

    #[test]
    fn parses_manifest_with_runtime_and_auth() {
        let json = r#"{
            "name": "Deploy",
            "description": "Deploy app",
            "group": "infra",
            "version": "1.0.0",
            "runtime": {"language": "python", "min_version": "3.11"},
            "requires_auth": ["AWS_USER", "AWS_PASS"],
            "args": [{"name": "env", "prompt": "Environment?", "options": ["prod", "staging"]}]
        }"#;
        let m = ScriptManifest::from_json(json).unwrap();
        let rt = m.runtime.unwrap();
        assert_eq!(rt.language, Language::Python);
        assert_eq!(rt.min_version, "3.11");
        assert_eq!(m.requires_auth, vec!["AWS_USER", "AWS_PASS"]);
        assert_eq!(m.args[0].options, vec!["prod", "staging"]);
    }
}
