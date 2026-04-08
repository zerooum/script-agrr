//! Exemplo mínimo de script Rust usando agrr-script-sdk.
//!
//! Compilar: cargo build --release
//! Copiar o binário gerado para scripts/ para que o agrr o descubra.

use agrr_script_sdk::{AgrrScript, Args, ArgSpec, AuthError, Credentials, ScriptMeta, run_script};

struct HelloWorld;

impl AgrrScript for HelloWorld {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Hello World (Rust)".into(),
            description: "Exibe uma saudação personalizada (Rust nativo)".into(),
            group: "exemplos".into(),
            version: "1.0.0".into(),
            runtime: None, // binários nativos não precisam de runtime
            requires_auth: vec!["GREETING_TOKEN".into()],
            args: vec![
                ArgSpec {
                    name: "name".into(),
                    prompt: "Qual é o seu nome?".into(),
                    options: vec![],
                },
                ArgSpec {
                    name: "language".into(),
                    prompt: "Idioma da saudação?".into(),
                    options: vec!["pt".into(), "en".into(), "es".into()],
                },
            ],
        }
    }

    fn run(&self, creds: Credentials, _args: Args) -> Result<(), AuthError> {
        let token = creds.get("GREETING_TOKEN").unwrap_or_default();
        if token != "valid-token" {
            // Simula rejeição de credencial — o CLI pedirá nova senha.
            return Err(AuthError);
        }

        let name = Args::get("name").unwrap_or_else(|| "Mundo".into());
        let language = Args::get("language").unwrap_or_else(|| "pt".into());

        let greeting = match language.as_str() {
            "en" => "Hello",
            "es" => "Hola",
            _ => "Olá",
        };

        println!("{greeting}, {name}!");
        Ok(())
    }
}

fn main() {
    run_script(HelloWorld);
}
