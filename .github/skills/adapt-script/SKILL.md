---
name: adapt-script
description: Adapt an existing script to the agrr project template. The user provides the source folder and the skill produces a ready-to-run script in `scripts/` that follows the agrr subprocess protocol and SDK conventions.
metadata:
  version: "1.0.0"
  domain: scripting
  triggers: adapt script, migrate script, convert script, port script, onboard script, new agrr script
  role: specialist
  scope: implementation
  output-format: code
---

# Adapt Script to agrr

Adapts an existing script (Python, JS, or Rust) into the agrr subprocess protocol. The output is a script placed under `scripts/` that the agrr CLI can discover, collect credentials/args via TUI, and execute.

## Inputs

The user MUST provide:
1. **Source folder/file** — path to the script to be adapted.

If not provided, ask before proceeding. Do NOT guess.

## Core Workflow

### 1. Analyze the source script

Read all files in the source folder. Identify:

- **Language** — Python (`.py`), JavaScript (`.js`/`.mjs`), or Rust.
- **Purpose** — what the script does (one-line description).
- **Credentials** — any API keys, passwords, tokens, or login values currently hardcoded, read from env, config files, or CLI args. List each with a proposed key name (UPPERCASE, no spaces).
- **Arguments** — any user inputs the script expects (CLI flags, prompts, config values). For each, determine:
  - `type`: `"text"`, `"select"`, or `"multiselect"`.
  - `options`: for `select`/`multiselect`, list at least 2 choices.
  - Constraints: `max_length`, `pattern` (`"numeric"`, `"alpha"`, `"alphanumeric"`), `required`, `default`.
- **Global auth** — does the script use shared corporate credentials (login + password)? If yes, set `global_auth = True` and map to `CHAVE`/`SENHA`.
- **Subcommands** — does the script have multiple distinct operations? If yes, map to `subcommands` (minimum 2; mutually exclusive with top-level `args`).
- **Dependencies** — external libraries the script imports.

Present a summary to the user and confirm before proceeding.

### 2. Decide script structure

| Condition | Structure |
|---|---|
| Single file, no helper modules | Single file: `scripts/<name>.<ext>` |
| Multiple files or helper modules | Folder: `scripts/<name>/main.<ext>` + helper files |

Naming rules:
- Use `snake_case` for file/folder names.
- Do NOT prefix with `_` (that hides scripts from discovery).
- Folder names must not conflict with existing entries in `scripts/`.

### 3. Build the agrr-compatible script

Use the appropriate SDK. Follow these templates exactly.

#### Python template

```python
"""<One-line description>."""

import sys
sys.path.insert(0, "sdk/python")
# For folder scripts use relative path:
# sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript, AgrrAuthError


class <ClassName>(AgrrScript):
    name = "<Display Name>"
    description = "<One-line description>"
    group = "<group-kebab-case>"
    version = "1.0.0"

    # Optional fields — include only when needed:
    # runtime = {"language": "python", "min_version": "3.8"}
    # global_auth = True
    # requires_auth = ["API_KEY", "TOKEN"]

    args = [
        # Every arg MUST have: name, prompt, type
        # {"name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"]},
        # {"name": "code", "prompt": "Code?", "type": "text", "pattern": "numeric", "max_length": 6},
    ]

    def run(self, creds: dict, args: dict) -> None:
        # Access credentials: creds["API_KEY"], creds["CHAVE"], creds["SENHA"]
        # Access arguments: args["env"], args["code"]
        # On auth failure: raise AgrrAuthError()
        ...


if __name__ == "__main__":
    <ClassName>.main()
```

#### JavaScript template

```javascript
'use strict';

const path = require('path');
const { createAgrrScript, AgrrAuthError } = require(path.join(__dirname, '..', 'sdk', 'js', 'index.js'));
// For folder scripts:
// require(path.join(__dirname, '..', '..', 'sdk', 'js', 'index.js'));

createAgrrScript({
  meta: {
    name: '<Display Name>',
    description: '<One-line description>',
    group: '<group-kebab-case>',
    version: '1.0.0',
    // Optional:
    // runtime: { language: 'node', min_version: '18' },
    // global_auth: true,
    // requires_auth: ['API_KEY', 'TOKEN'],
    args: [
      // Every arg MUST have: name, prompt, type
      // { name: 'env', prompt: 'Environment?', type: 'select', options: ['prod', 'staging'] },
    ],
  },

  async run({ creds, args }) {
    // Access credentials: creds.API_KEY, creds.CHAVE, creds.SENHA
    // Access arguments: args.env
    // On auth failure: throw new AgrrAuthError();
  },
});
```

#### Rust template

```rust
use agrr_script_sdk::{
    AgrrScript, ArgSpec, ArgType, Args, AuthError, Credentials, ScriptMeta, run_script,
};

struct <StructName>;

impl AgrrScript for <StructName> {
    fn meta(&self) -> ScriptMeta {
        ScriptMeta {
            name: "<Display Name>".into(),
            description: "<One-line description>".into(),
            group: "<group-kebab-case>".into(),
            version: "1.0.0".into(),
            runtime: None,
            requires_auth: vec![],
            global_auth: false,
            args: vec![],
            // subcommands: vec![],
        }
    }

    fn run(&self, creds: Credentials, _args: Args) -> Result<(), AuthError> {
        // Access credentials: creds.get("API_KEY")
        // Access arguments: Args::get("env")
        // On auth failure: return Err(AuthError);
        Ok(())
    }
}

fn main() {
    run_script(<StructName>);
}
```

### 4. Migrate the logic

Apply these transformations to the original code:

| Original pattern | agrr equivalent |
|---|---|
| Hardcoded password/token | `requires_auth = ["KEY"]` → accessed via `creds["KEY"]` |
| `input()` / `readline` / CLI arg | `args = [{"name": ..., "type": ...}]` → accessed via `args["name"]` |
| Env var for credentials | `requires_auth` — the CLI injects `AGRR_CRED_<KEY>` |
| Env var for config | `args` with appropriate type |
| `argparse` / `click` / `commander` subcommands | `subcommands` list (≥ 2 entries) |
| `sys.exit(1)` on auth failure | `raise AgrrAuthError()` / `throw new AgrrAuthError()` / `return Err(AuthError)` |
| Print to stdout | Keep as-is — the TUI streams stdout |
| Print to stderr | Keep as-is — the TUI streams stderr |

Rules:
- **Never** hardcode credentials in the adapted script.
- **Never** call remote APIs during `--agrr-meta` (5-second timeout enforced).
- Keep the original business logic intact — only change how inputs are received and how errors are signaled.
- If the script has external dependencies, note them but do NOT add a `requirements.txt` or `package.json` installation step inside the script.

### 5. Validate the manifest contract

Before finishing, verify:

- [ ] `name`, `description`, `group`, `version` — all non-empty strings.
- [ ] Every arg has `type` field (`"text"`, `"select"`, or `"multiselect"`).
- [ ] `select`/`multiselect` args have `options` with ≥ 2 entries.
- [ ] `text` args do NOT have `options`.
- [ ] `max_length` and `pattern` only on `text` args.
- [ ] `default` for `select` is one of its `options`.
- [ ] `multiselect` options do NOT contain commas.
- [ ] If `subcommands` is used: ≥ 2 entries, top-level `args` is empty, each subcommand has a matching handler method/function.
- [ ] `requires_auth` keys are UPPERCASE with no spaces.
- [ ] `global_auth` scripts do NOT declare `CHAVE` or `SENHA` in `requires_auth` (those are injected automatically).
- [ ] The SDK import path resolves correctly from the script's location in `scripts/`.

### 6. Test the script

Run the manifest check:

```bash
# Python
python3 scripts/<name>.py --agrr-meta

# JavaScript
node scripts/<name>.js --agrr-meta

# Rust (after building)
scripts/<name>/main --agrr-meta
```

Verify the JSON output is valid and contains all required fields.

Then run a quick smoke test with env vars:

```bash
# Python example
AGRR_ARG_ENV=prod python3 scripts/<name>.py --agrr-run

# JavaScript example
AGRR_ARG_ENV=prod node scripts/<name>.js --agrr-run
```

### 7. Report to user

Summarize:
- Script location in `scripts/`.
- Credentials it requires (and whether global auth is used).
- Arguments it collects.
- Any external dependencies that must be installed separately.
- Any logic changes or simplifications made.

## Validation Rules Quick-Reference

| Rule | Enforced by |
|---|---|
| `--agrr-meta` must return JSON to stdout and exit 0 | `discovery.rs` |
| `--agrr-meta` must complete within 5 seconds | `discovery.rs` |
| `--agrr-run` exits 0 (success), 1 (error), 99 (auth failure) | `executor.rs` |
| Exit 99 deletes ALL creds in `requires_auth` from keychain | `app.rs` |
| `AGRR_CRED_<KEY>` env vars injected for credentials | `executor.rs` |
| `AGRR_ARG_<NAME>` env vars injected for arguments | `executor.rs` |
| `AGRR_SUBCOMMAND` env var injected when subcommands are used | `executor.rs` |
| Multiselect values joined with `,` | `app.rs` |
| Credential keys containing SENHA/PASSWORD/SECRET are masked in TUI | `ui.rs` |
| Folders prefixed with `_` are ignored by discovery | `discovery.rs` |

## SDK Import Paths

| Script location | Python `sys.path.insert` | JS `require` path |
|---|---|---|
| `scripts/<name>.py` | `sys.path.insert(0, "sdk/python")` | `path.join(__dirname, '..', 'sdk', 'js', 'index.js')` |
| `scripts/<folder>/main.py` | `sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "sdk", "python"))` | `path.join(__dirname, '..', '..', 'sdk', 'js', 'index.js')` |
