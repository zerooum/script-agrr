## Why

Global credentials (`CHAVE` and `SENHA`) are currently collected as free-form text with no validation, allowing inputs of arbitrary length and format. This creates inconsistency with real-world systems where these fields have known constraints — e.g., a fixed-length PIN-style password that must be numeric.

## What Changes

- The `CHAVE` field (global credential) will enforce a **max_length of 8 characters**
- The `SENHA` field (global credential) will enforce a **max_length of 8 characters** and **pattern: numeric** (digits only)
- The TUI MUST reject out-of-constraint keystrokes and show inline feedback when collecting global credentials
- These constraints are hardcoded in the CLI (not script-declared), since global credentials are CLI-defined fields

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `credential-management`: Add constraint rules for global credential fields (`CHAVE`: max 8 chars; `SENHA`: max 8 chars, digits only)
- `tui-shell`: Credential collection prompts for `CHAVE` and `SENHA` must enforce `max_length` and `pattern` the same way text args do

## Impact

- `agrr/src/credentials.rs`: Define global credential field constraints (max_length, pattern)
- `agrr/src/ui/prompts.rs` (or equivalent): Apply constraints when rendering global credential input prompts
- No changes to script manifests, SDKs, or environment variable injection
