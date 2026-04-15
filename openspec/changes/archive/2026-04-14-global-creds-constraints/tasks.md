## 1. Define Global Credential Constraints

- [x] 1.1 Add a `GlobalCredConstraint` struct in `agrr/src/credentials.rs` with fields `max_length: u32` and `pattern: Option<Pattern>` (reuse the `Pattern` type from `manifest.rs`)
- [x] 1.2 Add a `global_cred_constraint(key: &str) -> Option<GlobalCredConstraint>` function in `credentials.rs` that returns `max_length: 8` for `CHAVE` and `max_length: 8, pattern: Some(Pattern::Numeric)` for `SENHA`, and `None` for any other key

## 2. Extend CollectingCred Mode

- [x] 2.1 Add a `validation_error: Option<String>` field to the `CollectingCred` variant in `agrr/src/app.rs`
- [x] 2.2 Update all `Mode::CollectingCred { .. }` construction sites in `app.rs` and `main.rs` to include `validation_error: None`

## 3. Enforce Constraints in Credential Collection Flow

- [x] 3.1 In `main.rs::handle_collecting_cred`, for `KeyCode::Char(c)`: look up the constraint for `cred_key_str`, reject the keystroke and set `validation_error` if the char violates the `pattern` or would exceed `max_length`; clear `validation_error` otherwise
- [x] 3.2 In `main.rs::handle_collecting_cred`, for `KeyCode::Backspace`: clear `validation_error`

## 4. Enforce Constraints in Credential Manager Saving Flow

- [x] 4.1 In `main.rs::handle_cred_manager_saving`, for `KeyCode::Char(c)`: when `script_idx` is `None` (global creds), look up the constraint for the current `key`, reject the keystroke if it violates `pattern` or would exceed `max_length`

## 5. Update TUI Credential Prompt Rendering

- [x] 5.1 In `agrr/src/ui/prompts.rs::render_cred_prompt`, read the `validation_error` from `CollectingCred` mode and render it below the input (styled in red, matching the arg prompt error style)
- [x] 5.2 In `render_cred_prompt`, add a constraint hint line (e.g. `"max 8 chars"`, `"apenas dígitos"`) when the key has a known global constraint, displayed in muted style above the input

## 6. Tests

- [x] 6.1 Add unit tests in `agrr/src/credentials.rs` verifying that `global_cred_constraint("CHAVE")` returns `max_length: 8` and no pattern, and `global_cred_constraint("SENHA")` returns `max_length: 8` and `pattern: Numeric`
- [x] 6.2 Add unit tests in `agrr/tests/` or `agrr/src/main.rs` verifying that a char keystroke violating SENHA pattern (e.g. a letter) is rejected and `validation_error` is set during `CollectingCred` for the SENHA key
- [x] 6.3 Add unit tests verifying that inputting more than 8 characters into CHAVE or SENHA is rejected
