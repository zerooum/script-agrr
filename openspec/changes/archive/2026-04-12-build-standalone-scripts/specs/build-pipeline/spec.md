## ADDED Requirements

### Requirement: Build script compila a TUI e todos os scripts em uma pasta distribuível
O script `tools/build.py` SHALL produzir uma pasta `build/` contendo o binário da TUI (`agrr`) e uma subpasta `scripts/` com todos os scripts compilados em binários standalone. O script SHALL ser executado a partir da raiz do projeto.

#### Scenario: Build completo com sucesso
- **WHEN** o desenvolvedor executa `python tools/build.py` a partir da raiz do projeto
- **THEN** a pasta `build/` é criada contendo `agrr` e `scripts/`
- **THEN** todos os scripts válidos em `scripts/` são compilados para binários em `build/scripts/`

#### Scenario: Pasta build/ pré-existente é limpa
- **WHEN** a pasta `build/` já existe antes do build
- **THEN** o script SHALL remover `build/` e recriá-la do zero

#### Scenario: Build executado fora da raiz do projeto
- **WHEN** o script é executado de um diretório que não contém `scripts/` e `agrr/`
- **THEN** o script SHALL exibir um erro e encerrar com exit code 1

### Requirement: Build compila a TUI via cargo
O script SHALL compilar a TUI executando `cargo build --release -p agrr` e copiar o binário resultante para `build/agrr`.

#### Scenario: TUI compilada com sucesso
- **WHEN** `cargo build --release -p agrr` executa com sucesso
- **THEN** `target/release/agrr` é copiado para `build/agrr`

#### Scenario: Falha na compilação cargo
- **WHEN** `cargo build --release -p agrr` falha
- **THEN** o build SHALL exibir o erro e encerrar com exit code 1

### Requirement: Build compila scripts Python via PyInstaller
Para cada script Python candidato, o build script SHALL compilar usando `pyinstaller --onefile` com `--paths=sdk/python` para que o SDK `agrr_sdk` seja resolvido.

#### Scenario: Single-file Python sem dependências externas
- **WHEN** existe `scripts/hello_world.py` e não há `requirements.txt` associado
- **THEN** o build executa `pyinstaller --onefile --paths=sdk/python scripts/hello_world.py`
- **THEN** o binário resultante é copiado para `build/scripts/hello_world`

#### Scenario: Multi-file Python com dependências
- **WHEN** existe `scripts/deploy_prod/main.py` e `scripts/deploy_prod/requirements.txt`
- **THEN** o build cria um venv temporário
- **THEN** instala as dependências de `requirements.txt` no venv
- **THEN** executa PyInstaller dentro do venv com `--paths=sdk/python`
- **THEN** o binário resultante é copiado para `build/scripts/deploy_prod/main`
- **THEN** o venv temporário é destruído após a compilação

#### Scenario: Multi-file Python sem requirements.txt
- **WHEN** existe `scripts/hello_multi/main.py` sem `requirements.txt`
- **THEN** o build compila com PyInstaller sem criar venv (dependências locais e SDK são suficientes)
- **THEN** o binário resultante é copiado para `build/scripts/hello_multi/main`

### Requirement: Build compila scripts Node.js via pkg
Para cada script Node.js candidato, o build script SHALL compilar usando `pkg` (@yao-pkg/pkg).

#### Scenario: Single-file JS sem dependências externas
- **WHEN** existe `scripts/hello_world.js` e não há `package.json` associado
- **THEN** o build executa `pkg scripts/hello_world.js --output build/scripts/hello_world`
- **THEN** o binário resultante é gerado em `build/scripts/hello_world`

#### Scenario: Multi-file JS com package.json
- **WHEN** existe `scripts/status_check/main.js` e `scripts/status_check/package.json`
- **THEN** o build executa `npm install` dentro de `scripts/status_check/`
- **THEN** o build executa `pkg main.js --output build/scripts/status_check/main`

#### Scenario: Multi-file JS sem package.json
- **WHEN** existe `scripts/utils_check/main.js` sem `package.json`
- **THEN** o build compila com `pkg` diretamente sem `npm install`
- **THEN** o binário resultante é copiado para `build/scripts/utils_check/main`

### Requirement: Build compila scripts Rust via cargo
Para cada script Rust candidato (pasta com `Cargo.toml`), o build script SHALL compilar via `cargo build --release`.

#### Scenario: Script Rust compilado com sucesso
- **WHEN** existe `scripts/hello_world_rs/Cargo.toml` com `[[bin]] name = "main"`
- **THEN** o build executa `cargo build --release` dentro da pasta do script
- **THEN** o binário `target/release/main` é copiado para `build/scripts/hello_world_rs/main`

### Requirement: Build copia binários pré-compilados
Se um script candidato é um binário executável pré-compilado (arquivo `main` sem extensão dentro de uma pasta), ele SHALL ser copiado diretamente para `build/scripts/`.

#### Scenario: Binário pré-compilado copiado
- **WHEN** existe `scripts/my_tool/main` que é um binário executável
- **THEN** o build copia para `build/scripts/my_tool/main`

### Requirement: Build ignora pastas com prefixo underscore
Pastas dentro de `scripts/` cujo nome começa com `_` SHALL ser ignoradas pelo build, consistente com o discovery da TUI.

#### Scenario: Pasta _examples ignorada
- **WHEN** existe `scripts/_examples/` com scripts válidos dentro
- **THEN** nenhum script de `_examples` é compilado ou copiado para `build/`

### Requirement: Build imprime resumo ao final
Ao final da execução, o build script SHALL imprimir um resumo com o número de scripts compilados com sucesso e falhos.

#### Scenario: Resumo de build com sucesso parcial
- **WHEN** 4 scripts são compilados com sucesso e 1 falha
- **THEN** o build imprime um resumo indicando 4 sucessos e 1 falha
- **THEN** o build encerra com exit code 1 (indicando falha parcial)

#### Scenario: Resumo de build totalmente bem sucedido
- **WHEN** todos os scripts são compilados com sucesso
- **THEN** o build imprime um resumo indicando todos os sucessos
- **THEN** o build encerra com exit code 0
