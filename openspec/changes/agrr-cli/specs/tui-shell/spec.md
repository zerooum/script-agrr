## ADDED Requirements

### Requirement: TUI renders scripts in grouped menu
The CLI SHALL open a full-screen TUI on startup that organizes loaded scripts by their `group` field into collapsible/labeled sections. Groups are sorted alphabetically; scripts within a group are sorted alphabetically by `name`.

#### Scenario: Scripts displayed grouped
- **WHEN** the TUI loads and the registry contains scripts with groups "infra" and "dados"
- **THEN** the menu shows two sections labeled with the group names
- **THEN** each script appears under its declared group

#### Scenario: No scripts loaded
- **WHEN** the registry is empty after discovery
- **THEN** the TUI displays "Nenhum script disponível neste momento."

### Requirement: TUI supports keyboard navigation
The user SHALL navigate the menu using keyboard shortcuts without a mouse.

Key bindings:
- `↑` / `↓` or `k` / `j`: move selection up/down
- `Enter`: execute selected script
- `/`: enter search/filter mode
- `Esc`: exit search mode / cancel
- `q` or `Ctrl+C`: quit the CLI

#### Scenario: User navigates with arrow keys
- **WHEN** the TUI is in normal mode and the user presses `↓`
- **THEN** the selection moves to the next script in the list

#### Scenario: User presses Enter on a script
- **WHEN** the user presses `Enter` with a script selected
- **THEN** the CLI transitions to execution mode for that script

### Requirement: TUI supports fuzzy search with /
Pressing `/` SHALL activate a filter input at the bottom of the TUI. As the user types, the menu SHALL update in real time to show only scripts whose `name`, `description`, or `group` match the query (fuzzy match).

#### Scenario: Fuzzy search filters the list
- **WHEN** the user presses `/` and types "dep"
- **THEN** the menu shows only scripts whose name/description/group fuzzy-match "dep" (e.g., "Deploy Produção")

#### Scenario: Search mode exited with Esc
- **WHEN** the user is in search mode and presses `Esc`
- **THEN** the search query is cleared and the full menu is restored

### Requirement: TUI displays startup validation warnings
Warnings produced during the discovery phase (invalid scripts, missing runtimes) SHALL be displayed in a dismissible panel before the main menu, or in a dedicated status area within the TUI.

#### Scenario: Warnings shown after discovery
- **WHEN** discovery produces one or more warnings
- **THEN** the TUI shows each warning as: "⚠ Script <filename> não carregado. Motivo: <reason>"
- **THEN** the user can press a key to dismiss the warnings and proceed to the menu

### Requirement: TUI shows execution output inline or returns to menu
When a script is executed, its stdout/stderr SHALL be streamed to the TUI output area. When the script exits, the user is returned to the main menu.

#### Scenario: Script output displayed
- **WHEN** a script is running
- **THEN** its stdout and stderr are shown in a scrollable output pane within the TUI

#### Scenario: Returned to menu after execution
- **WHEN** a script exits (any exit code, except 99 which triggers credential flow)
- **THEN** the TUI returns to the main menu
- **THEN** a brief status line shows the exit code and elapsed time
