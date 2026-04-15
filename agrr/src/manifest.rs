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

/// Prompt type for an argument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgType {
    /// Free-text input.
    Text,
    /// Single-choice from `options` list.
    Select,
    /// Multi-choice from `options` list; values injected as comma-separated string.
    #[serde(rename = "multiselect")]
    MultiSelect,
}

/// Character pattern constraint for text arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pattern {
    Numeric,
    Alpha,
    Alphanumeric,
}

fn default_required() -> bool {
    true
}

/// A single argument the script expects to receive via `AGRR_ARG_*` env vars.
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ArgSpec {
    /// Identifier used to build `AGRR_ARG_<NAME>`.
    pub name: String,
    /// Human-readable prompt shown to the user before execution.
    pub prompt: String,
    /// Prompt type — required field.
    #[serde(rename = "type")]
    pub arg_type: ArgType,
    /// For `select`/`multiselect`: list of allowed values (≥ 2 required).
    #[serde(default)]
    pub options: Vec<String>,
    /// Max character count for `text` inputs.
    #[serde(default)]
    pub max_length: Option<u32>,
    /// Character class constraint for `text` inputs.
    #[serde(default)]
    pub pattern: Option<Pattern>,
    /// Whether the field rejects empty input. Defaults to `true`.
    #[serde(default = "default_required")]
    pub required: bool,
    /// Default value used when the user submits blank input (implies optional).
    #[serde(default)]
    pub default: Option<String>,
}

/// A subcommand the script exposes as a named operation.
///
/// When a manifest declares `subcommands`, the TUI shows a selection step
/// before collecting args. The selected subcommand name is injected as
/// `AGRR_SUBCOMMAND` into the subprocess environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcommandSpec {
    /// Unique identifier — non-empty, no whitespace.
    pub name: String,
    /// Optional description shown in the TUI selection list.
    #[serde(default)]
    pub description: Option<String>,
    /// Arguments collected for this specific subcommand.
    #[serde(default)]
    pub args: Vec<ArgSpec>,
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
    /// MUST be empty when `subcommands` is non-empty.
    #[serde(default)]
    pub args: Vec<ArgSpec>,
    /// If true, the agrr global credentials (CHAVE and SENHA) are collected
    /// and injected as AGRR_CRED_CHAVE / AGRR_CRED_SENHA before execution.
    /// These are shared across all scripts that enable this flag.
    #[serde(default)]
    pub global_auth: bool,
    /// Named subcommands exposed by this script (≥ 2 when non-empty).
    /// Mutually exclusive with a non-empty top-level `args`.
    #[serde(default)]
    pub subcommands: Vec<SubcommandSpec>,
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
    #[error("arg at index {0}: select/multiselect requires at least 2 options")]
    InsufficientOptions(usize),
    #[error("arg at index {0}: text type must not have options")]
    TextWithOptions(usize),
    #[error("arg at index {0}: `max_length` and `pattern` are only valid for text type")]
    ConstraintOnWrongType(usize),
    #[error("arg at index {0}: default value must be one of the declared options")]
    InvalidDefaultForSelect(usize),
    #[error("arg at index {0}: multiselect options must not contain commas")]
    CommaInMultiselectOption(usize),
    #[error("subcommands requires at least 2 entries")]
    InsufficientSubcommands,
    #[error("manifest must not declare both `args` and `subcommands`")]
    ArgsAndSubcommandsMutuallyExclusive,
    #[error("subcommand at index {0}: `name` must not be empty")]
    EmptySubcommandName(usize),
    #[error("subcommand at index {0}: name must not contain whitespace")]
    WhitespaceInSubcommandName(usize),
    #[error("duplicate subcommand name: {0}")]
    DuplicateSubcommandName(String),
    #[error("subcommand '{0}': {1}")]
    SubcommandError(String, String),
}

/// Validate a single arg spec at position `i`. Extracted to avoid duplication
/// between top-level args and subcommand args.
fn validate_arg(i: usize, arg: &ArgSpec) -> Result<(), ManifestError> {
    if arg.name.trim().is_empty() {
        return Err(ManifestError::EmptyArgName(i));
    }
    if arg.prompt.trim().is_empty() {
        return Err(ManifestError::EmptyArgPrompt(i));
    }
    match arg.arg_type {
        ArgType::Text => {
            if !arg.options.is_empty() {
                return Err(ManifestError::TextWithOptions(i));
            }
        }
        ArgType::Select | ArgType::MultiSelect => {
            if arg.options.len() < 2 {
                return Err(ManifestError::InsufficientOptions(i));
            }
            if arg.max_length.is_some() || arg.pattern.is_some() {
                return Err(ManifestError::ConstraintOnWrongType(i));
            }
            if arg.arg_type == ArgType::MultiSelect {
                for opt in &arg.options {
                    if opt.contains(',') {
                        return Err(ManifestError::CommaInMultiselectOption(i));
                    }
                }
            }
            if let Some(def) = &arg.default {
                if !def.is_empty() {
                    let parts: Vec<&str> = if arg.arg_type == ArgType::MultiSelect {
                        def.split(',').collect()
                    } else {
                        vec![def.as_str()]
                    };
                    for part in parts {
                        if !arg.options.iter().any(|o| o == part) {
                            return Err(ManifestError::InvalidDefaultForSelect(i));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

impl ScriptManifest {
    /// Parse and validate a manifest from a raw JSON string.
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        let manifest: ScriptManifest = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Returns the effective arg list for a given subcommand name, or the
    /// top-level `args` when no subcommand is specified.
    pub fn effective_args(&self, subcommand: Option<&str>) -> &[ArgSpec] {
        if let Some(name) = subcommand {
            if let Some(sub) = self.subcommands.iter().find(|s| s.name == name) {
                return &sub.args;
            }
        }
        &self.args
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
            validate_arg(i, arg)?;
        }

        // Subcommand validation
        if !self.subcommands.is_empty() {
            if !self.args.is_empty() {
                return Err(ManifestError::ArgsAndSubcommandsMutuallyExclusive);
            }
            if self.subcommands.len() < 2 {
                return Err(ManifestError::InsufficientSubcommands);
            }
            let mut seen_names = std::collections::HashSet::new();
            for (i, sub) in self.subcommands.iter().enumerate() {
                if sub.name.trim().is_empty() {
                    return Err(ManifestError::EmptySubcommandName(i));
                }
                if sub.name.chars().any(|c| c.is_whitespace()) {
                    return Err(ManifestError::WhitespaceInSubcommandName(i));
                }
                if !seen_names.insert(sub.name.as_str()) {
                    return Err(ManifestError::DuplicateSubcommandName(sub.name.clone()));
                }
                for (j, arg) in sub.args.iter().enumerate() {
                    validate_arg(j, arg).map_err(|e| {
                        ManifestError::SubcommandError(sub.name.clone(), e.to_string())
                    })?;
                }
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
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"","prompt":"p","type":"text"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptyArgName(0))
        ));
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
            "args": [{"name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"]}]
        }"#;
        let m = ScriptManifest::from_json(json).unwrap();
        let rt = m.runtime.unwrap();
        assert_eq!(rt.language, Language::Python);
        assert_eq!(rt.min_version, "3.11");
        assert_eq!(m.requires_auth, vec!["AWS_USER", "AWS_PASS"]);
        assert_eq!(m.args[0].options, vec!["prod", "staging"]);
    }

    // ── New arg constraint tests ───────────────────────────────────────────────

    #[test]
    fn rejects_arg_missing_type() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::Json(_))
        ));
    }

    #[test]
    fn parses_text_arg() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"q","prompt":"p","type":"text"}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        assert_eq!(m.args[0].arg_type, ArgType::Text);
        assert!(m.args[0].required); // default true
        assert!(m.args[0].default.is_none());
    }

    #[test]
    fn parses_text_arg_with_constraints() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"code","prompt":"Code?","type":"text","max_length":6,"pattern":"numeric","required":false,"default":"000"}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        let arg = &m.args[0];
        assert_eq!(arg.arg_type, ArgType::Text);
        assert_eq!(arg.max_length, Some(6));
        assert_eq!(arg.pattern, Some(Pattern::Numeric));
        assert!(!arg.required);
        assert_eq!(arg.default.as_deref(), Some("000"));
    }

    #[test]
    fn rejects_text_arg_with_options() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"text","options":["a","b"]}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::TextWithOptions(0))
        ));
    }

    #[test]
    fn parses_select_arg() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"env","prompt":"Env?","type":"select","options":["prod","staging"]}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        assert_eq!(m.args[0].arg_type, ArgType::Select);
        assert_eq!(m.args[0].options, vec!["prod", "staging"]);
    }

    #[test]
    fn rejects_select_with_one_option() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"select","options":["only"]}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::InsufficientOptions(0))
        ));
    }

    #[test]
    fn rejects_select_with_invalid_default() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"select","options":["a","b"],"default":"c"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::InvalidDefaultForSelect(0))
        ));
    }

    #[test]
    fn parses_select_with_valid_default() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"select","options":["a","b"],"default":"b"}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        assert_eq!(m.args[0].default.as_deref(), Some("b"));
    }

    #[test]
    fn parses_multiselect_arg() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"tags","prompt":"Tags?","type":"multiselect","options":["alpha","beta","gamma"]}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        assert_eq!(m.args[0].arg_type, ArgType::MultiSelect);
        assert_eq!(m.args[0].options.len(), 3);
    }

    #[test]
    fn rejects_multiselect_with_comma_in_option() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"multiselect","options":["a,b","c"]}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::CommaInMultiselectOption(0))
        ));
    }

    #[test]
    fn rejects_constraint_on_select_type() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"select","options":["a","b"],"max_length":5}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::ConstraintOnWrongType(0))
        ));
    }

    // ── Subcommand tests ──────────────────────────────────────────────────────

    #[test]
    fn parses_valid_subcommands() {
        let json = r#"{
            "name":"n","description":"d","group":"g","version":"1.0.0",
            "subcommands":[
                {"name":"deploy","description":"Deploy it","args":[{"name":"env","prompt":"Env?","type":"select","options":["prod","staging"]}]},
                {"name":"rollback"}
            ]
        }"#;
        let m = ScriptManifest::from_json(json).unwrap();
        assert_eq!(m.subcommands.len(), 2);
        assert_eq!(m.subcommands[0].name, "deploy");
        assert_eq!(m.subcommands[0].description.as_deref(), Some("Deploy it"));
        assert_eq!(m.subcommands[0].args.len(), 1);
        assert_eq!(m.subcommands[1].name, "rollback");
        assert!(m.subcommands[1].description.is_none());
        assert!(m.subcommands[1].args.is_empty());
    }

    #[test]
    fn rejects_single_subcommand() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","subcommands":[{"name":"only"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::InsufficientSubcommands)
        ));
    }

    #[test]
    fn rejects_args_and_subcommands_together() {
        let json = r#"{
            "name":"n","description":"d","group":"g","version":"1.0.0",
            "args":[{"name":"x","prompt":"p","type":"text"}],
            "subcommands":[{"name":"a"},{"name":"b"}]
        }"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::ArgsAndSubcommandsMutuallyExclusive)
        ));
    }

    #[test]
    fn rejects_empty_subcommand_name() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","subcommands":[{"name":""},{"name":"b"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::EmptySubcommandName(0))
        ));
    }

    #[test]
    fn rejects_subcommand_name_with_whitespace() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","subcommands":[{"name":"run deploy"},{"name":"b"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::WhitespaceInSubcommandName(0))
        ));
    }

    #[test]
    fn rejects_duplicate_subcommand_names() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","subcommands":[{"name":"deploy"},{"name":"deploy"}]}"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::DuplicateSubcommandName(ref s)) if s == "deploy"
        ));
    }

    #[test]
    fn rejects_invalid_arg_in_subcommand() {
        let json = r#"{
            "name":"n","description":"d","group":"g","version":"1.0.0",
            "subcommands":[
                {"name":"deploy","args":[{"name":"x","prompt":"p","type":"select","options":["only"]}]},
                {"name":"rollback"}
            ]
        }"#;
        assert!(matches!(
            ScriptManifest::from_json(json),
            Err(ManifestError::SubcommandError(ref name, _)) if name == "deploy"
        ));
    }

    #[test]
    fn effective_args_returns_subcommand_args() {
        let json = r#"{
            "name":"n","description":"d","group":"g","version":"1.0.0",
            "subcommands":[
                {"name":"deploy","args":[{"name":"env","prompt":"Env?","type":"text"}]},
                {"name":"rollback"}
            ]
        }"#;
        let m = ScriptManifest::from_json(json).unwrap();
        let args = m.effective_args(Some("deploy"));
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "env");
    }

    #[test]
    fn effective_args_falls_back_to_top_level() {
        let json = r#"{"name":"n","description":"d","group":"g","version":"1.0.0","args":[{"name":"x","prompt":"p","type":"text"}]}"#;
        let m = ScriptManifest::from_json(json).unwrap();
        let args = m.effective_args(None);
        assert_eq!(args.len(), 1);
    }
}

