## 1. Repositório e estrutura do projeto

- [x] 1.1 Inicializar Cargo workspace com crates: `agrr` (binário), `agrr-script-sdk` (lib Rust)
- [x] 1.2 Criar estrutura de diretórios: `sdk/python/`, `sdk/js/`, `scripts/`
- [x] 1.3 Adicionar dependências ao `Cargo.toml`: `ratatui`, `crossterm`, `keyring`, `serde`, `serde_json`, `which`, `tokio` (async para timeout)
- [x] 1.4 Criar `.gitignore`, `README.md` de alto nível e `scripts/.gitkeep`

## 2. Protocolo e SDKs

- [x] 2.1 Definir o schema JSON do manifest como struct Rust (`ScriptManifest`) com serde e validações
- [x] 2.2 Implementar crate `agrr-script-sdk`: trait `AgrrScript`, tipo `AuthError`, função `run_script()` que despacha `--agrr-meta` / `--agrr-run`
- [x] 2.3 Criar `sdk/python/agrr_sdk/__init__.py`: classe abstrata `AgrrScript`, exceção `AgrrAuthError`, entrypoint `main()` que despacha os flags e serializa manifest
- [x] 2.4 Criar `sdk/python/setup.py` (ou `pyproject.toml`) para instalação local com `pip install -e`
- [x] 2.5 Criar `sdk/js/index.js`: factory `createAgrrScript({ meta, run })`, classe `AgrrAuthError`, entrypoint que despacha flags e imprime manifest JSON
- [x] 2.6 Criar `sdk/js/package.json` e documentar uso com `npm link` ou caminho relativo

## 3. Runtime Resolution

- [x] 3.1 Implementar módulo `runtime` no host: detecção de pyenv via `which pyenv` e listagem de versões (`pyenv versions --bare`)
- [x] 3.2 Implementar seleção de versão pyenv: filtrar versões >= `min_version`, escolher a maior
- [x] 3.3 Implementar fallback PATH para Python: buscar `python{major}.{minor}` → `python3` → `python`, verificar versão com `--version`
- [x] 3.4 Implementar detecção de nvm via `$NVM_DIR` e listagem de versões instaladas
- [x] 3.5 Implementar seleção de versão nvm e fallback PATH para Node
- [x] 3.6 Implementar caminho "binário nativo" (Rust): sem verificação de runtime quando campo `runtime` é ausente no manifest
- [x] 3.7 Escrever testes unitários para a lógica de seleção de versão (sem exec real de processos)

## 4. Script Discovery

- [x] 4.1 Implementar scanner de `scripts/`: listar candidatos por extensão (`.py`, `.js`, `.mjs`) e executáveis sem extensão
- [x] 4.2 Implementar invocação de `--agrr-meta` com timeout de 5 segundos usando `tokio::time::timeout`
- [x] 4.3 Implementar parsing e validação do JSON retornado contra `ScriptManifest`
- [x] 4.4 Implementar verificação de runtime durante discovery (chamar módulo runtime, marcar inválido se não encontrado)
- [x] 4.5 Construir `ScriptRegistry`: vetor de scripts válidos + vetor de avisos de scripts inválidos
- [x] 4.6 Escrever testes de integração para discovery com scripts stub em cada linguagem

## 5. Credential Management

- [x] 5.1 Implementar módulo `credentials` usando crate `keyring`: funções `get`, `set`, `delete` com namespace `agrr`
- [x] 5.2 Implementar fallback para arquivo criptografado `~/.config/agrr/credentials.enc` quando keychain indisponível (AES-256-GCM via crate `aes-gcm`)
- [x] 5.3 Implementar fluxo de prompt para credenciais ausentes: input mascarado para senhas, pergunta de salvar [s/N]
- [x] 5.4 Implementar fluxo de auth error: ao receber exit 99, deletar todas as chaves do `requires_auth`, exibir mensagem, oferecer re-execução
- [x] 5.5 Implementar injeção de credenciais no subprocess via `AGRR_CRED_<KEY>` env vars

## 6. Execução de Scripts

- [x] 6.1 Implementar executor: monta o comando correto (resolvido pelo runtime module), injeta `AGRR_CRED_*` e `AGRR_ARG_*`, spawna subprocess
- [x] 6.2 Implementar streaming de stdout/stderr do subprocess para o TUI
- [x] 6.3 Implementar tratamento de exit codes: 0 (sucesso), 1 (erro genérico), 99 (auth error → credential flow)
- [x] 6.4 Logar runtime selecionado antes da execução: "Executando com Python 3.11.9 via pyenv"

## 7. Coleta de Args

- [x] 7.1 Implementar tela de coleta de args no TUI: para cada `arg` declarado no manifest, exibir `prompt` e opções (se `options` presente, mostrar seleção; caso contrário input livre)
- [x] 7.2 Injetar args coletados como `AGRR_ARG_<NAME>` no subprocess

## 8. TUI Shell

- [x] 8.1 Criar estrutura base do TUI com ratatui: layout de dois painéis (menu + output), inicialização de crossterm em modo raw
- [x] 8.2 Implementar renderização do menu agrupado: grupos em cabeçalhos, scripts listados abaixo, seleção destacada
- [x] 8.3 Implementar navegação por teclado: `↑`/`↓`/`k`/`j`, `Enter`, `q`/`Ctrl+C`
- [x] 8.4 Implementar modo de busca fuzzy com `/`: input na barra inferior, filtragem em tempo real por nome/descrição/grupo
- [x] 8.5 Implementar painel de avisos de discovery: exibir warnings antes do menu, dispensar com qualquer tecla
- [x] 8.6 Implementar área de output de execução: pane rolável com stdout/stderr em tempo real
- [x] 8.7 Implementar retorno ao menu após execução com status line (exit code + tempo decorrido)
- [x] 8.8 Garantir limpeza correta do terminal ao sair (restaurar modo normal mesmo em caso de panic)

## 9. Testes Unitários

- [x] 9.1 Testar validação do `ScriptManifest`: campos obrigatórios ausentes, tipos errados, campos extras inesperados e manifest vazio — lógica pura sem subprocess
- [x] 9.2 Testar o fluxo de auth error: dado `requires_auth: ["A", "B"]` e exit code 99, verificar que exatamente as chaves `A` e `B` são marcadas para deleção e que nenhuma outra chave é afetada — usando mock do keychain
- [x] 9.3 Testar o mapeamento de exit codes para ações: 0 → sucesso, 1 → erro genérico sem side-effects, 99 → trigger do fluxo de credential deletion — lógica pura sem subprocess
- [x] 9.4 Testar a construção dos nomes de variáveis de ambiente: `AGRR_CRED_<KEY>` e `AGRR_ARG_<NAME>` com chaves em diferentes cases (lowercase, mixed, com underscores) — crítico para garantir que credenciais são injetadas com o nome esperado pelo script
- [x] 9.5 Testar o dispatcher do SDK Python (`agrr_sdk`): `--agrr-meta` retorna JSON correto e sai com 0; `--agrr-run` chama `run()`; `AggrAuthError` resulta em exit 99; exceção genérica resulta em exit 1
- [x] 9.6 Testar o dispatcher do SDK JS (`agrr-sdk`): mesmos cenários de 9.5 para o factory `createAgrrScript`

## 10. Multiplataforma e Distribuição

- [x] 10.1 Configurar GitHub Actions (ou equivalente) com build matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`
- [x] 10.2 Verificar que `keyring` crate compila e funciona nos três sistemas (libsecret em Linux, Keychain em macOS, Credential Manager em Windows)
- [x] 10.3 Documentar pré-requisitos no README: instalação do binário, instalação dos SDKs locais, criação do primeiro script
- [x] 10.4 Criar script de exemplo em cada linguagem (Python, JS, Rust) em `scripts/examples/` demonstrando o uso dos SDKs
