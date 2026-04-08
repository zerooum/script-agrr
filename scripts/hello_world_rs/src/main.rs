//! Exemplo mínimo de script Rust usando agrr-script-sdk.
//!
//! Compilar e instalar:
//!   cd scripts/hello_world_rs
//!   cargo build
//!   cp target/debug/main .
//!
//! O binário `main` na raiz da pasta é descoberto automaticamente pelo agrr.

use agrr_script_sdk::{AgrrScript, Args, ArgSpec, AuthError, Credentials, ScriptMeta, run_script};

struct HelloWorld;

impl AgrrScript for HelloWorld {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "Hello World (Rust)".into(),
            description: "Exibe uma saudação — script de teste Rust nativo".into(),
            group: "exemplos".into(),
            version: "1.0.0".into(),
            runtime: None, // binários nativos não precisam de runtime
            requires_auth: vec![],
            args: vec![
                ArgSpec {
                    name: "nome".into(),
                    prompt: "Qual é o seu nome?".into(),
                    options: vec![],
                },
                ArgSpec {
                    name: "idioma".into(),
                    prompt: "Idioma da saudação?".into(),
                    options: vec!["pt".into(), "en".into(), "es".into()],
                },
            ],
        }
    }

    fn run(&self, _creds: Credentials, _args: Args) -> Result<(), AuthError> {
        let nome = Args::get("nome").unwrap_or_else(|| "Mundo".into());
        let idioma = Args::get("idioma").unwrap_or_else(|| "pt".into());

        let saudacao = match idioma.as_str() {
            "en" => "Hello",
            "es" => "Hola",
            _ => "Olá",
        };

        println!("{saudacao}, {nome}!");
        println!("Script Rust executado com sucesso. ✓");
        Ok(())
    }
}

fn main() {
    run_script(HelloWorld);
}
