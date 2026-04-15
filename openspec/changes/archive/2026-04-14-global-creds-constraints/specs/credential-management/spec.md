## ADDED Requirements

### Requirement: Global credential fields have fixed input constraints
The CLI SHALL define hardcoded input constraints for each global credential field:

- `CHAVE`: max_length = 8
- `SENHA`: max_length = 8, pattern = `"numeric"` (digits 0–9 only)

These constraints are defined in the CLI source and apply whenever the TUI collects a global credential value, regardless of which script triggered the collection.

#### Scenario: CHAVE capped at 8 characters
- **WHEN** the user is entering the `CHAVE` global credential
- **THEN** the TUI rejects any keystroke that would bring the input length beyond 8 characters

#### Scenario: SENHA accepts only digits
- **WHEN** the user is entering the `SENHA` global credential
- **THEN** the TUI rejects any keystroke that is not a digit (0–9)

#### Scenario: SENHA capped at 8 characters
- **WHEN** the user is entering the `SENHA` global credential and the input already contains 8 digits
- **THEN** additional keystrokes are rejected

#### Scenario: Valid CHAVE accepted
- **WHEN** the user enters a CHAVE value of 8 or fewer characters
- **THEN** the input is accepted and the TUI advances normally

#### Scenario: Valid SENHA accepted
- **WHEN** the user enters a SENHA value consisting of 1–8 digits
- **THEN** the input is accepted and the TUI advances normally
