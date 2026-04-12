## 1. Estender `collect_candidates` para suportar pastas

- [x] 1.1 Em `collect_candidates`, iterar entradas do diretório e identificar subdiretórios (não apenas arquivos)
- [x] 1.2 Ignorar subdiretórios cujo nome comece com `_` (underscore)
- [x] 1.3 Para cada subdiretório válido, buscar arquivo `main` com extensões suportadas na ordem: `main.py`, `main.js`, `main.mjs`, `main` (binário)
- [x] 1.4 Incluir o path do arquivo `main` encontrado na lista de candidatos
- [x] 1.5 Se nenhum `main` for encontrado na pasta, retornar um `Err` com mensagem descritiva de "arquivo main não encontrado"

## 2. Ajustar `LoadWarning` para usar nome da pasta

- [x] 2.1 Em `discover`, ao processar candidatos de pasta, usar o nome do diretório pai (não o nome do arquivo `main`) como `filename` no `LoadWarning`
- [x] 2.2 Garantir que erros de validação de manifest e runtime continuem exibindo o nome correto do script (pasta ou arquivo)

## 3. Testes unitários

- [x] 3.1 Adicionar teste: pasta com `main.py` é reconhecida como candidato e carregada com sucesso
- [x] 3.2 Adicionar teste: pasta com `main.js` é reconhecida como candidato
- [x] 3.3 Adicionar teste: pasta com binário `main` executável é reconhecida como candidato
- [x] 3.4 Adicionar teste: pasta sem `main` emite `LoadWarning` com nome da pasta
- [x] 3.5 Adicionar teste: pasta com nome `_examples` é ignorada (sem warning, sem candidato)
- [x] 3.6 Adicionar teste: subdiretório aninhado (`scripts/foo/bar/main.py`) não é tratado como candidato
- [x] 3.7 Adicionar teste: arquivo único e pasta coexistem sem conflito no registry

## 4. Validação e testes de integração

- [x] 4.1 Verificar que `cargo test --workspace` passa sem regressões
- [x] 4.2 Criar script exemplo em `scripts/hello_world_multi/` com `main.py` e módulo auxiliar `greetings.py`
- [x] 4.3 Renomear `scripts/examples/` → `scripts/_examples/` para silenciar warning indesejado
