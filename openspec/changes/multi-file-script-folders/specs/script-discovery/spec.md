## MODIFIED Requirements

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
