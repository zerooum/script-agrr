## Why

`ui.rs` cresceu para 1144 linhas contendo 20 funções em 6 áreas semânticas distintas, dificultando navegação e manutenção. `main.rs` acumulou um event loop de 420 linhas com um `match` de 12 braços cobrindo cada estado do FSM. Ambos os arquivos precisam de reorganização para facilitar leitura e futuras contribuições.

## What Changes

- `agrr/src/ui.rs` é convertido em módulo `agrr/src/ui/` com submodules por área semântica: `theme`, `layout`, `menu`, `prompts`, `output`, `cred_mgr`, e `mod.rs` como dispatcher público
- `agrr/src/main.rs` tem seu event loop (`run_app`) refatorado: o `match` monolítico de 12 braços é quebrado em funções privadas nomeadas por modo — `handle_menu`, `handle_search`, `handle_collecting_args`, etc.
- Nenhuma mudança semântica ou de comportamento — refatoração puramente estrutural

## Capabilities

### New Capabilities

- `ui-module-layout`: Organização do código de renderização TUI em submodules por responsabilidade

### Modified Capabilities

<!-- Nenhuma mudança de requirements, apenas organização estrutural -->

## Impact

- `agrr/src/ui.rs` → `agrr/src/ui/mod.rs` + 6 submodules
- `agrr/src/main.rs`: `run_app()` e funções `handle_*` no mesmo arquivo (~120 linhas vs 457)
- O contrato externo permanece idêntico: `ui::render()` e `main::run_app()` não mudam de assinatura
- Todos os testes existentes devem continuar passando sem alterações
