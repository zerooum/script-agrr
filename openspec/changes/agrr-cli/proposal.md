## Why

Scripts de automação criados por diferentes membros do time residem em lugares distintos, foram escritos em linguagens variadas e não têm nenhuma interface unificada de descoberta ou execução. O resultado é que cada script exige documentação ad-hoc, onboarding manual e conhecimento tácito sobre onde e como rodar cada um. `agrr` resolve isso: uma única CLI interativa que agrega todos os scripts do time sob um menu navegável, com contratos formais de integração e gerenciamento seguro de credenciais.

## What Changes

- Criação do binário `agrr` (host em Rust) — CLI interativa com TUI navegável
- Definição do **Protocolo agrr**: convenção de invocação via subprocess (`--agrr-meta` / `--agrr-run`) com contrato JSON
- Criação dos **SDKs por linguagem** (`sdk/python`, `sdk/js`, `sdk/rust`) que implementam o protocolo e expõem abstrações por linguagem
- Sistema de **descoberta e validação** de scripts no startup: carrega apenas scripts com manifest válido e runtime disponível
- **Gerenciamento de credenciais** via OS Keychain com fluxo de prompt e remoção automática em caso de auth error (exit code 99)
- **Resolução de runtime**: detecta pyenv/nvm para versões específicas, com fallback para PATH; campo `runtime` omitido em binários Rust

## Capabilities

### New Capabilities

- `script-protocol`: Protocolo de comunicação entre a CLI e scripts externos via subprocess — define `--agrr-meta` (descoberta), `--agrr-run` (execução), schema JSON do manifest e exit codes especiais (0 = sucesso, 1 = erro genérico, 99 = auth error)
- `script-discovery`: Varredura da pasta `scripts/` no startup, invocação de `--agrr-meta` em cada candidato, validação do JSON retornado e construção do registry de scripts válidos com avisos para os inválidos
- `runtime-resolution`: Lógica de detecção de runtime por linguagem — usa pyenv/nvm quando disponível, cai para busca hierárquica no PATH; verifica versão mínima declarada no manifest
- `credential-management`: Armazenamento e recuperação de credenciais no OS Keychain; prompt interativo quando ausentes; remoção automática ao receber exit code 99; injeção via variáveis de ambiente `AGRR_CRED_*` na execução
- `tui-shell`: Interface TUI em modo shell híbrido — navegação por setas, agrupamento de scripts por `group`, busca fuzzy com `/`, execução com `Enter`, exibição de avisos de scripts não carregados

### Modified Capabilities

## Impact

- Novo repositório / projeto Rust (`agrr`) com Cargo workspace incluindo o binário host e o crate `agrr-script-sdk`
- Diretórios `sdk/python` e `sdk/js` com packages publicáveis localmente (pip install -e / npm link)
- Diretório `scripts/` como ponto de extensão — scripts adicionados via PR devem implementar o contrato do SDK
- Dependências externas: `ratatui`, `crossterm`, `keyring` (crate), `serde_json`, `which`
- Compatibilidade multiplataforma: Windows, Linux, macOS
