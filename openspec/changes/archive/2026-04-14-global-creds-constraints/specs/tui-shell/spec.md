## ADDED Requirements

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
