# agrr

Interactive CLI shell that aggregates team scripts with plug-and-play support. Run `agrr` to open a navigable menu of all available scripts — written in Python, JavaScript, or Rust.

## Prerequisites

| Tool | Minimum version | Purpose |
|------|----------------|---------|
| [Rust](https://rustup.rs) | 1.75 | Build the `agrr` binary |
| Python | 3.9 | Run Python scripts |
| Node.js | 18 | Run JavaScript scripts |
| pyenv *(optional)* | any | Per-script Python version pinning |
| nvm *(optional)* | any | Per-script Node version pinning |

**OS Keychain** — credentials are stored using the native OS keychain:
- **macOS**: Keychain Access
- **Windows**: Credential Manager
- **Linux**: libsecret (`sudo apt install libsecret-1-dev`) or GNOME Keyring / KWallet

## Build & Install

```bash
cargo build --release
# Binary: ./target/release/agrr
sudo cp target/release/agrr /usr/local/bin/   # or add to $PATH
```

## Installing the SDKs

**Python** (one-time, editable install):
```bash
pip install -e sdk/python
```

**JavaScript** (one-time global link):
```bash
cd sdk/js && npm link
# In each script project that depends on it:
npm link agrr-sdk
```

**Rust** — add to your script's `Cargo.toml`:
```toml
[dependencies]
agrr-script-sdk = { path = "../../agrr-script-sdk" }
```

## Adding Scripts

1. Create a script in `scripts/` using the SDK for your language (see `scripts/examples/`)
2. Implement the manifest contract — `agrr` validates manifest fields when it starts
3. Scripts with invalid manifests are shown as warnings in the TUI and skipped; they never block the menu

## Usage

```
agrr

  ↑↓ / k j   navigate menu
  Enter       execute selected script
  /           open fuzzy search
  Esc         close search / cancel
  q / Ctrl+C  quit
```

## Script Contract

Scripts must respond to two invocation modes:

- `--agrr-meta` → print JSON manifest to stdout, exit 0
- `--agrr-run`  → execute using `AGRR_CRED_*` and `AGRR_ARG_*` env vars

### Manifest JSON example

```json
{
  "name": "Deploy Produção",
  "description": "Faz deploy via AWS CLI",
  "group": "infra",
  "version": "1.0.0",
  "runtime": { "language": "python", "min_version": "3.11" },
  "requires_auth": ["AWS_ACCESS_KEY", "AWS_SECRET_KEY"],
  "args": [
    { "name": "env", "prompt": "Ambiente?", "options": ["prod", "staging"] }
  ]
}
```

### Authentication errors

Scripts *must* exit with code **99** when credentials are rejected. The CLI will:

1. Delete all stored credentials for the script from the OS Keychain
2. Show an error prompt with a "Retry" option
3. Re-collect credentials on retry

Use `AgrrAuthError` (Python/JS SDK) or return `Err(AuthError)` (Rust SDK).

## Running Tests

```bash
# Rust unit + integration tests
cargo test

# Python SDK tests
cd sdk/python && python3 -m unittest discover -s tests -v

# JavaScript SDK tests
cd sdk/js && npm test
```

## Project Structure

```
agrr/              Rust binary (host CLI)
agrr-script-sdk/   Rust SDK for script authors
sdk/python/        Python SDK (agrr-sdk)
  tests/           Python SDK unit tests
sdk/js/            JavaScript SDK (agrr-sdk)
  tests/           JS SDK unit tests
scripts/           Scripts folder (add yours here)
  examples/        Working examples in Python, JS, and Rust
openspec/          Project specs and change proposals
.github/workflows/ CI (build + test on Linux, macOS, Windows)
```

