---
name: adapt-script-template
description: Use when adapting an existing script into the agrr project template. Ask for the source folder, place the adapted script under `scripts/`, and enforce the agrr subprocess contract.
---

# Adapt Script to agrr Template

## Goal

Produce a **self-contained** script under `scripts/` that implements the agrr subprocess contract. The adapted script must NOT import or reference the original source folder — all business-logic code must be copied into the new script location. The original source folder is intended to be deleted afterward.

## Workflow

### 1. Identify the source

Ask the user for the source script folder/path if it was not provided.

### 2. Inspect and discover functions

Analyze the source code and identify:

- **Runtime**: Python, JS, or Rust/binary.
- **All distinct functions/commands** the source exposes (e.g. CLI subcommands, separate handlers, distinct entry points).
- **Shared dependencies**: libraries, data files, credential schemes, and helper modules used by those functions.

### 3. Ask the user what to adapt

Present the list of discovered functions to the user and ask:

> "O script possui as seguintes funções: [list]. Deseja adaptar todas ou somente algumas?"

Let the user pick which functions to include. If only one function exists, skip this step.

### 4. Adapt user-interaction layer

In the agrr model, all user interaction is handled by the TUI — not by the script itself. Inspect the source and remove or adapt anything that interacts with the user directly:

- **If the source has a CLI framework** (clap, argparse, click, commander, etc.): strip it entirely. Arguments and credentials are now declared in the manifest and injected as env vars — there is nothing left for the CLI parser to do.
- **If the source has no CLI framework** (e.g. a library, a daemon, a plain function): skip this step. Simply ensure that any hardcoded values that should vary per execution are exposed as `AGRR_ARG_*` or `AGRR_CRED_*` env vars instead.
- **Arguments** are declared in the manifest (`args` or `subcommands[].args`) and collected by the agrr TUI before execution. The script reads them from `AGRR_ARG_<NAME>` env vars.
- **Credentials** are declared in `requires_auth` (or `global_auth: true` for shared CHAVE/SENHA) and collected/stored by the agrr TUI and credential manager. The script reads them from `AGRR_CRED_<KEY>` env vars.
- **Interactive prompts** (dialoguer, inquirer, input(), etc.) must be removed. All user input comes through the TUI prompts defined in the manifest.
- **Progress bars and spinners** should be replaced with simple `println!`/`print()` messages — the TUI streams stdout.

### 5. Copy business logic into `scripts/`

The adapted script must be **fully self-contained** under `scripts/`:

- **Single-file**: `scripts/<name>.py` or `scripts/<name>.js`
- **Multi-file**: `scripts/<name>/main.py|main.js|main.mjs|main` (binary)
  - For Rust: create a standalone crate at `scripts/<name>/` with its own `Cargo.toml`, `[workspace]` table, and `src/main.rs`.
- **All source code** the script must be copied into the script folder. Do not use path dependencies pointing outside `scripts/<name>/` (except for `sdk/rust` at `../../sdk/rust`).
- **Data files** (layouts, templates, configs, reference books) that the script needs at runtime **must** be identified, listed for the user, and copied into the script folder. Never leave them in the original source location.
    - Locate these files at runtime using the **binary's parent directory**, not the working directory. For Rust, use `std::env::current_exe()?.parent()` or the `CONVERSOR_LAYOUTS_DIR`-style env var override pattern. For Python/JS, use `os.path.dirname(os.path.abspath(__file__))` or `path.dirname(new URL(import.meta.url).pathname)`.
  - If a data file is missing at `--agrr-run` time, the script should fail with a clear error message explaining which file is missing and where it is expected.
  - For `--agrr-meta`, **never** read data files — the manifest must return instantly with no I/O.
- Avoid folder names starting with `_` (discovery ignores them).

### 6. Implement the agrr contract

Use the appropriate SDK (`sdk/python`, `sdk/js`, `sdk/rust`):

- **`--agrr-meta`** → prints valid manifest JSON to stdout and exits 0.
- **`--agrr-run`** → executes logic using env vars and exits 0 (success), 1 (error), or 99 (auth failure).

#### Manifest rules

- Required non-empty fields: `name`, `description`, `group`, `version`.
- `global_auth: true` when the script needs shared `AGRR_CRED_CHAVE` and `AGRR_CRED_SENHA`.
- Every `args` entry **must** include `type`: `text`, `select`, or `multiselect`.
- `select`/`multiselect` require `options` with ≥ 2 items.
- `multiselect` options must not contain commas.
- `text` must not declare `options`; `max_length`/`pattern` are `text`-only.
- `pattern` supports `"numeric"`, `"alpha"`, `"alphanumeric"`, or `null`.
- `required` defaults to `true`; set `required: false` for optional input.

#### Mapping multiple functions to subcommands

When multiple functions are selected (step 3), map them to `subcommands`:

- Each function becomes a `SubcommandSpec` with its own `name`, `description`, and `args`.
- Subcommands require ≥ 2 entries and must have unique non-empty names (no whitespace).
- `subcommands` and top-level `args` are mutually exclusive — use one or the other.
- The selected subcommand name arrives via `AGRR_SUBCOMMAND` env var.

When only one function is selected, use top-level `args` instead of subcommands.

### 7. Build and validate

- **Build**: `cargo build --release` (Rust), or verify the script runs (Python/JS).
- **Copy binary**: For Rust, copy `target/release/main` to `scripts/<name>/main`.
- **Test `--agrr-meta`**: run with `--agrr-meta` and pipe through `python3 -m json.tool` to verify valid JSON.
- **Test `--agrr-run`**: run with appropriate `AGRR_ARG_*`/`AGRR_CRED_*`/`AGRR_SUBCOMMAND` env vars.
- **Workspace tests**: run `cargo test --workspace` to ensure nothing is broken.
- **Exclude from workspace**: If the script is a Rust crate, add it to `workspace.exclude` in the root `Cargo.toml` and include an empty `[workspace]` table in the script's own `Cargo.toml`.

## Must Follow

- **Self-contained**: The adapted script under `scripts/` must not reference or depend on the original source folder. Copy all needed code.
- **No CLI frameworks**: Strip clap, argparse, click, commander, etc. The TUI handles all user interaction.
- **No interactive prompts**: Remove dialoguer, inquirer, input(), etc. Arguments come from env vars.
- **SDK required**: Use project SDKs (`sdk/python`, `sdk/js`, `sdk/rust`) — do not implement the protocol manually.
- **Data files co-located**: Every file the script needs at runtime (layouts, `BOOK.txt`, config schemas, etc.) must be copied into `scripts/<name>/`. Resolve their paths relative to the binary location, not the working directory. The original data files in the source folder must not be referenced.
- **Fast `--agrr-meta`**: No network calls, no file I/O beyond reading embedded data.
- **Auth errors → exit 99**: Use `AgrrAuthError`/`AuthError` for credential failures.
- **Env var contract**: Read args via `AGRR_ARG_*`, credentials via `AGRR_CRED_*`, subcommand via `AGRR_SUBCOMMAND`.

## Output

When done, provide:

- Source path inspected.
- Functions discovered and which were adapted.
- New script path under `scripts/`.
- Contract checks performed (`--agrr-meta`, `--agrr-run`, build, tests).
