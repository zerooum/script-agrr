//! Example: script using its own credentials via requires_auth.
//!
//! Build and copy the binary to the parent folder:
//!   cd scripts/_examples/own_creds/rust
//!   cargo build --release
//!   cp target/release/main ..

use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, ScriptMeta, run_script,
};

struct OwnCredsExample;

impl AgrrScript for OwnCredsExample {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Own Credentials Example (Rust)".into(),
            description: "Example script that requires its own credentials (API_KEY and TOKEN)".into(),
            group: "Examples".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec!["API_KEY".into(), "TOKEN".into()],
            global_auth: false,
            args: vec![ArgSpec {
                name: "action".into(),
                prompt: "Action:".into(),
                arg_type: ArgType::Select,
                options: vec!["list".into(), "create".into(), "delete".into()],
                max_length: None,
                pattern: None,
                required: true,
                default: Some("list".into()),
            }],
        }
    }

    fn run(&self, creds: Credentials, _args: Args) -> Result<(), AuthError> {
        let api_key = creds.get("API_KEY").unwrap_or_default();
        let token = creds.get("TOKEN").unwrap_or_default();

        if api_key.is_empty() || token.is_empty() {
            return Err(AuthError);
        }

        let action = Args::get("action").unwrap_or_else(|| "list".into());

        println!("=== Own Credentials Example ===\n");
        println!("  API_KEY: {api_key:?}");
        println!("  TOKEN:   {token:?}");
        println!("  action:  {action:?}");
        println!("\nCredentials received successfully. ✓");
        Ok(())
    }
}

fn main() {
    run_script(OwnCredsExample);
}
