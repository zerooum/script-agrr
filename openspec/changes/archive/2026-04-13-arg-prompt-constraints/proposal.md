## Why

Script authors currently have no way to constrain user input for arguments beyond a simple free-text field or a flat list of options. This means the CLI cannot validate input length, enforce numeric-only values, require non-empty answers, or offer multi-select prompts. Every script must either accept arbitrary strings or duplicate validation logic internally, leading to inconsistent UX and repeated effort across scripts.

Adding structured prompt constraints to the manifest lets script authors declaratively define input rules. The TUI enforces them before execution, so scripts receive clean, pre-validated data.

## What Changes

- Extend the `ArgSpec` manifest struct with new optional and required fields:
  - **`type`** (required): prompt type — `"text"`, `"select"`, or `"multiselect"`.
  - **`max_length`**: maximum number of characters (applies to `text`).
  - **`pattern`**: character class constraint — `"numeric"`, `"alpha"`, `"alphanumeric"`, or a custom regex (applies to `text`).
  - **`required`**: whether the field rejects empty input (default `true`).
  - **`default`**: default value used when the user submits blank input (implies `required: false`).
- `select` type replaces the current `options` array semantic — `options` becomes mandatory and must have ≥ 2 entries.
- `multiselect` type allows choosing one or more options from the `options` list (≥ 2 entries required).
- **Manifest validation** rejects scripts that:
  - Omit `type` on any arg.
  - Declare `select` or `multiselect` without ≥ 2 `options`.
- **TUI** enforces constraints inline (error messages, input masking for numeric, etc.) and renders `multiselect` as a checkbox list.
- **SDKs** (Python, JS, Rust) expose the new fields in their arg helpers / types.
- **BREAKING**: `type` becomes a required field on args. Existing scripts that declare `args` without `type` will fail validation and not load.

## Capabilities

### New Capabilities
- `arg-prompt-constraints`: Declarative prompt constraints on script arguments — type, max_length, pattern, required, default — with TUI enforcement and manifest validation.

### Modified Capabilities
- `script-protocol`: The `--agrr-meta` manifest schema gains new required (`type`) and optional fields on arg objects, plus stricter validation rules for `select`/`multiselect`.
- `tui-shell`: The arg-collection screen gains inline validation, `multiselect` rendering, default-value behavior, and constraint error feedback.

## Impact

- **`agrr/src/manifest.rs`**: `ArgSpec` struct and `validate()` gain new fields / rules.
- **`agrr/src/app.rs`**: `CollectingArgs` state handles multiselect toggling and per-keystroke validation.
- **`agrr/src/ui/prompts.rs`**: Render logic for text constraints, select, multiselect, defaults, inline errors.
- **`agrr/src/executor.rs`**: Multiselect values need a serialization convention for the env var (e.g., comma-separated).
- **`sdk/python/agrr_sdk/__init__.py`**: Arg dict accepts new keys; docstrings updated.
- **`sdk/js/index.js`**: Meta arg schema accepts new keys; JSDoc updated.
- **`sdk/rust/src/lib.rs`**: `ArgSpec` struct gains new fields.
- **`scripts/` examples**: Update or add examples exercising new arg types.
- **Breaking change**: scripts declaring `args` without `type` will not load. Migration path: add `"type": "text"` (or `"select"` if `options` is present) to each arg.
