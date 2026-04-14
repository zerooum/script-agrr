# script-protocol Specification

## Purpose
TBD - created by archiving change agrr-cli. Update Purpose after archive.
## Requirements
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

### Requirement: Script executes via --agrr-run
A script SHALL accept the `--agrr-run` flag as the signal to perform its main operation. Credentials and args SHALL be read from environment variables injected by the CLI.

Credential env vars use the pattern `AGRR_CRED_<KEY>` where `<KEY>` matches the uppercase key declared in `requires_auth`.
Arg env vars use the pattern `AGRR_ARG_<NAME>` where `<NAME>` matches the uppercase arg name.

#### Scenario: Successful execution
- **WHEN** the CLI invokes a script with `--agrr-run` and all required env vars are set
- **THEN** the script performs its operation and exits with code 0

#### Scenario: Generic failure
- **WHEN** the script encounters an error unrelated to authentication
- **THEN** the script exits with code 1
- **THEN** the CLI displays the script's stderr output to the user

### Requirement: Script signals auth error with exit code 99
A script that declares `requires_auth` MUST signal authentication failure by exiting with code 99. This is the only mechanism the CLI uses to detect invalid credentials.

#### Scenario: Authentication failure during execution
- **WHEN** the script attempts to authenticate with the provided credentials and fails
- **THEN** the script exits with code 99

#### Scenario: Exit 99 triggers credential removal
- **WHEN** the CLI receives exit code 99 from a script
- **THEN** the CLI deletes the stored credentials for that script's `requires_auth` keys from the OS Keychain
- **THEN** the CLI informs the user: "Credenciais inválidas. As credenciais salvas foram removidas."
- **THEN** the CLI prompts the user to enter credentials again

### Requirement: SDKs implement the protocol per language
Each supported language SHALL have an SDK in the repository that implements the `--agrr-meta` / `--agrr-run` protocol, so script authors interact with a typed abstraction, not raw flags.

- `sdk/python`: abstract base class `AgrrScript` with `meta()` and `run(creds, args)` abstract methods; `AgrrAuthError` exception mapped to exit 99
- `sdk/js`: `createAgrrScript({ meta, run })` factory; `AgrrAuthError` class mapped to exit 99
- `sdk/rust`: `AgrrScript` trait with `meta()` and `run()` methods; `run_script()` entry point; `AuthError` return type mapped to exit 99

#### Scenario: SDK handles --agrr-meta automatically
- **WHEN** a script using the SDK is invoked with `--agrr-meta`
- **THEN** the SDK serializes the declared metadata to JSON and prints it to stdout
- **THEN** the SDK exits with code 0 without calling the user's `run` implementation

#### Scenario: SDK maps AuthError to exit 99
- **WHEN** a script's `run` implementation raises/returns an `AuthError` (or equivalent)
- **THEN** the SDK exits with code 99

