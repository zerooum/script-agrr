## 1. Resolução de scripts_dir na TUI

- [x] 1.1 Criar função `resolve_scripts_dir()` em `agrr/src/main.rs` que tenta `scripts/` relativo ao executável, com fallback para CWD
- [x] 1.2 Substituir `let scripts_dir = Path::new("scripts")` pela chamada a `resolve_scripts_dir()`
- [x] 1.3 Adicionar testes unitários para `resolve_scripts_dir()` cobrindo ambos os cenários (relativo ao exe e fallback CWD)

## 2. Estrutura base do build script

- [x] 2.1 Criar `tools/build.py` com argparse, validação de CWD (verifica existência de `scripts/` e `agrr/`), e limpeza da pasta `build/`
- [x] 2.2 Implementar lógica de discovery no build script: iterar `scripts/`, classificar candidatos por tipo (Python single-file, Python folder, JS single-file, JS folder, Rust folder, binário pré-compilado), ignorar pastas com prefixo `_`

## 3. Compilação da TUI

- [x] 3.1 Implementar no build script a etapa de `cargo build --release -p agrr` e cópia de `target/release/agrr` para `build/agrr`

## 4. Compilação de scripts Python

- [x] 4.1 Implementar compilação de scripts Python single-file via `pyinstaller --onefile --paths=sdk/python`, copiando resultado para `build/scripts/<stem>`
- [x] 4.2 Implementar compilação de scripts Python multi-file (pasta): criar venv temporário, instalar `requirements.txt` se existir, executar PyInstaller com `--paths=sdk/python`, copiar resultado para `build/scripts/<folder>/main`, destruir venv
- [x] 4.3 Garantir que PyInstaller resolve o SDK `agrr_sdk` via `--paths` e que imports locais (helpers de mesmo diretório) são detectados automaticamente

## 5. Compilação de scripts Node.js

- [x] 5.1 Implementar compilação de scripts JS single-file via `pkg <file> --output build/scripts/<stem>`
- [x] 5.2 Implementar compilação de scripts JS multi-file (pasta): executar `npm install` se `package.json` existir, executar `pkg main.js --output build/scripts/<folder>/main`
- [x] 5.3 Garantir que `pkg` resolve o `require()` relativo ao SDK JS — copiar SDK para pasta temporária ou usar configuração de assets do pkg

## 6. Compilação de scripts Rust

- [x] 6.1 Implementar compilação de scripts Rust: detectar pastas com `Cargo.toml`, executar `cargo build --release` dentro da pasta, copiar `target/release/main` para `build/scripts/<folder>/main`

## 7. Cópia de binários pré-compilados

- [x] 7.1 Implementar cópia direta de binários pré-compilados (arquivo `main` sem extensão em subpasta) para `build/scripts/<folder>/main`

## 8. Resumo e controle de erros

- [x] 8.1 Implementar contadores de sucesso/falha e impressão de resumo ao final do build
- [x] 8.2 Implementar exit code: 0 se tudo ok, 1 se algum script falhou
- [x] 8.3 Garantir que falha em um script não interrompe a compilação dos demais (fail-soft)

## 9. Teste de integração

- [x] 9.1 Rodar build completo, verificar que `build/agrr` e todos os `build/scripts/*` existem e são executáveis
- [x] 9.2 Verificar que cada binário em `build/scripts/` responde corretamente a `--agrr-meta` (mesmo JSON que o script original)
- [x] 9.3 Rodar `build/agrr` e verificar que a TUI descobre os scripts compilados em `build/scripts/`
