use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::credentials;
use crate::discovery::ScriptEntry;

/// Exit status of a script execution.
#[derive(Debug, PartialEq)]
pub enum ExitStatus {
    /// Script exited with code 0.
    Success,
    /// Script exited with code 1 or any non-special code.
    Failure(i32),
    /// Script exited with code 99 — credentials were rejected.
    AuthError,
}

/// One line of script output (stdout or stderr).
#[derive(Debug)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
}

/// Collected args for a single execution.
pub type CollectedArgs = HashMap<String, String>;

/// Execute a script, streaming output line-by-line to the provided callback.
///
/// Credentials for all keys in `requires_auth` must already be present in the
/// OS Keychain or the caller must supply them in `extra_creds`.
///
/// `extra_creds` is used for credentials collected during the current session
/// that have not yet been persisted (i.e., user declined to save).
pub fn run<F>(
    entry: &ScriptEntry,
    collected_args: &CollectedArgs,
    extra_creds: &HashMap<String, String>,
    mut on_line: F,
) -> ExitStatus
where
    F: FnMut(OutputLine),
{
    let runtime_info = describe_runtime(entry);
    on_line(OutputLine::Stdout(format!(
        "\x1b[2m{}\x1b[0m",
        runtime_info
    )));

    let mut cmd = build_run_command(entry, collected_args, extra_creds);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            on_line(OutputLine::Stderr(format!("Falha ao iniciar script: {e}")));
            return ExitStatus::Failure(1);
        }
    };

    // Stream stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => on_line(OutputLine::Stdout(l)),
                Err(_) => break,
            }
        }
    }

    // Stream stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(l) => on_line(OutputLine::Stderr(l)),
                Err(_) => break,
            }
        }
    }

    match child.wait() {
        Ok(status) => match status.code() {
            Some(0) => ExitStatus::Success,
            Some(99) => ExitStatus::AuthError,
            Some(code) => ExitStatus::Failure(code),
            None => ExitStatus::Failure(-1), // terminated by signal
        },
        Err(e) => {
            on_line(OutputLine::Stderr(format!("Erro aguardando script: {e}")));
            ExitStatus::Failure(1)
        }
    }
}

// ─── Command construction ──────────────────────────────────────────────────────

fn build_run_command(
    entry: &ScriptEntry,
    collected_args: &CollectedArgs,
    extra_creds: &HashMap<String, String>,
) -> Command {
    let mut cmd = interpreter_command(entry);
    cmd.arg("--agrr-run");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Inject global credentials when the script requests them
    if entry.manifest.global_auth {
        for key in credentials::GLOBAL_KEYS {
            let env_name = format!("AGRR_CRED_{}", key.to_uppercase());
            if let Some(val) = extra_creds.get(key) {
                cmd.env(&env_name, val);
            } else if let Some(val) = credentials::get(key) {
                cmd.env(&env_name, val);
            }
        }
    }

    // Inject credentials: keychain first, then session-only overrides
    for key in &entry.manifest.requires_auth {
        let env_name = format!("AGRR_CRED_{}", key.to_uppercase());
        let keychain_val = credentials::get(key);
        if let Some(val) = extra_creds.get(key).map(|s| s.as_str()).or_else(|| keychain_val.as_deref()) {
            cmd.env(&env_name, val);
        }
    }

    // Inject collected args
    for (name, value) in collected_args {
        let env_name = format!("AGRR_ARG_{}", name.to_uppercase());
        cmd.env(env_name, value);
    }

    cmd
}

fn interpreter_command(entry: &ScriptEntry) -> Command {
    let ext = entry
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" => {
            let python = which::which("python3")
                .or_else(|_| which::which("python"))
                .expect("python3 não encontrado no PATH");
            let mut cmd = Command::new(python);
            cmd.arg(&entry.path);
            cmd
        }
        "js" | "mjs" => {
            let node =
                which::which("node").expect("node não encontrado no PATH");
            let mut cmd = Command::new(node);
            cmd.arg(&entry.path);
            cmd
        }
        _ => {
            // Compiled binary — invoke directly.
            Command::new(&entry.path)
        }
    }
}

fn describe_runtime(entry: &ScriptEntry) -> String {
    let filename = entry
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = entry
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" => format!("Executando {} com Python", filename),
        "js" | "mjs" => format!("Executando {} com Node.js", filename),
        _ => format!("Executando {} (binário nativo)", filename),
    }
}

// ─── Arg injection ─────────────────────────────────────────────────────────────

/// Build the `AGRR_ARG_*` env-var map from the user's collected input.
#[allow(dead_code)]
pub fn build_arg_env(collected: &CollectedArgs) -> HashMap<String, String> {
    collected
        .iter()
        .map(|(name, value)| (format!("AGRR_ARG_{}", name.to_uppercase()), value.clone()))
        .collect()
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_env_key_format() {
        let mut args = CollectedArgs::new();
        args.insert("env".into(), "prod".into());
        args.insert("myParam".into(), "value".into());

        let env = build_arg_env(&args);
        assert_eq!(env.get("AGRR_ARG_ENV").map(String::as_str), Some("prod"));
        assert_eq!(env.get("AGRR_ARG_MYPARAM").map(String::as_str), Some("value"));
    }

    #[test]
    fn arg_env_with_underscores() {
        let mut args = CollectedArgs::new();
        args.insert("my_arg".into(), "val".into());
        let env = build_arg_env(&args);
        assert!(env.contains_key("AGRR_ARG_MY_ARG"));
    }

    #[test]
    fn exit_status_mapping() {
        // Validate the exit code → ExitStatus mapping logic (pure)
        let map = |code: i32| match code {
            0 => ExitStatus::Success,
            99 => ExitStatus::AuthError,
            n => ExitStatus::Failure(n),
        };
        assert_eq!(map(0), ExitStatus::Success);
        assert_eq!(map(99), ExitStatus::AuthError);
        assert_eq!(map(1), ExitStatus::Failure(1));
        assert_eq!(map(42), ExitStatus::Failure(42));
    }
}
