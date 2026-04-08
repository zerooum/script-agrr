# Project Guidelines

## Overview

agrr is an interactive CLI that aggregates team scripts via a subprocess protocol. Scripts are external processes (Python, JS, Rust) that implement a two-flag contract (`--agrr-meta` / `--agrr-run`). The host CLI discovers them, renders a TUI menu, collects credentials/args, and executes.

## Architecture

| Crate / Dir | Role |
|---|---|
| `agrr/` | Host CLI binary — TUI (ratatui), discovery, credential mgmt, subprocess execution |
| `agrr-script-sdk/` | Rust SDK for script authors (`AgrrScript` trait, `run_script()` dispatcher) |
| `sdk/python/` | Python SDK (`AgrrScript` base class, `AgrrAuthError`, `main()` dispatcher) |
| `sdk/js/` | JS SDK (`createAgrrScript()` factory, `AgrrAuthError`, dispatcher) |
| `scripts/` | User scripts; `scripts/examples/` has working samples |
| `openspec/` | Spec-driven design docs and change proposals |

**Key source files in `agrr/src/`:**
- `app.rs` — 9-state FSM (Menu → Search → CollectingCred → CollectingArgs → Running → …)
- `credentials.rs` — OS keychain via `keyring`; fallback to AES-256-GCM encrypted file
- `discovery.rs` — Scans `scripts/`, invokes `--agrr-meta` (5 s timeout), validates manifest
- `executor.rs` — Builds subprocess, injects `AGRR_CRED_*`/`AGRR_ARG_*` env vars, streams output
- `manifest.rs` — `ScriptManifest` serde struct with required-field validation
- `runtime.rs` — Resolves pyenv/nvm → PATH; picks highest version ≥ `min_version`
- `ui.rs` — ratatui rendering (menu, search, prompts, scrollable output)

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
`name`, `description`, `group`, `version` — all non-empty strings. `runtime`, `requires_auth`, `args` are optional.

### Error Handling
- Script-side: `raise AgrrAuthError()` (Python), `throw new AgrrAuthError()` (JS), `return Err(AuthError)` (Rust) → exit 99
- CLI-side: invalid manifests produce warnings in the TUI sidebar; the app never blocks on bad scripts

### Credential Flow
Credentials are collected **before** args. If keychain is unavailable, falls back to encrypted file at `~/.config/agrr/credentials.enc`.

## Pitfalls

- **Manifest timeout is 5 s per script** — scripts that call remote APIs during `--agrr-meta` will be marked invalid
- **Exit 99 deletes ALL creds for the script at once**, not just the failing one
- **Arg `options` matching is case-sensitive** — `"Prod"` ≠ `"prod"`
- **Terminal raw mode** — panic handlers restore terminal state; always ensure `disable_raw_mode()` on exit paths
- SDKs live in-repo; protocol changes must update CLI + all three SDKs atomically
