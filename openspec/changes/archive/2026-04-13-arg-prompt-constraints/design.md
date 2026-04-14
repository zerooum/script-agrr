## Context

The agrr CLI collects arguments from users via a simple text-input or a single-select list (`options`) before invoking a script. Script authors have no way to express richer constraints — numeric-only input, character limits, multi-select, or default values. The TUI applies zero validation, so scripts receive raw strings and must validate internally.

The proposal introduces a `type` field on arg objects and several optional constraint fields. This design covers the manifest schema changes, TUI rendering, validation strategy, multiselect serialization, and SDK surface.

## Goals / Non-Goals

**Goals:**
- Let script authors declare the prompt type (`text`, `select`, `multiselect`) as a required field.
- Support optional constraints: `max_length`, `pattern`, `required`, `default`.
- Enforce constraints in the TUI before execution — scripts receive pre-validated input.
- Reject manifests missing `type` or violating `select`/`multiselect` rules (< 2 options) at load time.
- Update all three SDKs (Python, JS, Rust) to expose the new fields.
- Preserve backward compatibility for the env-var injection mechanism (`AGRR_ARG_*`).

**Non-Goals:**
- Custom regex patterns for `pattern` — only predefined classes (`numeric`, `alpha`, `alphanumeric`) in v1.
- Dependent/conditional args (arg B shown only if arg A = X).
- Async validation (calling a remote API to validate input).
- File-picker or date-picker prompt types.

## Decisions

### 1. `type` is a required field — strict validation at discovery

**Decision:** If a script declares `args` and any arg object lacks a `type` field, the manifest is invalid and the script is not loaded.

**Rationale:** Making `type` required from day one avoids ambiguity. The migration path is simple — add `"type": "text"` to existing free-text args or `"type": "select"` to args that have `options`.

**Alternatives considered:**
- Default `type` to `"text"` when missing → hides broken manifests; defers errors.
- Warn but still load → inconsistent TUI behavior for untyped args.

### 2. `options` is mandatory and ≥ 2 for `select` and `multiselect`

**Decision:** Manifest validation ensures `select` and `multiselect` args have an `options` array with at least 2 entries. `text` args MUST NOT have `options`.

**Rationale:** A single-option select is pointless; zero options is broken. Keeping `options` away from `text` avoids confusing hybrid prompts.

### 3. Multiselect values serialized as comma-separated in env var

**Decision:** For `multiselect`, the selected values are joined with `,` and injected as a single `AGRR_ARG_<NAME>` env var. Scripts split on `,` to get the list.

**Rationale:** Env vars are flat strings. Comma-separated is the simplest convention that works across Python, JS, and Rust without special parsing. SDKs can add a helper to split automatically.

**Alternatives considered:**
- JSON array in env var → adds parsing overhead and escaping complexity.
- Multiple env vars (`AGRR_ARG_<NAME>_0`, `_1`, …) → unpredictable number of vars, harder to consume.

### 4. `default` implies `required: false`

**Decision:** If `default` is set, the arg is treated as optional regardless of the `required` field. When the user submits blank input, the `default` value is used.

**Rationale:** A default value and a "must not be empty" constraint are contradictory. Explicitly resolving this avoids confusion.

### 5. `pattern` is an enum of predefined classes, not a regex

**Decision:** v1 supports `"numeric"`, `"alpha"`, `"alphanumeric"` only. The TUI filters keystrokes in real time (non-matching characters are rejected).

**Rationale:** Free-form regex raises security concerns (ReDoS) and complicates the TUI (regex errors are hard to message). Predefined classes cover the most common use cases.

### 6. Constraint fields only apply to `text` type

**Decision:** `max_length`, `pattern`, `required`, and `default` only apply to `type: "text"`. For `select`, the user must pick an option (no free text). For `multiselect`, at least one option must be selected (unless `required: false`).

**Rationale:** Select/multiselect inputs are already constrained by their option lists. Mixing text constraints onto select types is meaningless.

**Exception:** `required` also applies to `select` and `multiselect` — when `required: false`, the user may skip the prompt with an empty selection.

### 7. `default` on `select` pre-selects an option

**Decision:** For `select`, `default` must be one of the `options` values. The TUI cursor starts on that option. For `multiselect`, `default` is a comma-separated list of pre-selected options.

### 8. TUI inline validation feedback

**Decision:** When a constraint is violated (too long, wrong pattern, empty when required), the TUI shows a red error line below the input. The user cannot advance to the next arg until the constraint is satisfied.

**Rationale:** Blocking invalid input early prevents confusing script-side errors.

## Risks / Trade-offs

- **Breaking change** — Scripts with `args` that lack `type` will stop loading. → Mitigation: clear error message in the TUI sidebar naming the offending script and missing field. Document migration in release notes.
- **Multiselect comma collision** — Option values containing `,` would break the serialization. → Mitigation: manifest validation rejects options containing `,` for `multiselect` args.
- **Limited `pattern` set** — Users wanting e.g. email or IP validation won't be served in v1. → Mitigation: extensible enum; custom regex can be added in a future version.
- **SDK breaking change** — Rust SDK `ArgSpec` gains required `arg_type` field; existing code won't compile. → Mitigation: SDK version bump; migration guide.
