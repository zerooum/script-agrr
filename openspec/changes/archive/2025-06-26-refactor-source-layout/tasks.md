## 1. Preparação

- [x] 1.1 Criar diretório `agrr/src/ui/`
- [x] 1.2 Criar `agrr/src/ui/theme.rs` com palette TN_*, `is_masked_field()`, `key()`, `desc()`
- [x] 1.3 Criar `agrr/src/ui/layout.rs` com `pub(crate) fn centered_rect()`

## 2. Submodules de renderização

- [x] 2.1 Criar `agrr/src/ui/menu.rs` com `render_menu`, `render_script_list`, `render_detail_panel`, `render_footer`, `render_search_input`
- [x] 2.2 Criar `agrr/src/ui/prompts.rs` com `render_arg_prompt`, `render_cred_prompt`, `render_ask_save`
- [x] 2.3 Criar `agrr/src/ui/output.rs` com `render_output`, `render_auth_error`
- [x] 2.4 Criar `agrr/src/ui/cred_mgr.rs` com `render_cred_manager`, `cred_manager_global_detail`, `cred_manager_script_detail`, `render_cred_manager_saving`, `render_cred_manager_clear_confirm`

## 3. Módulo raiz e remoção do arquivo antigo

- [x] 3.1 Criar `agrr/src/ui/mod.rs` com `pub fn render()`, `fn render_warnings()` e `mod` declarations para cada submodule
- [x] 3.2 Deletar `agrr/src/ui.rs`
- [x] 3.3 Verificar que `cargo build --workspace` compila sem erros

## 4. Refatoração do event loop em main.rs

- [x] 4.1 Extrair `fn handle_menu(app, key) -> bool` de `run_app()` em `main.rs`
- [x] 4.2 Extrair `fn handle_search(app, key) -> bool` de `run_app()` em `main.rs`
- [x] 4.3 Extrair `fn handle_collecting_args(app, key)` de `run_app()` em `main.rs`
- [x] 4.4 Extrair `fn handle_collecting_cred(app, key)` de `run_app()` em `main.rs`
- [x] 4.5 Extrair `fn handle_ask_save_cred(app, key)` de `run_app()` em `main.rs`
- [x] 4.6 Extrair `fn handle_auth_error(app, key)` de `run_app()` em `main.rs`
- [x] 4.7 Extrair `fn handle_cred_manager(app, key) -> bool` de `run_app()` em `main.rs`
- [x] 4.8 Extrair `fn handle_cred_manager_saving(app, key)` de `run_app()` em `main.rs`
- [x] 4.9 Extrair `fn handle_cred_manager_clear(app, key)` de `run_app()` em `main.rs`
- [x] 4.10 Atualizar `run_app()` para delegar a cada `handle_*`

## 5. Validação

- [x] 5.1 `cargo build --workspace` sem erros ou warnings novos
- [x] 5.2 `cargo test --workspace` todos os testes passam (falha pré-existente em sdk/rust doctest não relacionada)
- [ ] 5.3 Smoke test manual: iniciar o TUI e verificar que menu, busca, execução e credential manager funcionam
