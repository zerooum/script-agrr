# folder-script-discovery Specification

## Purpose
Defines how the CLI discovers and loads multi-file folder scripts inside `scripts/`, including the main entry point resolution, warning behavior for invalid folders, depth limits, and underscore-prefixed folder exclusions.

## Requirements

### Requirement: Pasta com arquivo main é tratada como script candidato
A CLI SHALL reconhecer subdiretórios diretos de `scripts/` como candidatos de script quando contiverem um arquivo de entrada válido chamado `main` com extensão suportada (`.py`, `.js`, `.mjs`) ou um binário executável chamado `main` sem extensão.

#### Scenario: Pasta com main.py é carregada com sucesso
- **WHEN** existe `scripts/meu-script/main.py` que implementa o protocolo `--agrr-meta`
- **THEN** o script é carregado no registry com `ScriptEntry.path` apontando para `main.py`

#### Scenario: Pasta com main.js é carregada com sucesso
- **WHEN** existe `scripts/meu-script/main.js` que implementa o protocolo `--agrr-meta`
- **THEN** o script é carregada no registry com `ScriptEntry.path` apontando para `main.js`

#### Scenario: Pasta com binário main é carregada com sucesso
- **WHEN** existe `scripts/meu-script/main` (executável) que implementa o protocolo `--agrr-meta`
- **THEN** o script é carregado no registry com `ScriptEntry.path` apontando para o binário `main`

### Requirement: Pasta sem arquivo main gera warning e é ignorada
A CLI SHALL emitir um `LoadWarning` descritivo para subdiretórios de `scripts/` que não contenham um arquivo `main` válido, e SHALL ignorar essa pasta na construção do registry.

#### Scenario: Pasta sem main gera warning com nome da pasta
- **WHEN** existe `scripts/meu-script/` sem nenhum arquivo `main` com extensão suportada
- **THEN** um `LoadWarning` é emitido com `filename` igual ao nome da pasta (`meu-script`) e mensagem indicando ausência do arquivo `main`
- **THEN** o diretório não é incluído no registry de scripts válidos

#### Scenario: Pasta vazia gera warning
- **WHEN** existe `scripts/pasta-vazia/` sem nenhum arquivo
- **THEN** um `LoadWarning` é emitido para `pasta-vazia`

### Requirement: Descoberta de pasta é limitada a um nível de profundidade
A CLI SHALL inspecionar apenas subdiretórios diretos de `scripts/`; subdiretórios aninhados (ex.: `scripts/foo/bar/`) SHALL NOT ser tratados como candidatos de script.

#### Scenario: Subdiretório aninhado não é tratado como script
- **WHEN** existe `scripts/foo/bar/main.py`
- **THEN** `bar/` não é tratado como candidato e nenhum warning é emitido para `bar`

### Requirement: Pastas com nome iniciado por underscore são ignoradas
A CLI SHALL ignorar subdiretórios de `scripts/` cujo nome comece com `_` (underscore), tratando-os como pastas de suporte não-executáveis (ex.: `_examples/`, `_shared/`).

#### Scenario: Pasta _examples não gera warning nem candidato
- **WHEN** existe `scripts/_examples/` com ou sem arquivo `main`
- **THEN** nenhum warning é emitido e a pasta não é incluída no registry

### Requirement: Scripts de arquivo único coexistem com scripts de pasta
A CLI SHALL continuar carregando scripts de arquivo único existentes em `scripts/` sem alteração quando scripts em pasta também estiverem presentes.

#### Scenario: Arquivo único e pasta coexistem sem conflito
- **WHEN** existem `scripts/simples.py` e `scripts/complexo/main.py` em `scripts/`
- **THEN** ambos são carregados no registry com suas respectivas `ScriptEntry`
