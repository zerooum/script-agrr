## Context

A descoberta atual de scripts (`discovery.rs`) varre apenas arquivos individuais em `scripts/`. Subdiretórios são ignorados por completo. Scripts com múltiplos arquivos (helpers, módulos, assets) não têm como ser representados — autores são forçados a criar scripts monolíticos ou a inlinar lógica auxiliar.

O `collect_candidates` atual faz `read_dir` e filtra apenas itens `is_file()` com extensão suportada (`.py`, `.js`, `.mjs`) ou binários executáveis sem extensão. Para habilitar pastas, a função precisa ser estendida para reconhecer subdiretórios como candidatos, resolver o arquivo `main` dentro deles e passá-lo adiante para `build_command` e `fetch_meta`.

## Goals / Non-Goals

**Goals:**
- Reconhecer subdiretórios de `scripts/` como candidatos de script
- Usar o arquivo `main` (com extensão suportada) como ponto de entrada para `--agrr-meta` e `--agrr-run`
- Emitir warning descritivo quando uma pasta não contiver arquivo `main` válido
- Manter retrocompatibilidade total com scripts de arquivo único existentes
- Expor o nome da pasta (não o path do `main`) nos warnings e no `ScriptEntry`

**Non-Goals:**
- Descoberta recursiva além de um nível de profundidade (pastas dentro de pastas)
- Arquivo de manifest separado dentro da pasta (o contrato `--agrr-meta` permanece inalterado)
- Alterações no protocolo, SDKs ou schema JSON do manifest

## Decisions

### D1 — Convenção de arquivo de entrada: `main.*`

**Decisão**: O arquivo de entrada em uma pasta deve se chamar `main` com extensão suportada: `main.py`, `main.js`, `main.mjs`, ou `main` (binário sem extensão). A busca segue essa ordem de prioridade.

**Alternativas consideradas**:
- `index.*` (convenção JS) — rejeitado por ser específico de ecossistema e inconsistente com Python/Rust
- Arquivo de `agrr.toml` ou `.agrr` com campo `entrypoint` — rejeitado por adicionar complexidade de configuração sem ganho claro

**Rationale**: `main` é a convenção mais neutra entre linguagens (Python, Rust, binários) e é intuitivo para autores de scripts.

### D2 — Espaço de busca: um nível de profundidade

**Decisão**: Somente subdiretórios diretos de `scripts/` são tratados como candidatos. Subpastas internas (ex.: `scripts/meu-script/utils/`) são ignoradas.

**Rationale**: Mantém a lógica de descoberta simples e O(1) em relação a profundidade. Scripts complexos com estrutura interna profunda podem usar imports relativos sem restrição, pois a descoberta não precisa conhecê-los.

### D3 — Path no `ScriptEntry`: apontar para o `main`, identificador pelo nome da pasta

**Decisão**: `ScriptEntry.path` aponta para o arquivo `main.*` (para que `build_command` e `executor` não precisem mudar). O `filename` em `LoadWarning` usa o nome da pasta para que o warning seja legível.

**Alternativas consideradas**:
- `ScriptEntry.path` aponta para a pasta — exigiria mudanças em `build_command`, `executor` e `fetch_meta`.

## Risks / Trade-offs

- **[Risco] Pasta `scripts/examples/` é um subdiretório**: a varredura atual ignora subdiretórios; após a mudança, `scripts/examples/` seria interpretado como candidato de script e falharia (sem `main`), gerando warning. 
  → **Mitigação**: Filtrar pastas cujo nome começa com `_` ou `.` como convenção de exclusão; alternativamente, tratar `examples/` como exceção explícita. Decisão mais simples: documentar que pastas dentro de `scripts/` com nome iniciado por `_` são ignoradas (ex: `_examples/`), e renomear ou mover `scripts/examples/` conforme necessário.

- **[Risco] `hello_world_rs/` dentro de `scripts/examples/`**: atualmente dentro de uma pasta não varrida; após suporte a pastas, ficaria exposto se `examples/` for varrida. Sem `main` bem definido, geraria warning. 
  → **Mitigação**: tratado pela mitigação acima.

- **[Trade-off] Ordem de busca do `main`**: buscar por `main.py` antes de `main.js` cria uma precedência implícita. Scripts com ambos os arquivos (incomum) pegariam sempre o primeiro encontrado.
  → Aceitável; esta situação é um erro de autoria.

## Migration Plan

Nenhuma migração necessária. Scripts de arquivo único existentes continuam funcionando sem alteração. A mudança é aditiva e retrocompatível.

Se `scripts/examples/` estiver causando warning indesejado após o deploy, renomeá-la para `scripts/_examples/` resolve o problema.
