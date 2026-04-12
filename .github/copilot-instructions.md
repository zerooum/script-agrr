# Project Guidelines

## Overview

agrr is an interactive CLI that aggregates team scripts via a subprocess protocol. Scripts are external processes (Python, JS, Rust) that implement a two-flag contract (`--agrr-meta` / `--agrr-run`). The host CLI discovers them, renders a TUI menu, collects credentials/args, and executes.

## Architecture

| Crate / Dir | Role |
|---|---|
| `agrr/` | Host CLI binary — TUI (ratatui), discovery, credential mgmt, subprocess execution |
| `sdk/rust/` | Rust SDK for script authors (`AgrrScript` trait, `run_script()` dispatcher) |
| `sdk/python/` | Python SDK (`AgrrScript` base class, `AggrAuthError`, `main()` dispatcher) |
| `sdk/js/` | JS SDK (`createAgrrScript()` factory, `AgrrAuthError`, dispatcher) |
| `scripts/` | User scripts; `scripts/_examples/` has working samples (leading `_` = ignored by discovery) |
| `openspec/` | Spec-driven design docs and change proposals |

**Key source files in `agrr/src/`:**
- `app.rs` — 12-state FSM (Menu → Search → CollectingCred → AskSaveCred → CollectingArgs → Running → ExecutionResult → AuthErrorPrompt → CredManager → CredManagerSaving → CredManagerClearConfirm → Quit)
- `credentials.rs` — OS keychain via `keyring`; fallback to AES-256-GCM encrypted file; `GLOBAL_KEYS = ["CHAVE", "SENHA"]`
- `discovery.rs` — Scans `scripts/`, supports single files and multi-file folders (`main.*`); invokes `--agrr-meta` (5 s timeout), validates manifest
- `executor.rs` — Builds subprocess, injects `AGRR_CRED_*`/`AGRR_ARG_*` env vars, streams output; injects global creds when `global_auth: true`; dispatches by file extension (`.py` → python3, `.js` → node, no ext → native binary)
- `manifest.rs` — `ScriptManifest` serde struct with required-field validation; optional `global_auth: bool`
- `ui.rs` — ratatui rendering (menu, search, prompts, scrollable output, credential manager)

## Build & Test

```bash
# Build
cargo build --workspace

# Tests — all three must pass
cargo test --workspace
cd sdk/python && python3 -m unittest discover -s tests -v
cd sdk/js && npm test
```

CI runs on ubuntu, macOS, and Windows via `.github/workflows/ci.yml`.

## Conventions

### Script Contract
- `--agrr-meta` → JSON manifest to stdout, exit 0
- `--agrr-run` → execute using env vars, exit 0 (success), 1 (error), **99 (auth failure)**
- Exit 99 deletes **all** credentials in `requires_auth` from keychain and prompts retry

### Environment Variables
```
AGRR_CRED_<UPPERCASE_KEY>   — credentials
AGRR_ARG_<UPPERCASE_NAME>   — arguments
```

### Manifest Required Fields
`name`, `description`, `group`, `version` — all non-empty strings. `requires_auth`, `args`, `global_auth` are optional. The `runtime` field is accepted but ignored by the CLI (kept for backward compatibility with script manifests).

### Global Credentials (`global_auth`)
Scripts that set `global_auth: true` in their manifest receive two additional credentials shared across all such scripts:

```
AGRR_CRED_CHAVE   — login/username (not masked in TUI)
AGRR_CRED_SENHA   — password (masked in TUI)
```

These are stored in the keychain under the keys `CHAVE` and `SENHA` (constant `GLOBAL_KEYS` in `credentials.rs`). They are collected **before** script-specific `requires_auth` credentials. To declare in each SDK:

```python
# Python
class MyScript(AgrrScript):
    global_auth = True
```
```js
// JS
createAgrrScript({ meta: { global_auth: true, ... }, run({ creds }) { creds.CHAVE; creds.SENHA } })
```
```rust
// Rust
fn meta(&self) -> ScriptMeta { ScriptMeta { global_auth: true, .. } }
```

### Credential Field Masking
The TUI masks credential inputs based on the key name. Masking logic lives in `ui.rs::is_masked_field()`:
- **Masked** (shows `*` per character): keys containing `SENHA`, `PASSWORD`, or `SECRET`
- **Visible** (plain text input): all other keys — `CHAVE`, `USUARIO`, `LOGIN`, `API_KEY`, `TOKEN`, etc.

### Credential Manager TUI (`c` key)
Press `c` from the main menu to open the credential manager. It shows:
- `◆ Globais (agrr)` at cursor position 0 — manages `CHAVE` and `SENHA`
- One entry per script that declares `requires_auth`

Keys: `↑↓` navigate, `Enter`/`s` save missing credentials, `l` clear saved credentials, `Esc` return to menu.

### Error Handling
- Script-side: `raise AgrrAuthError()` (Python), `throw new AgrrAuthError()` (JS), `return Err(AuthError)` (Rust) → exit 99
- CLI-side: invalid manifests produce warnings in the TUI sidebar; the app never blocks on bad scripts

### Credential Flow
Credentials are collected **before** args. Global credentials (`CHAVE`/`SENHA`) are collected before script-specific ones when `global_auth: true`. If keychain is unavailable (e.g. headless Linux without D-Bus/`org.freedesktop.secrets`), the `keyring` crate falls back to an AES-256-GCM encrypted file at `~/.config/agrr/credentials.enc`.

## Pitfalls

- **Manifest timeout is 5 s per script** — scripts that call remote APIs during `--agrr-meta` will be marked invalid
- **Exit 99 deletes ALL creds for the script at once**, not just the failing one
- **Arg `options` matching is case-sensitive** — `"Prod"` ≠ `"prod"`
- **Terminal raw mode** — panic handlers restore terminal state; always ensure `disable_raw_mode()` on exit paths
- SDKs live in-repo; protocol changes must update CLI + all three SDKs atomically
- **Multi-file scripts**: subdirectories of `scripts/` are candidates if they contain `main.py`, `main.js`, `main.mjs`, or `main` (binary). Search follows this priority order. Maximum depth: 1 level.
- **Ignored folders**: subdirectories whose name starts with `_` (e.g., `_examples/`) are silently ignored by discovery. Use this prefix for examples, internal utilities, or folders that should not be exposed in the TUI.
- **`ScriptEntry.path` points to `main.*`**, not the folder — `build_command` and `executor` do not need to know about the folder concept.
