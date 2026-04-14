## ADDED Requirements

### Requirement: Arg declares a prompt type
Every arg object in a script manifest MUST include a `type` field with one of the following values: `"text"`, `"select"`, or `"multiselect"`.

#### Scenario: Arg with type text
- **WHEN** a script manifest contains an arg with `"type": "text"`
- **THEN** the TUI renders a free-text input prompt for that arg

#### Scenario: Arg with type select
- **WHEN** a script manifest contains an arg with `"type": "select"` and `"options": ["a", "b"]`
- **THEN** the TUI renders a single-selection list showing the options

#### Scenario: Arg with type multiselect
- **WHEN** a script manifest contains an arg with `"type": "multiselect"` and `"options": ["x", "y", "z"]`
- **THEN** the TUI renders a multi-selection checkbox list where the user can toggle one or more options

#### Scenario: Arg missing type field
- **WHEN** a script manifest contains an arg without a `type` field
- **THEN** the manifest is invalid and the script is not loaded
- **THEN** the TUI sidebar shows a warning: "arg at index N: `type` must be specified"

### Requirement: Select and multiselect require at least 2 options
An arg with `type` of `"select"` or `"multiselect"` MUST include an `options` array with at least 2 entries.

#### Scenario: Select with 2 options
- **WHEN** a script manifest contains an arg with `"type": "select"` and `"options": ["prod", "staging"]`
- **THEN** the manifest is valid for that arg

#### Scenario: Select with fewer than 2 options
- **WHEN** a script manifest contains an arg with `"type": "select"` and `"options": ["prod"]`
- **THEN** the manifest is invalid and the script is not loaded
- **THEN** the TUI sidebar shows a warning: "arg at index N: select/multiselect requires at least 2 options"

#### Scenario: Select with no options array
- **WHEN** a script manifest contains an arg with `"type": "select"` and no `options` field
- **THEN** the manifest is invalid and the script is not loaded

### Requirement: Text args may declare max_length
An arg with `type` `"text"` MAY include a `max_length` field (positive integer). The TUI MUST reject input exceeding this length.

#### Scenario: Input within max_length
- **WHEN** the user types 10 characters into a text arg with `"max_length": 20`
- **THEN** the input is accepted

#### Scenario: Input exceeding max_length
- **WHEN** the user types a 21st character into a text arg with `"max_length": 20`
- **THEN** the character is rejected and the TUI shows an inline error: "máximo de 20 caracteres"

### Requirement: Text args may declare a character pattern
An arg with `type` `"text"` MAY include a `pattern` field with one of: `"numeric"`, `"alpha"`, `"alphanumeric"`. The TUI MUST reject keystrokes that do not match the pattern.

#### Scenario: Numeric pattern rejects letters
- **WHEN** the user types a letter into a text arg with `"pattern": "numeric"`
- **THEN** the keystroke is rejected (character not added to input)

#### Scenario: Alpha pattern accepts letters
- **WHEN** the user types a letter into a text arg with `"pattern": "alpha"`
- **THEN** the keystroke is accepted

#### Scenario: Alphanumeric pattern accepts letters and digits
- **WHEN** the user types a letter or digit into a text arg with `"pattern": "alphanumeric"`
- **THEN** the keystroke is accepted

#### Scenario: Alphanumeric pattern rejects special characters
- **WHEN** the user types `@` into a text arg with `"pattern": "alphanumeric"`
- **THEN** the keystroke is rejected

### Requirement: Args have a required flag defaulting to true
Every arg MUST be treated as required by default. A script MAY set `"required": false` to allow empty input.

#### Scenario: Required arg rejects empty submit
- **WHEN** the user presses Enter on a text arg without typing anything and `required` is true (or omitted)
- **THEN** the TUI shows an inline error: "campo obrigatório" and does not advance

#### Scenario: Optional arg accepts empty submit
- **WHEN** the user presses Enter on a text arg without typing anything and `"required": false`
- **THEN** the arg value is set to an empty string and the TUI advances to the next arg

#### Scenario: Optional select accepts empty selection
- **WHEN** the user presses Esc or Enter without selecting on a select arg with `"required": false`
- **THEN** the arg value is set to an empty string and the TUI advances

### Requirement: Args may declare a default value
An arg MAY include a `default` field (string). When present, the arg is treated as optional. If the user submits empty input, the default value is used.

#### Scenario: Default value used on blank submit
- **WHEN** a text arg has `"default": "prod"` and the user presses Enter without typing
- **THEN** the arg value is set to `"prod"`

#### Scenario: Default shown in TUI prompt
- **WHEN** a text arg has `"default": "prod"`
- **THEN** the TUI displays the default value as a hint: "(padrão: prod)"

#### Scenario: Default on select pre-selects option
- **WHEN** a select arg has `"default": "staging"` and `"options": ["prod", "staging"]`
- **THEN** the TUI cursor starts on the "staging" option

#### Scenario: Default for select must be a valid option
- **WHEN** a select arg has `"default": "dev"` and `"options": ["prod", "staging"]`
- **THEN** the manifest is invalid and the script is not loaded

### Requirement: Multiselect values are comma-separated in env var
When a `multiselect` arg is submitted, the selected values MUST be joined with `,` and injected as a single `AGRR_ARG_<NAME>` env var.

#### Scenario: Multiple selections joined
- **WHEN** the user selects "x" and "z" from a multiselect arg named "targets"
- **THEN** the env var `AGRR_ARG_TARGETS` is set to `"x,z"`

#### Scenario: Multiselect options must not contain commas
- **WHEN** a multiselect arg has an option value containing `,`
- **THEN** the manifest is invalid and the script is not loaded
- **THEN** the TUI sidebar shows a warning: "arg at index N: multiselect options must not contain commas"

### Requirement: Text type must not have options
An arg with `type` `"text"` MUST NOT include an `options` field. If present, the manifest is invalid.

#### Scenario: Text arg with options
- **WHEN** a script manifest contains an arg with `"type": "text"` and `"options": ["a", "b"]`
- **THEN** the manifest is invalid and the script is not loaded
- **THEN** the TUI sidebar shows a warning: "arg at index N: text type must not have options"

### Requirement: Constraint fields only apply to compatible types
`max_length` and `pattern` MUST only appear on `type: "text"` args. If declared on `select` or `multiselect`, the manifest is invalid.

#### Scenario: max_length on select
- **WHEN** a script manifest contains an arg with `"type": "select"` and `"max_length": 10`
- **THEN** the manifest is invalid and the script is not loaded
