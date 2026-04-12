use serde::{Deserialize, Serialize};
use std::process;

/// Runtime requirement for the script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Node,
}

/// Runtime requirement block in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequirement {
    pub language: Language,
    pub min_version: String,
}

/// A named argument the script expects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,
    pub prompt: String,
    #[serde(default)]
    pub options: Vec<String>,
}

/// The manifest every agrr script must provide via `--agrr-meta`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMeta {
    pub name: String,
    pub description: String,
    pub group: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_auth: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<ArgSpec>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub global_auth: bool,
}

/// Credentials injected by the CLI as `AGRR_CRED_<KEY>` env vars.
pub struct Credentials {
    keys: Vec<String>,
}

impl Credentials {
    /// Collect credentials from environment variables.
    pub fn from_env(keys: &[String]) -> Self {
        Self {
            keys: keys.to_vec(),
        }
    }

    /// Retrieve a credential by its key name (case-insensitive lookup via uppercase).
    pub fn get(&self, key: &str) -> Option<String> {
        let env_key = format!("AGRR_CRED_{}", key.to_uppercase());
        std::env::var(env_key).ok()
    }

    /// All declared credential keys.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}

/// Named arguments injected by the CLI as `AGRR_ARG_<NAME>` env vars.
pub struct Args;

impl Args {
    /// Retrieve an arg by its name.
    pub fn get(name: &str) -> Option<String> {
        let env_key = format!("AGRR_ARG_{}", name.to_uppercase());
        std::env::var(env_key).ok()
    }
}

/// Signal authentication failure — maps to exit code 99.
///
/// Scripts MUST return this error when credentials are rejected by the remote
/// service. The CLI will delete the stored credentials and re-prompt the user.
#[derive(Debug)]
pub struct AuthError;

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "authentication failed")
    }
}

impl std::error::Error for AuthError {}

/// Trait every agrr-compatible Rust script must implement.
pub trait AgrrScript {
    /// Return the script metadata. Used to respond to `--agrr-meta`.
    fn meta(&self) -> ScriptMeta;

    /// Execute the script.
    ///
    /// # Arguments
    /// * `creds` — credentials injected via `AGRR_CRED_*` env vars
    /// * `args`  — arguments injected via `AGRR_ARG_*` env vars
    ///
    /// Return `Err(AuthError)` to signal invalid credentials (exit 99).
    fn run(&self, creds: Credentials, args: Args) -> Result<(), AuthError>;
}

/// Entry point for a Rust agrr script.
///
/// Place in `main()`:
/// ```no_run
/// use agrr_script_sdk::run_script;
/// # struct MyScript;
/// # impl agrr_script_sdk::AgrrScript for MyScript {
/// #     fn meta(&self) -> agrr_script_sdk::ScriptMeta {
/// #         agrr_script_sdk::ScriptMeta {
/// #             name: "s".into(), description: "d".into(),
/// #             group: "g".into(), version: "1.0.0".into(),
/// #             runtime: None, requires_auth: vec![], args: vec![], global_auth: false,
/// #         }
/// #     }
/// #     fn run(&self, _: agrr_script_sdk::Credentials, _: agrr_script_sdk::Args)
/// #         -> Result<(), agrr_script_sdk::AuthError> { Ok(()) }
/// # }
/// run_script(MyScript);
/// ```
pub fn run_script(script: impl AgrrScript) -> ! {
    let cli_args: Vec<String> = std::env::args().collect();

    if cli_args.iter().any(|a| a == "--agrr-meta") {
        let meta = script.meta();
        match serde_json::to_string(&meta) {
            Ok(json) => {
                println!("{json}");
                process::exit(0);
            }
            Err(e) => {
                eprintln!("agrr-sdk: failed to serialize meta: {e}");
                process::exit(1);
            }
        }
    }

    if cli_args.iter().any(|a| a == "--agrr-run") {
        let meta = script.meta();
        let mut creds = Credentials::from_env(&meta.requires_auth);
        if meta.global_auth {
            for key in &["CHAVE", "SENHA"] {
                creds.keys.push(key.to_string());
            }
        }
        let args = Args;
        match script.run(creds, args) {
            Ok(()) => process::exit(0),
            Err(AuthError) => process::exit(99),
        }
    }

    eprintln!("agrr-sdk: use --agrr-meta or --agrr-run");
    process::exit(1);
}
