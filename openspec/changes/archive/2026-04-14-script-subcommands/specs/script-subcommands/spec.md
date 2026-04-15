## ADDED Requirements

### Requirement: Manifest declares subcommands
A script manifest MAY include a `subcommands` field containing an array of subcommand objects. Each subcommand object MUST contain a `name` field (non-empty string, no whitespace). Each subcommand object MAY contain a `description` field (string) and an `args` field (array of arg objects following the same `ArgSpec` schema as top-level `args`).

#### Scenario: Valid subcommands declared
- **WHEN** a script manifest contains `"subcommands": [{"name": "deploy", "description": "Deploy to env", "args": [...]}, {"name": "rollback"}]`
- **THEN** the manifest is valid and the script is loaded with two subcommands

#### Scenario: Subcommand with args
- **WHEN** a subcommand declares `"args": [{"name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"]}]`
- **THEN** the args are validated using the same rules as top-level args (type required, select needs ≥ 2 options, etc.)

#### Scenario: Subcommand without description
- **WHEN** a subcommand declares only `"name": "status"` without a `description` field
- **THEN** the manifest is valid and the subcommand has no description

#### Scenario: Subcommand without args
- **WHEN** a subcommand declares no `args` field or an empty `args` array
- **THEN** the manifest is valid and the subcommand has no args to collect

### Requirement: Subcommands require at least 2 entries
A manifest that declares `subcommands` MUST provide at least 2 subcommand entries.

#### Scenario: Single subcommand rejected
- **WHEN** a script manifest contains `"subcommands"` with only 1 entry
- **THEN** the script is marked as invalid and not loaded
- **THEN** the CLI displays a warning: "subcommands requires at least 2 entries"

#### Scenario: Two subcommands accepted
- **WHEN** a script manifest contains `"subcommands"` with 2 entries
- **THEN** the manifest is valid

### Requirement: Subcommands and top-level args are mutually exclusive
A manifest MUST NOT declare both a non-empty `args` array and a non-empty `subcommands` array.

#### Scenario: Both args and subcommands present
- **WHEN** a script manifest contains both `"args": [...]` (non-empty) and `"subcommands": [...]` (non-empty)
- **THEN** the script is marked as invalid and not loaded
- **THEN** the CLI displays a warning: "manifest must not declare both `args` and `subcommands`"

#### Scenario: Empty args with subcommands is valid
- **WHEN** a script manifest contains `"args": []` and `"subcommands": [{"name": "a"}, {"name": "b"}]`
- **THEN** the manifest is valid (empty `args` does not conflict)

### Requirement: Subcommand names are unique and non-empty
Each subcommand `name` within a manifest MUST be non-empty, MUST NOT contain whitespace, and MUST be unique among all subcommands in that manifest. Names are case-sensitive.

#### Scenario: Duplicate subcommand names rejected
- **WHEN** a manifest contains two subcommands with `"name": "deploy"`
- **THEN** the script is marked as invalid and not loaded
- **THEN** the CLI displays a warning: "duplicate subcommand name: deploy"

#### Scenario: Empty subcommand name rejected
- **WHEN** a manifest contains a subcommand with `"name": ""`
- **THEN** the script is marked as invalid and not loaded

#### Scenario: Subcommand name with whitespace rejected
- **WHEN** a manifest contains a subcommand with `"name": "run deploy"`
- **THEN** the script is marked as invalid and not loaded
- **THEN** the CLI displays a warning: "subcommand name must not contain whitespace"

### Requirement: CLI injects AGRR_SUBCOMMAND env var
When executing a script that declares subcommands, the CLI SHALL set the `AGRR_SUBCOMMAND` environment variable to the name of the selected subcommand before invoking `--agrr-run`. For scripts without subcommands, `AGRR_SUBCOMMAND` SHALL NOT be set.

#### Scenario: Subcommand env var injected
- **WHEN** the user selects the "deploy" subcommand of a script
- **THEN** the CLI invokes `--agrr-run` with `AGRR_SUBCOMMAND=deploy` in the environment

#### Scenario: No subcommand env var for regular scripts
- **WHEN** the user executes a script that does not declare subcommands
- **THEN** `AGRR_SUBCOMMAND` is not present in the environment

### Requirement: TUI shows subcommand selection step
When the user selects a script that declares subcommands and presses Enter, the TUI SHALL display a single-choice list of subcommand names. If a subcommand has a `description`, it SHALL be shown alongside the name. After the user selects a subcommand, the TUI proceeds to collect that subcommand's `args` (if any), then executes.

#### Scenario: Subcommand list displayed
- **WHEN** the user presses Enter on a script with subcommands `["deploy", "rollback", "status"]`
- **THEN** the TUI shows a selection list with the three subcommand names

#### Scenario: Subcommand description displayed
- **WHEN** a subcommand has `"name": "deploy"` and `"description": "Deploy to target environment"`
- **THEN** the TUI shows the name and description together in the selection list

#### Scenario: User selects subcommand then collects args
- **WHEN** the user selects a subcommand that has args
- **THEN** the TUI proceeds to collect the subcommand's args using the standard arg collection flow
- **THEN** after all args are collected, the script is executed

#### Scenario: User selects subcommand without args
- **WHEN** the user selects a subcommand that has no args
- **THEN** the TUI proceeds directly to execution (after credentials, if applicable)

#### Scenario: Esc cancels subcommand selection
- **WHEN** the user presses Esc during the subcommand selection step
- **THEN** the TUI returns to the main menu without executing

### Requirement: SDKs dispatch to subcommand handlers
When a script declares subcommands, each SDK SHALL route `--agrr-run` execution to the handler matching the `AGRR_SUBCOMMAND` env var value. If `AGRR_SUBCOMMAND` is set but no matching handler exists, the SDK SHALL print an error to stderr and exit with code 1.

#### Scenario: Python SDK dispatches to subcommand method
- **WHEN** a Python script declares `subcommands = {"deploy": "run_deploy", "rollback": "run_rollback"}`
- **WHEN** the script is invoked with `--agrr-run` and `AGRR_SUBCOMMAND=deploy`
- **THEN** the SDK calls `self.run_deploy(creds, args)` instead of `self.run(creds, args)`

#### Scenario: JS SDK dispatches to subcommand function
- **WHEN** a JS script passes `subcommands: { deploy: async ({ creds, args }) => { ... } }`
- **WHEN** the script is invoked with `--agrr-run` and `AGRR_SUBCOMMAND=deploy`
- **THEN** the SDK calls the `deploy` function with `{ creds, args }`

#### Scenario: Rust SDK dispatches to subcommand handler
- **WHEN** a Rust script implements `run_subcommand(&self, name: &str, creds, args)`
- **WHEN** the script is invoked with `--agrr-run` and `AGRR_SUBCOMMAND=deploy`
- **THEN** the SDK calls `run_subcommand` with `name = "deploy"`

#### Scenario: Unknown subcommand name exits with error
- **WHEN** the script is invoked with `AGRR_SUBCOMMAND=unknown` and no handler matches
- **THEN** the SDK prints "agrr-sdk: unknown subcommand 'unknown'" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: Python SDK validates subcommand handlers at meta time
- **WHEN** a Python script declares `subcommands` but a referenced method does not exist
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: subcommand handler '<method>' not found" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: JS SDK validates subcommand handlers at meta time
- **WHEN** a JS script passes `subcommands` where a value is not a function
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: subcommand handler '<name>' is not a function" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: SDK falls back to run() when no subcommands declared
- **WHEN** a script does not declare subcommands and `AGRR_SUBCOMMAND` is not set
- **THEN** the SDK calls the standard `run()` method (existing behavior unchanged)
