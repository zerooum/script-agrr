//! Example: script with multiple subcommands, each with its own args.
//!
//! Build and copy the binary to the parent folder:
//!   cd scripts/_examples/subcommands/rust
//!   cargo build --release
//!   cp target/release/main ..

use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, Pattern, ScriptMeta,
    SubcommandSpec, run_script,
};

struct SubcommandsExample;

impl AgrrScript for SubcommandsExample {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Subcommands Example (Rust)".into(),
            description: "Demonstrates scripts with multiple subcommands, each with its own args".into(),
            group: "Examples".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec![],
            global_auth: false,
            args: vec![],
            subcommands: vec![
                SubcommandSpec {
                    name: "deploy".into(),
                    description: Some("Deploy the application to an environment".into()),
                    args: vec![
                        ArgSpec {
                            name: "environment".into(),
                            prompt: "Target environment:".into(),
                            arg_type: ArgType::Select,
                            options: vec!["dev".into(), "staging".into(), "prod".into()],
                            max_length: None,
                            pattern: None,
                            required: true,
                            default: Some("dev".into()),
                        },
                        ArgSpec {
                            name: "version".into(),
                            prompt: "Version to deploy (e.g. 1.2.3):".into(),
                            arg_type: ArgType::Text,
                            options: vec![],
                            max_length: Some(20),
                            pattern: Some(Pattern::Alphanumeric),
                            required: true,
                            default: None,
                        },
                    ],
                },
                SubcommandSpec {
                    name: "rollback".into(),
                    description: Some("Roll back to the previous deployment".into()),
                    args: vec![
                        ArgSpec {
                            name: "environment".into(),
                            prompt: "Target environment:".into(),
                            arg_type: ArgType::Select,
                            options: vec!["dev".into(), "staging".into(), "prod".into()],
                            max_length: None,
                            pattern: None,
                            required: true,
                            default: None,
                        },
                        ArgSpec {
                            name: "confirm".into(),
                            prompt: "Confirm rollback:".into(),
                            arg_type: ArgType::Select,
                            options: vec!["yes".into(), "no".into()],
                            max_length: None,
                            pattern: None,
                            required: true,
                            default: None,
                        },
                    ],
                },
                SubcommandSpec {
                    name: "status".into(),
                    description: Some("Show deployment status".into()),
                    args: vec![],
                },
            ],
        }
    }

    fn run_subcommand(
        &self,
        subcommand: &str,
        _creds: Credentials,
        _args: Args,
    ) -> Result<(), AuthError> {
        match subcommand {
            "deploy" => {
                let environment = Args::get("environment").unwrap_or_default();
                let version = Args::get("version").unwrap_or_default();
                println!("Deploying version '{version}' to '{environment}'...");
                println!("  environment : {environment}");
                println!("  version     : {version}");
                println!("Deploy complete (example — no real action performed).");
            }
            "rollback" => {
                let environment = Args::get("environment").unwrap_or_default();
                let confirm = Args::get("confirm").unwrap_or_default();
                if confirm != "yes" {
                    println!("Rollback cancelled.");
                } else {
                    println!("Rolling back '{environment}'...");
                    println!("Rollback complete (example — no real action performed).");
                }
            }
            "status" => {
                println!("Deployment status:");
                println!("  dev     : running v1.2.3");
                println!("  staging : running v1.2.2");
                println!("  prod    : running v1.2.1");
            }
            other => {
                eprintln!("agrr-sdk: unknown subcommand '{other}'");
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

fn main() {
    run_script(SubcommandsExample);
}
