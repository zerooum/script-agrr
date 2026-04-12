## Context

O agrr é uma TUI que descobre e executa scripts externos seguindo o protocolo `--agrr-meta`/`--agrr-run`. Hoje, scripts Python e Node.js exigem que o runtime esteja instalado na máquina do usuário. O executor em `executor.rs` invoca o interpretador adequado baseado na extensão do arquivo. Scripts Rust já são compilados em binários nativos.

O build pipeline proposto transforma **todos** os scripts em binários nativos, produzindo uma pasta `build/` autocontida que pode ser distribuída sem dependência de runtimes. O protocolo não muda — binários compilados continuam respondendo a `--agrr-meta` e `--agrr-run` normalmente.

Atualmente `scripts_dir` é hardcoded como `Path::new("scripts")` relativo ao CWD em `main.rs`. Isso funciona em desenvolvimento mas não em distribuição.

## Goals / Non-Goals

**Goals:**
- Produzir `build/agrr` (TUI) + `build/scripts/` (binários standalone) a partir de um único comando
- Suportar scripts Python (PyInstaller), Node.js (pkg), Rust (cargo), e binários pré-compilados
- Isolar dependências de cada script durante o build (venv por script Python, npm install local por script JS)
- Manter compatibilidade total com modo desenvolvimento (CWD-based discovery)
- Manter o protocolo `--agrr-meta`/`--agrr-run` intacto — sem mudanças nos SDKs

**Non-Goals:**
- Cross-compilation (cada OS compila localmente no seu ambiente)
- Compilar os SDKs como binários separados (eles são embutidos nos scripts pelo compilador)
- Integração com CI (pode ser adicionada depois, o script de build funciona localmente)
- Compressão dos binários via UPX ou similar
- Hot-reload ou build incremental

## Decisions

### 1. Script Python (`tools/build.py`) como ferramenta de build

**Escolha:** Um script Python orquestrador em `tools/build.py`.

**Alternativas consideradas:**
- **Shell script**: Mais frágil para lógica condicional e tratamento de erros. Cross-platform (Windows) seria complexo.
- **Subcomando `agrr build`**: Acoplaria lógica de build à TUI. A TUI é para rodar scripts; buildar é preocupação do desenvolvedor.
- **Justfile/Makefile**: Menos expressivo para lógica por linguagem e venvs temporários.
- **Crate Rust separado**: Overengineering — o build é um orquestrador de subprocessos, não precisa de performance ou safety de Rust.

**Rationale:** Python é legível, expressivo para manipulação de paths e subprocessos, e já está disponível na máquina do desenvolvedor (é prerequisito para compilar scripts Python com PyInstaller).

### 2. PyInstaller para scripts Python

**Escolha:** `pyinstaller --onefile` com `--paths=sdk/python` para resolver o SDK.

**Alternativas consideradas:**
- **Nuitka**: Compila para C, binários menores, mas mais lento e incompatibilidades frequentes com libs complexas.
- **PyOxidizer**: Projeto com manutenção incerta.
- **cx_Freeze**: Menos usado, comunidade menor.

**Rationale:** PyInstaller é o mais maduro, amplamente testado com diversas dependências (requests, boto3, pandas), e suporta `--onefile` que gera um único executável.

**SDK resolution:** O `sys.path.insert(0, "sdk/python")` nos scripts é um hack de desenvolvimento. O PyInstaller precisa encontrar `agrr_sdk` durante a análise de imports. A flag `--paths=sdk/python` resolve isso sem exigir instalação do SDK como pacote.

### 3. `pkg` (@yao-pkg/pkg) para scripts Node.js

**Escolha:** `pkg` para compilar scripts JS em binários standalone.

**Alternativas consideradas:**
- **Node SEA (Single Executable Applications)**: Oficial mas exige bundling prévio com esbuild/webpack, adicionando um passo extra.
- **Bun build --compile**: Binários menores, mas requer Bun instalado e compatibilidade não é 100% com Node.
- **nexe**: Menos mantido que pkg.

**Rationale:** `pkg` resolve `require()` e `node_modules` automaticamente, requer um único comando, e é compatível com o ecossistema Node existente.

**SDK resolution:** O `require(path.join(__dirname, '..', 'sdk', 'js', 'index.js'))` é resolvido pelo `pkg` em build time. O build script precisa garantir que o path relativo ao SDK esteja acessível. Para folder scripts, copiar o SDK para dentro da pasta temporária de build, ou ajustar o `--config` do pkg com `assets`.

### 4. Venv isolado por script Python

**Escolha:** Criar um `venv` temporário para cada script Python durante o build, instalar deps, compilar, destruir o venv.

**Rationale:** Scripts podem ter deps conflitantes (ex: script A usa `requests==2.28`, script B usa `requests==2.31`). Venvs isolados evitam conflitos. O custo é tempo de build, não tempo de execução.

### 5. Resolução de `scripts_dir` relativa ao executável

**Escolha:** Tentar `scripts/` relativo ao diretório do executável; fallback para `scripts/` relativo ao CWD.

```
resolve_scripts_dir():
  1. exe_dir = parent of std::env::current_exe()
  2. if exe_dir/scripts/ exists → use it
  3. else → use Path::new("scripts")  (CWD, modo dev)
```

**Rationale:** Modo distribuição (build/) e modo desenvolvimento coexistem sem configuração. Sem flags, sem env vars, sem config files.

### 6. Estrutura da pasta `build/`

```
build/
├── agrr                            ← cargo build --release
└── scripts/
    ├── hello_world                  ← pyinstaller de hello_world.py
    ├── hello_world_js               ← pkg de hello_world.js
    ├── hello_world_multi/
    │   └── main                     ← pyinstaller de hello_world_multi/main.py
    └── hello_world_rs/
        └── main                     ← cargo build de hello_world_rs/
```

**Regras de naming:**
- Single-file: nome do arquivo sem extensão → `build/scripts/<stem>`
- Multi-file (pasta): nome da pasta preservado → `build/scripts/<folder>/main`
- A estrutura espelha `scripts/` mas com binários no lugar de fontes

### 7. Declaração de dependências por pasta

- **Python**: `requirements.txt` na pasta do script. Ausência = sem deps externas.
- **Node.js**: `package.json` na pasta do script. Ausência = sem deps externas.
- **Single-file scripts**: sem deps externas (não há pasta para colocar requirements). Se precisar de deps, converter para multi-file (criar pasta).

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| PyInstaller não detecta imports dinâmicos (`importlib`, plugins) | Binário compila mas falha ao rodar | Suportar `--hidden-import` via config; documentar limitação |
| `pkg` perde manutenção (@yao-pkg é fork) | Build JS quebra no futuro | Migrar pra Node SEA + esbuild como plano B |
| Binários Python pesados (~50-80MB cada) | Pasta `build/` fica grande com muitos scripts | Aceitável para uso interno de equipe; UPX pode ser adicionado depois |
| `requirements.txt` desatualizado em relação ao código | Build compila mas script falha em runtime | Responsabilidade do dev do script; documentar convenção |
| Path relativo ao SDK (`--paths=sdk/python`) exige rodar build a partir da raiz do projeto | Build falha se rodado de outro diretório | Build script detecta e valida o CWD no startup |
| `pkg` pode não resolver corretamente o require relativo ao SDK JS | Build JS gera binário que não encontra o SDK | Build script copia o SDK para a pasta temporária ou usa `pkg --config` para declarar assets |
| Campo `runtime` no manifest do binário compilado é irrelevante | Lixo semântico no manifest, mas inofensivo | A TUI ignora o campo `runtime` em binários sem extensão — o executor despacha pela extensão do arquivo. O módulo `runtime.rs` (pyenv/nvm) foi removido |
