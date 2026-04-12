## MODIFIED Requirements

### Requirement: Candidatos de script são coletados da pasta scripts
A CLI SHALL resolver o diretório de scripts usando a seguinte lógica de prioridade:
1. Se existir uma pasta `scripts/` no diretório pai do executável da TUI (`std::env::current_exe().parent()`), usar essa pasta (modo distribuição/build)
2. Caso contrário, usar `scripts/` relativo ao diretório de trabalho atual (CWD) (modo desenvolvimento)

A CLI SHALL varrer o diretório de scripts resolvido e tratar como candidatos de script:
- Arquivos com extensão `.py`, `.js`, ou `.mjs`
- Arquivos sem extensão com bit de execução ativo (Unix) ou qualquer arquivo sem extensão (Windows)
- Subdiretórios diretos que contenham um arquivo `main` com extensão suportada (`.py`, `.js`, `.mjs`) ou um binário executável chamado `main` — exceto subdiretórios cujo nome comece com `_`

#### Scenario: Scripts resolvidos relativo ao executável (modo build)
- **WHEN** a TUI está em `build/agrr` e existe `build/scripts/` com scripts válidos
- **THEN** a TUI descobre os scripts em `build/scripts/`

#### Scenario: Fallback para CWD (modo desenvolvimento)
- **WHEN** a TUI está em `target/debug/agrr` e não existe `target/debug/scripts/`
- **WHEN** o CWD contém uma pasta `scripts/`
- **THEN** a TUI descobre os scripts em `./scripts/` (relativo ao CWD)

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
- **WHEN** o diretório de scripts resolvido não existe
- **THEN** the CLI starts with an empty menu
- **THEN** the CLI displays a message: "Nenhum script encontrado em ./scripts/"
