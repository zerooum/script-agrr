# tui-shell Specification

## Purpose
TBD - created by archiving change agrr-cli. Update Purpose after archive.
## Requirements
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
Warnings produced during the discovery phase (invalid scripts) SHALL be displayed in a dismissible panel before the main menu, or in a dedicated status area within the TUI.

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

### Requirement: TUI renders text arg with inline validation
When collecting a `text` arg that has constraints (`max_length`, `pattern`, `required`), the TUI MUST enforce them interactively and show inline error feedback.

#### Scenario: max_length enforced on keystroke
- **WHEN** the user is typing into a text arg with `max_length: 10` and the current input has 10 characters
- **THEN** additional keystrokes are rejected (character not appended)
- **THEN** the TUI shows "máximo de 10 caracteres" below the input

#### Scenario: pattern numeric rejects letter
- **WHEN** the user presses a letter key on a text arg with `pattern: "numeric"`
- **THEN** the keystroke is rejected (character not appended to input)

#### Scenario: Required text rejects empty submit
- **WHEN** the user presses Enter on an empty text input where `required` is true
- **THEN** the TUI shows "campo obrigatório" below the input and does not advance

#### Scenario: Default value hint shown
- **WHEN** a text arg has a `default` value
- **THEN** the TUI displays "(padrão: <value>)" next to the prompt

### Requirement: TUI renders select arg as single-choice list
When collecting a `select` arg, the TUI MUST render the options as a vertical list where the user navigates with `↑`/`↓` and confirms with `Enter`.

#### Scenario: Cursor navigation in select
- **WHEN** the user presses `↓` in a select prompt
- **THEN** the cursor moves to the next option

#### Scenario: Enter confirms selected option
- **WHEN** the user presses Enter on a highlighted option
- **THEN** the arg value is set to the selected option text
- **THEN** the TUI advances to the next arg or starts execution

#### Scenario: Default option pre-selected
- **WHEN** a select arg has `default: "staging"` and options include "staging"
- **THEN** the cursor starts on "staging"

### Requirement: TUI renders multiselect arg as checkbox list
When collecting a `multiselect` arg, the TUI MUST render the options as a checkbox list where `Space` toggles selection, `↑`/`↓` navigate, and `Enter` confirms the selection.

#### Scenario: Space toggles selection
- **WHEN** the user presses Space on a multiselect option
- **THEN** the option toggles between selected (☑) and unselected (☐)

#### Scenario: Enter confirms multiselect
- **WHEN** the user presses Enter with options "x" and "z" selected
- **THEN** the arg value is set to `"x,z"` (comma-separated)
- **THEN** the TUI advances to the next arg

#### Scenario: Required multiselect rejects zero selections
- **WHEN** the user presses Enter with no options selected and `required` is true
- **THEN** the TUI shows "selecione ao menos uma opção" and does not advance

#### Scenario: Optional multiselect allows zero selections
- **WHEN** the user presses Enter with no options selected and `required: false`
- **THEN** the arg value is set to an empty string and the TUI advances

### Requirement: Global credential prompts enforce hardcoded field constraints
When collecting a global credential value (`CHAVE` or `SENHA`), the TUI MUST apply the same keystroke filtering and inline error display that is used for constrained text args, using the CLI-defined constraint for that field.

#### Scenario: CHAVE prompt enforces max_length of 8
- **WHEN** the user is typing into the `CHAVE` global credential prompt and the input already has 8 characters
- **THEN** additional keystrokes are rejected (character not appended)
- **THEN** the TUI shows "máximo de 8 caracteres" below the input

#### Scenario: SENHA prompt rejects non-digit keystrokes
- **WHEN** the user presses a letter or symbol key in the `SENHA` global credential prompt
- **THEN** the keystroke is rejected (character not appended to input)

#### Scenario: SENHA prompt enforces max_length of 8
- **WHEN** the user is typing into the `SENHA` global credential prompt and the input already has 8 digits
- **THEN** additional keystrokes are rejected

#### Scenario: CHAVE prompt allows up to 8 characters normally
- **WHEN** the user types 8 or fewer characters into the `CHAVE` prompt
- **THEN** each keystroke is accepted and appended to the input

#### Scenario: SENHA prompt accepts digits normally
- **WHEN** the user types a digit into the `SENHA` prompt
- **THEN** the digit is accepted and appended to the input

