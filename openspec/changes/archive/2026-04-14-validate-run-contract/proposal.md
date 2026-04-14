## Why

Scripts que não implementam a função `run` (Python) ou não passam `run` (JS) são carregados normalmente na TUI, pois `--agrr-meta` não valida a existência dessa função. O erro só aparece no momento da execução (`--agrr-run`), gerando uma experiência ruim. A validação deve ocorrer no startup para que scripts incompletos sejam rejeitados com warning, assim como já acontece com manifests inválidos.

## What Changes

- Os SDKs Python e JS passam a validar a existência da função `run` durante `--agrr-meta`, emitindo erro (exit != 0) caso não exista.
- O SDK Rust já garante isso em tempo de compilação via trait `AgrrScript` com método obrigatório, então não precisa de alteração.
- Scripts sem `run` serão rejeitados no discovery e aparecerão como warning na TUI sidebar.

## Capabilities

### New Capabilities

_(nenhuma — a mudança reforça o contrato existente)_

### Modified Capabilities

- `script-protocol`: Adiciona requisito de que `--agrr-meta` DEVE falhar (exit != 0) quando o script não implementa a função `run`.

## Impact

- **SDK Python** (`sdk/python/agrr_sdk/__init__.py`): Validação no `_build_meta` / `main()` que `run` foi sobrescrito na subclasse.
- **SDK JS** (`sdk/js/index.js`): Validação no path `--agrr-meta` que `run` é uma função.
- **Testes SDK** (`sdk/python/tests/`, `sdk/js/tests/`): Novos testes para o cenário de `run` ausente.
- **CLI Rust**: Nenhuma alteração necessária — o discovery já rejeita scripts com exit != 0 no `--agrr-meta`.
