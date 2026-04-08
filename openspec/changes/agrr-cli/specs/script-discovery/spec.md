## ADDED Requirements

### Requirement: CLI discovers scripts on startup
On every startup, the CLI SHALL scan the `scripts/` directory at the root of the project for script candidates and attempt to load each one by invoking `--agrr-meta`.

A file is considered a candidate if it:
- Is a `.py`, `.js`, or `.mjs` file (interpreted), OR
- Is an executable file with no extension (compiled binary, e.g., Rust)

The CLI SHALL complete discovery before rendering the TUI.

#### Scenario: Valid script discovered
- **WHEN** a candidate script returns a valid JSON manifest with exit code 0
- **THEN** the script is added to the in-memory registry
- **THEN** the script appears in the TUI menu under its declared `group`

#### Scenario: Invalid manifest discovered
- **WHEN** a candidate script returns invalid or incomplete JSON, or exits with non-zero code during `--agrr-meta`
- **THEN** the script is NOT added to the registry
- **THEN** the CLI displays at startup: "Script <filename> não carregado. Motivo: <reason>"

#### Scenario: No scripts directory
- **WHEN** the `scripts/` directory does not exist
- **THEN** the CLI starts with an empty menu
- **THEN** the CLI displays a message: "Nenhum script encontrado em ./scripts/"

### Requirement: Discovery warns about missing runtime
If a script's manifest declares a `runtime` field and the required runtime is not available on the current machine, the script SHALL NOT be loaded.

#### Scenario: Required runtime not found
- **WHEN** a script declares `runtime: { language: "python", min_version: "3.11" }` and no Python >= 3.11 is found
- **THEN** the script is NOT added to the registry
- **THEN** the CLI displays: "Script <filename> não carregado. Motivo: Python >= 3.11 não encontrado"

### Requirement: Discovery has a per-script timeout
Each `--agrr-meta` invocation SHALL be bounded by a 5-second timeout to prevent a slow or hanging script from blocking startup.

#### Scenario: Script hangs during meta invocation
- **WHEN** a script does not respond to `--agrr-meta` within 5 seconds
- **THEN** the process is killed
- **THEN** the CLI displays: "Script <filename> não carregado. Motivo: timeout na leitura do manifest (>5s)"
