## 1. Manifest Schema & Validation (Rust CLI)

- [x] 1.1 Add `ArgType` enum (`Text`, `Select`, `MultiSelect`) and `Pattern` enum (`Numeric`, `Alpha`, `Alphanumeric`) to `manifest.rs`
- [x] 1.2 Extend `ArgSpec` struct with fields: `arg_type` (required), `max_length`, `pattern`, `required`, `default`
- [x] 1.3 Add `ManifestError` variants for new validation rules (missing type, insufficient options, invalid default, text with options, constraint on wrong type, comma in multiselect option)
- [x] 1.4 Update `validate()` to enforce: `type` required on every arg, `select`/`multiselect` need ≥ 2 options, `text` must not have `options`, `max_length`/`pattern` only on `text`, `default` on `select` must be a valid option, multiselect options must not contain commas
- [x] 1.5 Update existing manifest tests and add new tests for all new validation rules

## 2. TUI Arg Collection (Rust CLI)

- [x] 2.1 Update `CollectingArgs` state in `app.rs` to track multiselect toggles and per-arg validation errors
- [x] 2.2 Implement keystroke filtering in `app.rs` arg input handler: enforce `pattern` (reject non-matching chars), enforce `max_length` (reject past limit)
- [x] 2.3 Implement submit validation in `app.rs`: `required` check, apply `default` when input is blank
- [x] 2.4 Implement multiselect state: `Space` to toggle, `Enter` to confirm, comma-join selected values
- [x] 2.5 Update `select` navigation: if `default` is set, start cursor on that option

## 3. TUI Rendering (Rust CLI)

- [x] 3.1 Update `render_arg_prompt` in `ui/prompts.rs` to branch on `ArgType` (text, select, multiselect)
- [x] 3.2 Render `text` prompt with default hint `(padrão: <value>)` and inline error messages
- [x] 3.3 Render `select` prompt with `>` cursor indicator (reuse existing logic, add default pre-selection)
- [x] 3.4 Render `multiselect` prompt with `☑`/`☐` checkboxes, `Space` toggle hint, and inline error for zero selection

## 4. Executor (Rust CLI)

- [x] 4.1 Ensure multiselect comma-separated values are correctly injected into `AGRR_ARG_<NAME>` env var (verify existing path handles it — may need no changes if arg values are already plain strings)

## 5. Python SDK

- [x] 5.1 Update `args` docstring and `_build_meta` in `agrr_sdk/__init__.py` to accept and serialize new fields (`type`, `max_length`, `pattern`, `required`, `default`)
- [x] 5.2 Add Python SDK tests for manifests with new arg fields

## 6. JavaScript SDK

- [x] 6.1 Update JSDoc and meta serialization in `index.js` to accept and pass through new arg fields
- [x] 6.2 Add JS SDK tests for manifests with new arg fields

## 7. Rust SDK

- [x] 7.1 Extend `ArgSpec` in `sdk/rust/src/lib.rs` with `arg_type`, `max_length`, `pattern`, `required`, `default` fields and corresponding enums
- [x] 7.2 Add Rust SDK tests for the new `ArgSpec` fields

## 8. Example Scripts & Documentation

- [x] 8.1 Update example scripts in `scripts/_examples/` to use `type` field and demonstrate text, select, and multiselect args with constraints
- [x] 8.2 Update `copilot-instructions.md` to document the new arg schema and breaking change
