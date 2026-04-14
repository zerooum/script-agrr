## ADDED Requirements

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
