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
