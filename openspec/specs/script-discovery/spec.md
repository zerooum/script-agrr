# script-discovery Specification

## Purpose
Defines how the CLI discovers script candidates inside `scripts/`, including single-file scripts, multi-file folder scripts, runtime validation, and startup timeout behavior.

## Requirements
### Requirement: Candidatos de script são coletados da pasta scripts
A CLI SHALL varrer o diretório `scripts/` no startup e tratar como candidatos de script:
- Arquivos com extensão `.py`, `.js`, ou `.mjs`
- Arquivos sem extensão com bit de execução ativo (Unix) ou qualquer arquivo sem extensão (Windows)
- Subdiretórios diretos que contenham um arquivo `main` com extensão suportada (`.py`, `.js`, `.mjs`) ou um binário executável chamado `main` — exceto subdiretórios cujo nome comece com `_`

#### Scenario: Arquivo .py é candidato
- **WHEN** existe `scripts/script.py`
- **THEN** `script.py` é incluído na lista de candidatos

#### Scenario: Arquivo .js é candidato
- **WHEN** existe `scripts/script.js`
- **THEN** `script.js` é incluído na lista de candidatos

#### Scenario: Binário executável sem extensão é candidato (Unix)
- **WHEN** existe `scripts/meu-binario` com permissão de execução
- **THEN** `meu-binario` é incluído na lista de candidatos

#### Scenario: Subdiretório com main.py é candidato
- **WHEN** existe `scripts/meu-script/main.py`
- **THEN** `meu-script/main.py` é incluído como candidato representando o script `meu-script`

#### Scenario: Subdiretório sem main não é candidato
- **WHEN** existe `scripts/meu-script/` sem arquivo `main` com extensão suportada
- **THEN** `meu-script` não é incluído como candidato e um warning é emitido

#### Scenario: Subdiretório com nome começando por underscore é ignorado
- **WHEN** existe `scripts/_utils/main.py`
- **THEN** `_utils` não é incluído como candidato e nenhum warning é emitido

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

