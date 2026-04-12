## Context

`agrr` é um CLI TUI em Rust com ratatui. Toda a camada de renderização vive em `agrr/src/ui.rs` (1144 linhas, 20 funções), e o event loop vive em `agrr/src/main.rs` (457 linhas, sendo ~420 de um `match` de 12 braços em `run_app()`). À medida que novos estados são adicionados ao FSM, os dois arquivos crescem sem limite natural de tamanho.

A estrutura atual de módulos é plana:

```
agrr/src/
  app.rs, credentials.rs, discovery.rs, executor.rs,
  main.rs, manifest.rs, runtime.rs, ui.rs
```

## Goals / Non-Goals

**Goals:**
- Converter `ui.rs` em submodule `ui/` dividido por área semântica
- Quebrar o `match` monolítico de `run_app()` em funções privadas nomeadas no mesmo `main.rs`
- Zero mudança de comportamento ou API pública

**Non-Goals:**
- Mover lógica entre camadas (ex: levar event handling para `app.rs`)
- Refatorar `app.rs`, `credentials.rs` ou outros arquivos não listados
- Alterar o protocolo de scripts ou o FSM em si

## Decisions

### D1: ui/ como Rust submodule (não como crate separado)

`ui/mod.rs` expõe apenas `pub fn render()`. Os submodules internos usam visibilidade `pub(super)` (funções render_*) e `pub(crate)` para constantes de tema e `centered_rect()` que são compartilhados entre siblings.

**Alternativa considerada:** crate separado `agrr-ui`. Rejeitado — adiciona overhead de Cargo sem benefício real para esse tamanho de projeto.

### D2: Divisão de submodules por área semântica do TUI

```
ui/
  mod.rs       — pub fn render(), render_warnings()
  theme.rs     — palette TN_*, is_masked_field(), key(), desc()
  layout.rs    — centered_rect()
  menu.rs      — render_menu, render_script_list, render_detail_panel,
                  render_footer, render_search_input
  prompts.rs   — render_arg_prompt, render_cred_prompt, render_ask_save
  output.rs    — render_output, render_auth_error
  cred_mgr.rs  — render_cred_manager, cred_manager_*_detail (×2),
                  render_cred_manager_saving, render_cred_manager_clear_confirm
```

**Alternativa considerada:** dividir por Mode enum (um arquivo por mode). Rejeitado — `render_menu` e `render_script_list` são partes do mesmo screen; dividir por screen é mais intuitivo que por state machine.

### D3: Funções handle_* em main.rs, não em app.rs

O event loop permanece em `main.rs` mas quebrado em `fn handle_menu`, `fn handle_search`, etc. Não move para `app.rs` porque as handlers acessam `credentials::*` diretamente e mover criaria dependência nova `app → credentials` que vale discutir separadamente.

## Risks / Trade-offs

- [Visibilidade pub(super)] → Não há risco: nenhum código fora de `ui/` usa funções internas hoje.
- [Conflito de nomes em imports] → `use crate::ui::theme::*` pode conflitar com outros módulos. Mitigação: importações explicitas, não glob.
- [Revisão dos imports em cada submodule] → Cada novo arquivo precisa re-importar ratatui + app types. Não é risco, só trabalho mecânico.

## Migration Plan

1. Criar `ui/` directory, mover conteúdo de `ui.rs` para os submodules
2. Ajustar visibilidades (`pub(super)`, `pub(crate)`)
3. Deletar `ui.rs` — Rust resolve `mod ui;` para `ui/mod.rs` automaticamente
4. Extrair `handle_*` functions em `main.rs`
5. `cargo build --workspace` deve passar sem alterações em outros arquivos
6. `cargo test --workspace` valida zero regressão

Rollback: git revert — sem schema de banco, sem migração de dados.

## Open Questions

- Futuramente: vale mover o event loop inteiro para `app.rs::handle_key()` para centralizar o FSM? Fora do escopo agora, mas esta refatoração deixa o terreno preparado.
