## 1. Manifest Schema

- [x] 1.1 Add `SubcommandSpec` struct to `agrr/src/manifest.rs` with fields `name: String`, `description: Option<String>`, `args: Vec<ArgSpec>`
- [x] 1.2 Add optional `subcommands: Vec<SubcommandSpec>` field to `ScriptManifest` (default empty)
- [x] 1.3 Add validation: `subcommands` requires ≥ 2 entries when non-empty
- [x] 1.4 Add validation: non-empty `args` and non-empty `subcommands` are mutually exclusive
- [x] 1.5 Add validation: subcommand `name` must be non-empty, no whitespace, unique within manifest
- [x] 1.6 Add validation: each subcommand's `args` are validated with the same rules as top-level `args`
- [x] 1.7 Add `ManifestError` variants for all new validation failures
- [x] 1.8 Add unit tests for manifest parsing and validation of subcommands

## 2. TUI Subcommand Selection

- [x] 2.1 Add `SelectingSubcommand` variant to `Mode` enum in `agrr/src/app.rs` with `script_idx`, `cursor`, and `pending_creds` fields
- [x] 2.2 Update `Enter` handler in `Menu` mode: if selected script has subcommands, transition to `SelectingSubcommand` instead of `CollectingCred`/`CollectingArgs`
- [x] 2.3 Implement `SelectingSubcommand` key handling: `↑`/`↓`/`j`/`k` navigate, `Enter` selects and transitions to credential/arg collection, `Esc` returns to menu
- [x] 2.4 After subcommand selection, resolve the selected subcommand's args and pass them through the existing `CollectingArgs` flow
- [x] 2.5 Add subcommand selection renderer in `agrr/src/ui/` showing subcommand names and descriptions

## 3. Executor

- [x] 3.1 Update `build_run_command` in `agrr/src/executor.rs` to inject `AGRR_SUBCOMMAND` env var when a subcommand was selected
- [x] 3.2 Ensure `AGRR_ARG_*` env vars come from the selected subcommand's collected args (not top-level args)

## 4. Python SDK

- [x] 4.1 Add `subcommands` class attribute to `AgrrScript` (dict mapping name → method name, default empty)
- [x] 4.2 Include `subcommands` in `--agrr-meta` JSON output (array format with name, description, args)
- [x] 4.3 On `--agrr-run`, check `AGRR_SUBCOMMAND` env var and dispatch to the matching method
- [x] 4.4 Validate at `--agrr-meta` time that all referenced subcommand methods exist; exit 1 if not
- [x] 4.5 Add unit tests for subcommand dispatch, validation, and meta output

## 5. JS SDK

- [x] 5.1 Accept optional `subcommands` object in `createAgrrScript` (mapping name → handler function)
- [x] 5.2 Include `subcommands` in `--agrr-meta` JSON output
- [x] 5.3 On `--agrr-run`, check `AGRR_SUBCOMMAND` env var and dispatch to the matching handler
- [x] 5.4 Validate at `--agrr-meta` time that all subcommand handlers are functions; exit 1 if not
- [x] 5.5 Add unit tests for subcommand dispatch, validation, and meta output

## 6. Rust SDK

- [x] 6.1 Add `SubcommandSpec` struct and `subcommands` field to `ScriptMeta` in `sdk/rust/src/lib.rs`
- [x] 6.2 Add `run_subcommand(&self, name: &str, creds, args)` method to the `AgrrScript` trait (default impl returns error)
- [x] 6.3 On `--agrr-run`, check `AGRR_SUBCOMMAND` env var and dispatch to `run_subcommand` or `run`
- [x] 6.4 Add unit tests for subcommand dispatch and meta serialization

## 7. Example Scripts

- [x] 7.1 Create Python example script in `scripts/_examples/subcommands/main.py` demonstrating 2+ subcommands with args
- [x] 7.2 Create JS example script in `scripts/_examples/subcommands/main.js` demonstrating 2+ subcommands with args
- [x] 7.3 Create Rust example script in `scripts/_examples/subcommands/rust/` demonstrating 2+ subcommands with args

## 8. Integration Testing

- [x] 8.1 Add CLI integration test: discover a script with valid subcommands manifest
- [x] 8.2 Add CLI integration test: reject manifest with both `args` and `subcommands`
- [x] 8.3 Add CLI integration test: reject manifest with < 2 subcommands
- [x] 8.4 Add CLI integration test: `AGRR_SUBCOMMAND` env var is injected on execution
