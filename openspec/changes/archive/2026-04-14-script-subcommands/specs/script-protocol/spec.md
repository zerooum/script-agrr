## MODIFIED Requirements

### Requirement: Script declares metadata via --agrr-meta
A script SHALL expose its metadata by accepting the `--agrr-meta` flag and printing a valid JSON object to stdout, then exiting with code 0.

The JSON manifest MUST contain the following fields:
- `name` (string): Human-readable display name
- `description` (string): One-line description
- `group` (string): Grouping key used in the TUI menu (kebab-case)
- `version` (string): Semver string (e.g., `"1.0.0"`)

The JSON manifest MAY contain the following fields:
- `runtime` (object): Required for interpreted scripts. Contains `language` (`"python"` | `"node"`) and `min_version` (string, e.g., `"3.11"`). Omitted for compiled Rust binaries.
- `requires_auth` (array of strings): Named credential keys the script needs (e.g., `["DB_USER", "DB_PASS"]`).
- `args` (array of objects): Each arg object MUST contain `name` (string), `prompt` (string), and `type` (string, one of `"text"`, `"select"`, `"multiselect"`). MAY contain `options` (array of strings, required for `select`/`multiselect` with ≥ 2 entries), `max_length` (positive integer, `text` only), `pattern` (string: `"numeric"` | `"alpha"` | `"alphanumeric"`, `text` only), `required` (boolean, default `true`), and `default` (string).
- `global_auth` (boolean): If true, global credentials (CHAVE/SENHA) are injected.
- `subcommands` (array of objects): Optional list of named sub-operations. Each subcommand object MUST contain `name` (non-empty string, no whitespace). MAY contain `description` (string) and `args` (array of arg objects following the same schema as top-level `args`). Requires at least 2 entries. MUST NOT coexist with a non-empty top-level `args`. Subcommand names MUST be unique within the manifest.

#### Scenario: Valid manifest returned
- **WHEN** the CLI invokes a script with `--agrr-meta`
- **THEN** the script prints a JSON object to stdout conforming to the manifest schema
- **THEN** the script exits with code 0

#### Scenario: Missing required field
- **WHEN** the CLI invokes a script with `--agrr-meta` and the returned JSON is missing a required field (including `type` on any arg)
- **THEN** the script is marked as invalid
- **THEN** the CLI displays "Script <name> não carregado. Motivo: campo obrigatório ausente: <field>"

#### Scenario: Non-zero exit on meta invocation
- **WHEN** the CLI invokes a script with `--agrr-meta` and the script exits with code != 0
- **THEN** the script is marked as invalid and not loaded

#### Scenario: Arg with invalid type value
- **WHEN** the manifest contains an arg with `type` set to an unsupported value (not `text`, `select`, or `multiselect`)
- **THEN** the script is marked as invalid and not loaded

#### Scenario: Select arg with insufficient options
- **WHEN** the manifest contains an arg with `type` `"select"` and fewer than 2 entries in `options`
- **THEN** the script is marked as invalid and not loaded

#### Scenario: Valid manifest with subcommands
- **WHEN** the CLI invokes a script with `--agrr-meta` and the manifest contains a valid `subcommands` array (≥ 2 entries, unique names, no top-level `args`)
- **THEN** the script is loaded with its subcommands available for selection

#### Scenario: Invalid manifest with both args and subcommands
- **WHEN** the manifest contains both non-empty `args` and non-empty `subcommands`
- **THEN** the script is marked as invalid and not loaded

### Requirement: Script executes via --agrr-run
A script SHALL accept the `--agrr-run` flag as the signal to perform its main operation. Credentials and args SHALL be read from environment variables injected by the CLI.

Credential env vars use the pattern `AGRR_CRED_<KEY>` where `<KEY>` matches the uppercase key declared in `requires_auth`.
Arg env vars use the pattern `AGRR_ARG_<NAME>` where `<NAME>` matches the uppercase arg name from the selected subcommand's args (or top-level args if no subcommands are declared).

When a script declares subcommands and the user selects one, the CLI SHALL also set `AGRR_SUBCOMMAND=<selected_name>` in the environment.

#### Scenario: Successful execution
- **WHEN** the CLI invokes a script with `--agrr-run` and all required env vars are set
- **THEN** the script performs its operation and exits with code 0

#### Scenario: Generic failure
- **WHEN** the script encounters an error unrelated to authentication
- **THEN** the script exits with code 1
- **THEN** the CLI displays the script's stderr output to the user

#### Scenario: Execution with subcommand selected
- **WHEN** the CLI invokes a script that declares subcommands with `--agrr-run`
- **THEN** `AGRR_SUBCOMMAND` is set to the selected subcommand name
- **THEN** only the selected subcommand's `AGRR_ARG_*` env vars are injected

### Requirement: SDKs implement the protocol per language
Each supported language SHALL have an SDK in the repository that implements the `--agrr-meta` / `--agrr-run` protocol, so script authors interact with a typed abstraction, not raw flags.

- `sdk/python`: abstract base class `AgrrScript` with `meta()` and `run(creds, args)` abstract methods; `AgrrAuthError` exception mapped to exit 99. When the script declares `subcommands`, the SDK dispatches `--agrr-run` to the handler matching `AGRR_SUBCOMMAND`.
- `sdk/js`: `createAgrrScript({ meta, run })` factory; `AgrrAuthError` class mapped to exit 99. When `subcommands` handlers are provided, the SDK dispatches to the matching handler.
- `sdk/rust`: `AgrrScript` trait with `meta()` and `run()` methods; `run_script()` entry point; `AuthError` return type mapped to exit 99. When the script declares subcommands, the SDK dispatches via `run_subcommand()`.

Each SDK MUST validate during `--agrr-meta` processing that the `run` implementation exists (for scripts without subcommands) or that all declared subcommand handlers exist (for scripts with subcommands). If validation fails, the SDK MUST print a descriptive error to stderr and exit with code 1.

#### Scenario: SDK handles --agrr-meta automatically
- **WHEN** a script using the SDK is invoked with `--agrr-meta`
- **THEN** the SDK serializes the declared metadata to JSON and prints it to stdout
- **THEN** the SDK exits with code 0 without calling the user's `run` implementation

#### Scenario: SDK maps AuthError to exit 99
- **WHEN** a script's `run` implementation raises/returns an `AuthError` (or equivalent)
- **THEN** the SDK exits with code 99

#### Scenario: Python SDK rejects script without run implementation
- **WHEN** a Python script subclasses `AgrrScript` but does not override `run()` and does not declare `subcommands`
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: 'run' method not implemented" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: JS SDK rejects script without run function
- **WHEN** `createAgrrScript` is called without a `run` function (undefined, null, or non-function) and without `subcommands` handlers
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: 'run' function not provided" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: Python SDK accepts valid subclass with run
- **WHEN** a Python script subclasses `AgrrScript` and implements `run()`
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK emits valid metadata and exits with code 0 (no change in behavior)

#### Scenario: JS SDK accepts valid script with run function
- **WHEN** `createAgrrScript` is called with a valid `run` function
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK emits valid metadata and exits with code 0 (no change in behavior)
