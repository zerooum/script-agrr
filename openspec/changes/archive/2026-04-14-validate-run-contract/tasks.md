## 1. Python SDK — Validar `run` no `--agrr-meta`

- [x] 1.1 Em `sdk/python/agrr_sdk/__init__.py`, no método `main()`, antes do bloco `--agrr-meta`: verificar se `cls` ainda possui `'run'` em `__abstractmethods__`. Se sim, imprimir `"agrr-sdk: 'run' method not implemented"` em stderr e `sys.exit(1)`.
- [x] 1.2 Em `sdk/python/tests/test_agrr_sdk.py`, adicionar teste que cria subclasse sem `run` e verifica que `--agrr-meta` retorna exit code 1 com mensagem esperada no stderr.
- [x] 1.3 Em `sdk/python/tests/test_agrr_sdk.py`, adicionar teste que confirma que subclasse com `run` implementado continua retornando exit 0 e metadata válida no `--agrr-meta`.

## 2. JS SDK — Validar `run` no `--agrr-meta`

- [x] 2.1 Em `sdk/js/index.js`, na função `createAgrrScript`, antes de processar flags: verificar `typeof run !== 'function'`. Se verdadeiro, imprimir `"agrr-sdk: 'run' function not provided"` em stderr e `process.exit(1)`.
- [x] 2.2 Em `sdk/js/tests/agrr-sdk.test.js`, adicionar teste que chama `createAgrrScript` sem `run` e verifica exit code 1 com mensagem esperada no stderr.
- [x] 2.3 Em `sdk/js/tests/agrr-sdk.test.js`, adicionar teste que confirma que script com `run` válido continua funcionando normalmente no `--agrr-meta`.

## 3. Validação end-to-end

- [x] 3.1 Descomentar a função `run` em `scripts/hello_world.py` (restaurar o script ao estado funcional).
- [x] 3.2 Executar `cargo test --workspace`, `python3 -m unittest discover -s sdk/python/tests -v`, e `npm test --prefix sdk/js` — todos devem passar.
