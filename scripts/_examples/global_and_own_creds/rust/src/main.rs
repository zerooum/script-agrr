//! Example: script using both global credentials and its own credentials.
//!
//! Build and copy the binary to the parent folder:
//!   cd scripts/_examples/global_and_own_creds/rust
//!   cargo build --release
//!   cp target/release/main ..

use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, ScriptMeta, run_script,
};

struct GlobalAndOwnCredsExample;

impl AgrrScript for GlobalAndOwnCredsExample {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Global + Own Credentials Example (Rust)".into(),
            description: "Example script using global credentials (CHAVE/SENHA) plus its own (ORG_TOKEN)".into(),
            group: "Examples".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec!["ORG_TOKEN".into()],
            global_auth: true,
            args: vec![
                ArgSpec {
                    name: "resource".into(),
                    prompt: "Resource name:".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: None,
                },
                ArgSpec {
                    name: "action".into(),
                    prompt: "Action:".into(),
                    arg_type: ArgType::Select,
                    options: vec!["read".into(), "write".into(), "delete".into()],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: Some("read".into()),
                },
            ],
        }
    }

    fn run(&self, creds: Credentials, _args: Args) -> Result<(), AuthError> {
        let chave = creds.get("CHAVE").unwrap_or_default();
        let senha = creds.get("SENHA").unwrap_or_default();
        let org_token = creds.get("ORG_TOKEN").unwrap_or_default();

        if chave.is_empty() || senha.is_empty() || org_token.is_empty() {
            return Err(AuthError);
        }

        let resource = Args::get("resource").unwrap_or_default();
        let action = Args::get("action").unwrap_or_else(|| "read".into());

        println!("=== Global + Own Credentials Example ===\n");
        println!("  Global credentials:");
        println!("    CHAVE:     {chave:?}");
        println!("    SENHA:     {senha:?}");
        println!("  Own credentials:");
        println!("    ORG_TOKEN: {org_token:?}");
        println!("  Args:");
        println!("    resource: {resource:?}");
        println!("    action:   {action:?}");
        println!("\nAll credentials received successfully. ✓");
        Ok(())
    }
}

fn main() {
    run_script(GlobalAndOwnCredsExample);
}
