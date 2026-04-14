//! Example: script using global shared credentials (CHAVE and SENHA).
//!
//! Build and copy the binary to the parent folder:
//!   cd scripts/_examples/global_creds/rust
//!   cargo build --release
//!   cp target/release/main ..

use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, ScriptMeta, run_script,
};

struct GlobalCredsExample;

impl AgrrScript for GlobalCredsExample {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Global Credentials Example (Rust)".into(),
            description: "Example script using global shared credentials (CHAVE and SENHA)".into(),
            group: "Examples".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec![],
            global_auth: true,
            args: vec![ArgSpec {
                name: "action".into(),
                prompt: "Action:".into(),
                arg_type: ArgType::Select,
                options: vec!["list".into(), "status".into(), "sync".into()],
                max_length: None,
                pattern: None,
                required: true,
                default: Some("list".into()),
            }],
        }
    }

    fn run(&self, creds: Credentials, _args: Args) -> Result<(), AuthError> {
        let chave = creds.get("CHAVE").unwrap_or_default();
        let senha = creds.get("SENHA").unwrap_or_default();

        if chave.is_empty() || senha.is_empty() {
            return Err(AuthError);
        }

        let action = Args::get("action").unwrap_or_else(|| "list".into());

        println!("=== Global Credentials Example ===\n");
        println!("  CHAVE:  {chave:?}");
        println!("  SENHA:  {senha:?}");
        println!("  action: {action:?}");
        println!("\nGlobal credentials received successfully. ✓");
        Ok(())
    }
}

fn main() {
    run_script(GlobalCredsExample);
}
