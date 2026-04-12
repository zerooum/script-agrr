## Why

Scripts com múltiplos arquivos (helpers, módulos auxiliares, assets) não podem ser representados por um único arquivo no diretório `scripts/`, o que limita authors a scripts simples e monolíticos. A falta de suporte a pastas impede a organização modular e bloqueia casos de uso reais do time.

## What Changes

- Scripts podem agora residir em **pastas próprias** dentro de `scripts/` — cada pasta é tratada como um único script
- A convenção de entrada é o arquivo `main` (com extensão suportada: `main.py`, `main.js`, `main.mjs`, ou binário `main`) dentro da pasta
- A CLI valida a existência do arquivo `main` ao descobrir pastas; pastas sem `main` geram warning e são ignoradas
- A lógica de descoberta (`discovery.rs`) é estendida para iterar subpastas além de arquivos individuais
- `build_command` e `fetch_meta` passam a aceitar o arquivo `main` dentro da pasta como ponto de entrada

## Capabilities

### New Capabilities

- `folder-script-discovery`: Descoberta de scripts baseados em pasta — a varredura de `scripts/` reconhece subdiretórios como candidatos, localiza o arquivo `main` dentro deles, valida sua existência e o usa como ponto de entrada para `--agrr-meta` e `--agrr-run`

### Modified Capabilities

- `script-discovery`: A lógica de `collect_candidates` passa a incluir subdiretórios de `scripts/` como candidatos além de arquivos avulsos; o critério de candidatura para pastas é a presença de um arquivo `main` com extensão suportada

## Impact

- `agrr/src/discovery.rs`: `collect_candidates` estendida para entrar em subdiretórios; `build_command` estendida para resolver `main.*` dentro de pastas
- Scripts de arquivo único existentes continuam funcionando sem alteração (retrocompatibilidade total)
- Nenhuma mudança no protocolo (`--agrr-meta` / `--agrr-run`), no schema de manifest, nem nos SDKs
- `scripts/examples/hello_world_rs/` já segue o padrão de pasta (binário compilado) — passa a ser coberto nativamente
