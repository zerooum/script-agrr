---
name: adapt-script-template
description: Use when adapting an existing script into the agrr project template. Ask for the source folder, place the adapted script under `scripts/`, and enforce the agrr subprocess contract.
license: MIT
metadata:
  author: zerooum
  version: "1.0.0"
  domain: tooling
  triggers: adapt script, script template, agrr script, --agrr-meta, --agrr-run
  role: specialist
  scope: implementation
  output-format: code
---

# Adapt Script to agrr Template

## Workflow

1. Ask the user for the source script folder/path if it was not provided.
2. Inspect the source script and identify runtime (Python, JS, Rust/binary).
3. Adapt it to agrr contract:
   - `--agrr-meta`: prints valid manifest JSON and exits 0.
   - `--agrr-run`: runs logic from env vars and exits 0/1/99.
4. Place the adapted script in `scripts/`:
   - Single-file script: `scripts/<name>.py` or `scripts/<name>.js`
   - Multi-file script: `scripts/<name>/main.py|main.js|main.mjs|main` (binary supported)
   - Avoid folder names starting with `_` because discovery ignores them.
5. Ensure the manifest includes required non-empty fields (`name`, `description`, `group`, `version`) plus optional fields (`runtime` [ignored by CLI], `global_auth`, `requires_auth`, `args`) that satisfy validation rules.
   - Use `global_auth: true` when the script should receive shared `AGRR_CRED_CHAVE` and `AGRR_CRED_SENHA`.
   - Every `args` entry must include `type`: `text`, `select`, or `multiselect`.
   - `select`/`multiselect` require at least 2 `options`.
   - `multiselect` options must not contain commas.
   - `required` defaults to `true`; set `required: false` for optional input.
   - `text` must not declare `options`; `max_length`/`pattern` are `text`-only.
   - `pattern` supports `"numeric"`, `"alpha"`, `"alphanumeric"`, or `null`.
6. Validate with existing project tests/build commands.

## Must Follow

- Never put new executable scripts outside `scripts/`.
- Prefer project SDKs (`sdk/python`, `sdk/js`, `sdk/rust`) instead of custom protocol glue.
- Keep `--agrr-meta` fast (no network calls).
- Use `AgrrAuthError`/auth-exit 99 flow for credential failures.
- Ensure args/credentials are read via `AGRR_ARG_*` and `AGRR_CRED_*`.

## Output

When done, provide:
- Source path used for adaptation.
- New script path under `scripts/`.
- Contract checks performed (`--agrr-meta`, `--agrr-run`, tests).
