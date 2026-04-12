## ADDED Requirements

### Requirement: UI code is organized in a module with subfiles by semantic area
The system SHALL organize all TUI rendering code under `agrr/src/ui/` as a Rust submodule, replacing the single `ui.rs` file. The module SHALL expose only `pub fn render(frame, app)` as its public API.

#### Scenario: Module compiles and renders correctly
- **WHEN** `cargo build --workspace` is run after the restructure
- **THEN** the build succeeds with no errors and the TUI renders identically to before

#### Scenario: Submodule files group functions by rendering area
- **WHEN** a developer opens `agrr/src/ui/`
- **THEN** they find: `mod.rs`, `theme.rs`, `layout.rs`, `menu.rs`, `prompts.rs`, `output.rs`, `cred_mgr.rs`

#### Scenario: Internal functions are not publicly accessible outside ui/
- **WHEN** code outside `agrr/src/ui/` attempts to call a `render_*` function directly
- **THEN** the Rust compiler rejects it (visibility is `pub(super)` or lower)

### Requirement: Event handlers in main.rs are split into named functions
The system SHALL replace the monolithic `match` in `run_app()` with calls to private `handle_<mode>` functions, one per FSM mode, all within `main.rs`.

#### Scenario: run_app match arms delegate to named functions
- **WHEN** a developer reads `run_app()` in `main.rs`
- **THEN** each match arm contains a single function call (e.g., `handle_menu(&mut app, key)`)

#### Scenario: No behavioral change after refactor
- **WHEN** `cargo test --workspace` is run after the refactor
- **THEN** all existing tests pass without modification
