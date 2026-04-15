## Context

Global credentials (`CHAVE` and `SENHA`) are hardcoded fields in the CLI — defined by `GLOBAL_KEYS` in `credentials.rs` and collected by the TUI before script-specific credentials when `global_auth: true`. Currently, these prompts are plain free-text inputs with no length or format enforcement.

The `arg-prompt-constraints` spec defines `max_length` and `pattern` for script-declared args, and the TUI already enforces them during arg collection. The global cred prompts share the same underlying prompt rendering path, but today no constraints are attached to the global fields.

## Goals / Non-Goals

**Goals:**
- Constrain `CHAVE` to a maximum of 8 characters
- Constrain `SENHA` to a maximum of 8 characters, digits only (`pattern: "numeric"`)
- Enforce these constraints inline in the TUI (reject out-of-range keystrokes, show inline error)

**Non-Goals:**
- Making global credential constraints configurable via manifest or config file
- Changing the stored format, encryption, or keychain layout of global credentials
- Applying constraints to script-specific auth fields (`requires_auth`)

## Decisions

### Decision: Hardcode constraints in the CLI, not in script manifests

Global credentials are a CLI concept — not script-declared. Scripts simply receive `AGRR_CRED_CHAVE` and `AGRR_CRED_SENHA` as env vars. Therefore, constraints for these fields must be defined inside the CLI itself.

**Alternative considered**: Allow scripts to declare constraints for global creds in their manifest. Rejected — it would create conflicting definitions across scripts and the CLI would need a reconciliation strategy.

**Implementation**: Define a `GlobalCredConstraint` struct (or equivalent constant map) in `credentials.rs` keyed on field name. The TUI prompt layer reads these constraints when rendering a global cred prompt, applying the same `max_length`/`pattern` enforcement already used for text args.

### Decision: Reuse existing TUI constraint enforcement path

The TUI already has keystroke filtering and inline error display for `max_length` and `pattern` constraints on text args (`tui-shell` spec). Global cred prompts will plug into the same enforcement logic by passing a constraint object alongside the field name.

**Alternative considered**: Duplicate the enforcement inline in the global cred prompt handler. Rejected — increases maintenance surface and drift risk.

## Risks / Trade-offs

- **Existing saved credentials exceeding new constraints**: A user who previously saved a CHAVE or SENHA longer than 8 chars (or SENHA with non-digit chars) will find the stored value rejected on re-entry after deletion. The credential deletion flow (exit 99 / `l` clear in cred manager) does not validate format — this is acceptable since the constraint only applies at input time.
- **Hardcoded constraints are not extensible**: If the constraint values need to change, a code change and release is required. This is intentional given the controlled nature of global credentials.
