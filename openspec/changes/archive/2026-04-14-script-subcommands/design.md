## Context

Today each agrr script is a single unit: one manifest, one `run` entry point, one flat list of `args`. When a team has related operations (deploy / rollback / status), they must create separate script files, duplicating auth config and group metadata. The TUI menu grows linearly with every operation.

The proposal introduces an optional `subcommands` array in the manifest so a single script can expose multiple named operations, each with its own description and args.

## Goals / Non-Goals

**Goals:**
- Allow scripts to declare multiple subcommands in the manifest, each with `name`, optional `description`, and optional `args`.
- Add a TUI step to select a subcommand before collecting args when the script declares subcommands.
- Inject `AGRR_SUBCOMMAND=<name>` into the execution environment so the script knows which subcommand was selected.
- Update all three SDKs (Python, JS, Rust) with a subcommand dispatch API.
- Maintain full backward compatibility — scripts without `subcommands` behave exactly as today.

**Non-Goals:**
- Nested subcommands (subcommands within subcommands).
- Per-subcommand `requires_auth` — authentication stays at the script level.
- Per-subcommand `global_auth` — stays at the script level.
- Per-subcommand groups or versions — metadata stays at the script level.
- Changing the `--agrr-meta` / `--agrr-run` flag contract (still two flags).

## Decisions

### 1. Manifest shape: `subcommands` array at top level

The manifest gains an optional `subcommands` field:

```json
{
  "name": "Deploy Tools",
  "description": "Deploy, rollback, and check status",
  "group": "infra",
  "version": "1.0.0",
  "requires_auth": ["DEPLOY_TOKEN"],
  "subcommands": [
    {
      "name": "deploy",
      "description": "Deploy to the target environment",
      "args": [
        { "name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"] }
      ]
    },
    {
      "name": "rollback",
      "description": "Rollback to previous deployment",
      "args": []
    },
    {
      "name": "status",
      "args": [
        { "name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"] }
      ]
    }
  ]
}
```

**Rationale**: Placing subcommands at the same level as other manifest fields keeps the schema flat and easy to parse. Each subcommand reuses the existing `ArgSpec` format, avoiding new concepts.

**Alternative considered**: Wrapping everything in a `commands` map keyed by name. Rejected because ordered arrays match the TUI rendering order and are consistent with the existing `args` array pattern.

### 2. Mutual exclusivity: top-level `args` vs `subcommands`

A manifest MUST NOT declare both non-empty `args` and `subcommands`. The validator rejects this at discovery time.

**Rationale**: Mixing top-level args with subcommand-specific args creates ambiguity about when top-level args are collected (before or after subcommand selection?). Keeping them mutually exclusive avoids this complexity. Scripts that need "common args" can duplicate them across subcommands or handle them in their own code.

**Alternative considered**: Allow top-level `args` as "common args" collected before subcommand selection. Rejected for v1 — adds complexity to the FSM and arg collection flow. Can be revisited later.

### 3. Subcommand communication: `AGRR_SUBCOMMAND` env var

The CLI injects `AGRR_SUBCOMMAND=<selected_name>` when invoking `--agrr-run` on a script with subcommands. For scripts without subcommands, this env var is not set.

**Rationale**: Follows the existing pattern of `AGRR_CRED_*` and `AGRR_ARG_*` env vars. No flag changes needed.

### 4. TUI flow: subcommand selection uses `select`-style widget

After the user presses Enter on a script with subcommands, the TUI shows a single-choice list of subcommand names (with descriptions, if present). Once selected, the TUI proceeds to collect that subcommand's `args` (if any), then executes.

**Rationale**: Reuses the existing select arg rendering. The subcommand selector looks and behaves like a `select` arg but is not an arg — it's a navigation step.

### 5. Minimum subcommands: at least 2

A manifest with `subcommands` MUST declare at least 2 entries. A single subcommand would be pointless — just use the top-level args.

**Rationale**: Prevents degenerate cases and keeps the menu meaningful.

### 6. SDK dispatch: per-subcommand handlers

Each SDK adds a way to register named handlers:

- **Python**: `subcommands` class attribute mapping names to methods
  ```python
  class MyScript(AgrrScript):
      subcommands = {
          "deploy": "run_deploy",
          "rollback": "run_rollback",
      }
      def run_deploy(self, creds, args): ...
      def run_rollback(self, creds, args): ...
  ```
- **JS**: `subcommands` object in `createAgrrScript`
  ```js
  createAgrrScript({
    meta: { ..., subcommands: [...] },
    subcommands: {
      deploy: async ({ creds, args }) => { ... },
      rollback: async ({ creds, args }) => { ... },
    },
  });
  ```
- **Rust**: `run_subcommand(&self, name, creds, args)` method on the trait
  ```rust
  fn run_subcommand(&self, name: &str, creds: &Credentials, args: &Args) -> Result<(), ScriptError> { ... }
  ```

When `AGRR_SUBCOMMAND` is set, the SDK calls the matching handler. If no handler matches, the SDK prints an error to stderr and exits with code 1.

**Rationale**: Keeps the SDK API thin. Script authors register handlers per name; the SDK does the routing.

### 7. Subcommand name constraints

Subcommand names MUST be non-empty, unique within the manifest, and must not contain whitespace. They are case-sensitive.

**Rationale**: Names are used as env var values and keys in SDK dispatch maps. Whitespace would complicate parsing.

## Risks / Trade-offs

- **[Complexity in arg collection FSM]** → The `CollectingArgs` state needs to know which subcommand's args to use. Mitigation: resolve the arg list early (after subcommand selection) and pass it through the same existing flow.
- **[Discovery latency unchanged]** → Subcommands are declared in `--agrr-meta` output, no extra process invocations. No risk here.
- **[SDK backward compatibility]** → Existing scripts don't declare `subcommands`, so SDKs must keep the current `run()` path. Mitigation: `AGRR_SUBCOMMAND` env var is only checked when the script declares subcommands in its metadata.
- **[Menu density]** → Scripts with subcommands show as a single entry in the menu, trading discoverability for compactness. Mitigation: the subcommand selection step makes the available operations visible after selection. Subcommand descriptions help.
