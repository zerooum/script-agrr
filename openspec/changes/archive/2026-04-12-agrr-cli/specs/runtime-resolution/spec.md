## ADDED Requirements

### Requirement: CLI resolves Python runtime using pyenv if available
When a script declares `runtime.language = "python"`, the CLI SHALL attempt to resolve the interpreter using pyenv before falling back to PATH.

Resolution order:
1. If `pyenv` is detected on PATH: list installed versions, select highest version satisfying `>= min_version`
2. If pyenv not found or no satisfying version installed: search PATH for `python{major}.{minor}`, then `python3`, then `python`
3. Verify version of found binary satisfies `>= min_version`
4. If no satisfying interpreter found: mark script as invalid with appropriate message

#### Scenario: pyenv has a satisfying version
- **WHEN** pyenv is detected and has Python 3.12.2 installed, and script requires `>= 3.11`
- **THEN** the CLI selects Python 3.12.2 via pyenv for this script's execution

#### Scenario: pyenv installed but no satisfying version
- **WHEN** pyenv is detected but only has Python 3.9.x installed, and script requires `>= 3.11`
- **THEN** the CLI falls back to PATH search
- **WHEN** PATH also has no Python >= 3.11
- **THEN** the script is not loaded with message: "Script <name> não carregado. Motivo: Python >= 3.11 não encontrado (pyenv e PATH verificados)"

#### Scenario: pyenv not installed, PATH has compatible version
- **WHEN** pyenv is not found and `python3.11` exists on PATH with version 3.11.9
- **THEN** the CLI uses `python3.11` to execute the script

### Requirement: CLI resolves Node runtime using nvm if available
When a script declares `runtime.language = "node"`, the CLI SHALL attempt to resolve the interpreter using nvm before falling back to PATH.

Resolution order:
1. If `nvm` is detected: list installed versions, select highest version satisfying `>= min_version` (major-level match)
2. If nvm not found: search PATH for `node`, verify major version satisfies `>= min_version`
3. If no satisfying interpreter found: mark script as invalid

#### Scenario: nvm has a satisfying version
- **WHEN** nvm is detected and has Node 20.11.0 installed, and script requires `>= 18`
- **THEN** the CLI uses Node 20.11.0 via nvm for this script's execution

#### Scenario: No satisfying Node version anywhere
- **WHEN** neither nvm nor PATH provide a Node version >= required min
- **THEN** script is not loaded with message: "Script <name> não carregado. Motivo: Node >= <min_version> não encontrado"

### Requirement: Compiled Rust binaries skip runtime resolution
When a script candidate is an executable binary with no `runtime` field in its manifest (or omits the manifest's runtime field), the CLI SHALL invoke it directly without any runtime resolution.

#### Scenario: Binary executed directly
- **WHEN** a script is a compiled binary and its manifest has no `runtime` field
- **THEN** the CLI calls the binary directly for both `--agrr-meta` and `--agrr-run`
- **THEN** no runtime version check is performed

### Requirement: Selected runtime is logged at execution time
When a script is executed, the CLI SHALL display which runtime was selected, to aid debugging across different machines.

#### Scenario: Runtime logged before execution
- **WHEN** the user executes a Python script
- **THEN** the CLI displays before output: "Executando com Python 3.11.9 via pyenv" (or "via PATH")
