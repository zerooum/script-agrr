# credential-management Specification

## Purpose
TBD - created by archiving change agrr-cli. Update Purpose after archive.
## Requirements
### Requirement: Credentials stored in OS Keychain
The CLI SHALL store user credentials using the operating system's native keychain:
- **macOS**: Keychain Access
- **Windows**: Windows Credential Manager
- **Linux**: libsecret / Secret Service API

Credentials are stored per key name (matching `requires_auth` entries), scoped to the `agrr` service namespace.

#### Scenario: Credential stored successfully
- **WHEN** the user provides a credential and confirms saving it
- **THEN** the CLI stores it in the OS Keychain under `agrr/<key_name>`
- **THEN** subsequent executions of scripts using that key do not prompt the user

#### Scenario: OS Keychain unavailable (headless/CI)
- **WHEN** the OS Keychain is not accessible (e.g., headless Linux without libsecret)
- **THEN** the CLI falls back to an AES-256 encrypted file at `~/.config/agrr/credentials.enc`
- **THEN** the CLI prompts for a master password once per session to unlock the file

### Requirement: CLI prompts for missing credentials before execution
When a script declares `requires_auth` and one or more keys are not in the keychain, the CLI SHALL prompt the user for each missing credential before spawning the script process.

#### Scenario: Credential not found, user provides it
- **WHEN** a script requires `["DB_USER", "DB_PASS"]` and `DB_PASS` is not in the keychain
- **THEN** the CLI prompts: "DB_PASS para '<script name>': " (input masked for passwords)
- **THEN** after entry, the CLI asks: "Salvar para próximas execuções? [s/N]"
- **WHEN** user confirms
- **THEN** the credential is stored in the keychain

#### Scenario: User declines to save credential
- **WHEN** user provides a credential but declines to save
- **THEN** the credential is used only for the current execution and not persisted

### Requirement: Auth error triggers credential deletion
When a script exits with code 99 (AUTH_ERROR), the CLI SHALL delete ALL credentials declared in that script's `requires_auth` from the keychain and prompt the user to re-enter them.

This prevents credential-based account lockouts from repeated failed authentication attempts.

#### Scenario: Script exits 99, credentials deleted and re-prompted
- **WHEN** a script exits with code 99
- **THEN** the CLI deletes all keys from `requires_auth` in the keychain
- **THEN** the CLI displays: "Credenciais inválidas. As credenciais salvas foram removidas."
- **THEN** the CLI offers to re-run the script: "Deseja tentar novamente com novas credenciais? [S/n]"
- **WHEN** user confirms
- **THEN** the CLI re-prompts for all credentials and executes the script again

### Requirement: Credentials injected as environment variables
The CLI SHALL inject stored or just-entered credentials as environment variables into the script subprocess, using the pattern `AGRR_CRED_<UPPERCASE_KEY>`.

#### Scenario: Credentials injected before script execution
- **WHEN** a script with `requires_auth: ["DB_USER", "DB_PASS"]` is executed
- **THEN** the subprocess environment contains `AGRR_CRED_DB_USER` and `AGRR_CRED_DB_PASS`
- **THEN** no other process can read these variables (they are not exported to the parent shell)

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

