//! Example: all argument types and constraints supported by agrr.
//!
//! Build and copy the binary to the parent folder:
//!   cd scripts/_examples/arg_types/rust
//!   cargo build --release
//!   cp target/release/main ..

use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, Pattern, ScriptMeta, run_script,
};

struct ArgTypesExample;

impl AgrrScript for ArgTypesExample {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Arg Types Example (Rust)".into(),
            description: "Demonstrates text, select, and multiselect args with all supported constraints".into(),
            group: "Examples".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec![],
            global_auth: false,
            args: vec![
                // text: basic
                ArgSpec {
                    name: "name".into(),
                    prompt: "Your name:".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: None,
                },
                // text: max_length + default
                ArgSpec {
                    name: "short_code".into(),
                    prompt: "Short code (max 8 chars, default: abc):".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: Some(8),
                    pattern: None,
                    required: true,
                    default: Some("abc".into()),
                },
                // text: numeric pattern
                ArgSpec {
                    name: "age".into(),
                    prompt: "Age (numbers only, max 3 digits):".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: Some(3),
                    pattern: Some(Pattern::Numeric),
                    required: true,
                    default: None,
                },
                // text: alpha pattern, optional
                ArgSpec {
                    name: "suffix".into(),
                    prompt: "Suffix (letters only, optional):".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: None,
                    pattern: Some(Pattern::Alpha),
                    required: false,
                    default: None,
                },
                // text: alphanumeric pattern
                ArgSpec {
                    name: "code".into(),
                    prompt: "Alphanumeric code (max 8 chars):".into(),
                    arg_type: ArgType::Text,
                    options: vec![],
                    max_length: Some(8),
                    pattern: Some(Pattern::Alphanumeric),
                    required: true,
                    default: None,
                },
                // select: with default
                ArgSpec {
                    name: "environment".into(),
                    prompt: "Environment:".into(),
                    arg_type: ArgType::Select,
                    options: vec!["dev".into(), "staging".into(), "prod".into()],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: Some("dev".into()),
                },
                // select: no default
                ArgSpec {
                    name: "priority".into(),
                    prompt: "Priority:".into(),
                    arg_type: ArgType::Select,
                    options: vec!["low".into(), "medium".into(), "high".into(), "critical".into()],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: None,
                },
                // multiselect: required
                ArgSpec {
                    name: "regions".into(),
                    prompt: "Deploy regions:".into(),
                    arg_type: ArgType::MultiSelect,
                    options: vec![
                        "us-east-1".into(),
                        "us-west-2".into(),
                        "eu-west-1".into(),
                        "ap-southeast-1".into(),
                    ],
                    max_length: None,
                    pattern: None,
                    required: true,
                    default: None,
                },
                // multiselect: optional with default
                ArgSpec {
                    name: "channels".into(),
                    prompt: "Notification channels (optional):".into(),
                    arg_type: ArgType::MultiSelect,
                    options: vec![
                        "email".into(),
                        "slack".into(),
                        "pagerduty".into(),
                        "webhook".into(),
                    ],
                    max_length: None,
                    pattern: None,
                    required: false,
                    default: Some("email,slack".into()),
                },
            ],
        }
    }

    fn run(&self, _creds: Credentials, _args: Args) -> Result<(), AuthError> {
        println!("=== Arg Types Example ===\n");

        let sections: &[(&str, &[&str])] = &[
            ("text", &["name", "short_code", "age", "suffix", "code"]),
            ("select", &["environment", "priority"]),
            ("multiselect", &["regions", "channels"]),
        ];

        for (section, keys) in sections {
            println!("  [{section}]");
            for key in *keys {
                let value = Args::get(key).unwrap_or_default();
                println!("    {key}: {value:?}");
            }
            println!();
        }

        println!("All args received successfully. ✓");
        Ok(())
    }
}

fn main() {
    run_script(ArgTypesExample);
}
