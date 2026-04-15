## Why

Scripts currently expose a single operation with a flat list of args. Teams often group related operations into a single script (e.g., a "deploy" script that can deploy, rollback, or check status). Today, each operation must be a separate script file, leading to duplication of auth configuration, common code, and cluttered menus. Allowing scripts to declare multiple **subcommands** — each with its own name, optional description, and args — reduces boilerplate and keeps related operations together.

## What Changes

- **Manifest schema gains an optional `subcommands` field**: an array of subcommand objects, each with `name` (required), `description` (optional), and `args` (optional, same `ArgSpec` format as the current top-level `args`).
- **TUI adds a subcommand selection step**: after the user selects a script that declares subcommands, the TUI presents a selection prompt listing the available subcommands before collecting args.
- **Execution injects `AGRR_SUBCOMMAND` env var**: the CLI sets `AGRR_SUBCOMMAND=<selected_name>` so the script knows which subcommand was requested.
- **SDKs route to per-subcommand `run` handlers**: each SDK adds a way for script authors to register a handler per subcommand, and the SDK dispatcher calls the correct one based on `AGRR_SUBCOMMAND`.
- **Backward-compatible**: scripts without `subcommands` work exactly as today; top-level `args` remain the default when no subcommands are declared.
- **Mutual exclusivity**: a manifest MUST NOT declare both top-level `args` and `subcommands` simultaneously — **BREAKING** for any script that tries to use both. The validator rejects such manifests.

## Capabilities

### New Capabilities
- `script-subcommands`: Defines the subcommand manifest schema, TUI subcommand selection step, `AGRR_SUBCOMMAND` env var injection, and SDK dispatch per subcommand.

### Modified Capabilities
- `script-protocol`: The manifest optionally includes a `subcommands` array; `--agrr-run` now receives the `AGRR_SUBCOMMAND` env var when a subcommand is selected.
- `tui-shell`: After selecting a script with subcommands, the TUI shows a subcommand selection step before collecting args. Subcommand names and descriptions are displayed.

## Impact

- **CLI (`agrr/src/`)**: `manifest.rs` (new struct + validation), `app.rs` (new FSM state or extended `CollectingArgs`), `ui/` (subcommand selection renderer), `executor.rs` (inject `AGRR_SUBCOMMAND`).
- **SDKs (`sdk/python`, `sdk/js`, `sdk/rust`)**: each SDK gains a subcommand registration API and dispatch logic.
- **Existing scripts**: no change needed — manifests without `subcommands` are unaffected.
- **Example scripts**: new examples in `scripts/_examples/` demonstrating subcommands.
