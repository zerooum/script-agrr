## MODIFIED Requirements

### Requirement: TUI supports keyboard navigation
The user SHALL navigate the menu using keyboard shortcuts without a mouse.

Key bindings:
- `↑` / `↓` or `k` / `j`: move selection up/down
- `Enter`: execute selected script (or open subcommand selection if script declares subcommands)
- `/`: enter search/filter mode
- `Esc`: exit search mode / cancel subcommand selection / cancel
- `q` or `Ctrl+C`: quit the CLI
- `c`: open credential manager

#### Scenario: User navigates with arrow keys
- **WHEN** the TUI is in normal mode and the user presses `↓`
- **THEN** the selection moves to the next script in the list

#### Scenario: User presses Enter on a script without subcommands
- **WHEN** the user presses `Enter` with a script selected that does not declare subcommands
- **THEN** the CLI transitions to credential/arg collection or execution mode for that script

#### Scenario: User presses Enter on a script with subcommands
- **WHEN** the user presses `Enter` with a script selected that declares subcommands
- **THEN** the TUI transitions to the subcommand selection step showing the list of available subcommands

#### Scenario: Esc returns from subcommand selection to menu
- **WHEN** the user is in the subcommand selection step and presses `Esc`
- **THEN** the TUI returns to the main menu without executing
