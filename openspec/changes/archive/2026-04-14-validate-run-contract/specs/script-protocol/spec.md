## MODIFIED Requirements

### Requirement: SDKs implement the protocol per language
Each supported language SHALL have an SDK in the repository that implements the `--agrr-meta` / `--agrr-run` protocol, so script authors interact with a typed abstraction, not raw flags.

- `sdk/python`: abstract base class `AgrrScript` with `meta()` and `run(creds, args)` abstract methods; `AgrrAuthError` exception mapped to exit 99
- `sdk/js`: `createAgrrScript({ meta, run })` factory; `AgrrAuthError` class mapped to exit 99
- `sdk/rust`: `AgrrScript` trait with `meta()` and `run()` methods; `run_script()` entry point; `AuthError` return type mapped to exit 99

Each SDK MUST validate during `--agrr-meta` processing that the `run` implementation exists. If `run` is not implemented (Python) or not a function (JS), the SDK MUST print a descriptive error to stderr and exit with code 1. This ensures the CLI discovery phase rejects incomplete scripts.

#### Scenario: SDK handles --agrr-meta automatically
- **WHEN** a script using the SDK is invoked with `--agrr-meta`
- **THEN** the SDK serializes the declared metadata to JSON and prints it to stdout
- **THEN** the SDK exits with code 0 without calling the user's `run` implementation

#### Scenario: SDK maps AuthError to exit 99
- **WHEN** a script's `run` implementation raises/returns an `AuthError` (or equivalent)
- **THEN** the SDK exits with code 99

#### Scenario: Python SDK rejects script without run implementation
- **WHEN** a Python script subclasses `AgrrScript` but does not override `run()`
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: 'run' method not implemented" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: JS SDK rejects script without run function
- **WHEN** `createAgrrScript` is called without a `run` function (undefined, null, or non-function)
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK prints "agrr-sdk: 'run' function not provided" to stderr
- **THEN** the SDK exits with code 1

#### Scenario: Python SDK accepts valid subclass with run
- **WHEN** a Python script subclasses `AgrrScript` and implements `run()`
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK emits valid metadata and exits with code 0 (no change in behavior)

#### Scenario: JS SDK accepts valid script with run function
- **WHEN** `createAgrrScript` is called with a valid `run` function
- **WHEN** the script is invoked with `--agrr-meta`
- **THEN** the SDK emits valid metadata and exits with code 0 (no change in behavior)
