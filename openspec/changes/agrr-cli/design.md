## Context

O time não possui uma interface unificada para descobrir e executar scripts de automação. Scripts existem em repositórios pessoais, wikis e pastas locais — escritos em Python, JavaScript e Rust — sem contrato comum, sem gerenciamento de credenciais e sem mecanismo de descoberta. `agrr` é o host central que resolve isso: um binário Rust com TUI que descobre, valida e executa scripts externos via subprocess, mantendo cada script isolado em sua própria linguagem/runtime.

## Goals / Non-Goals

**Goals:**
- Binário único `agrr` multiplataforma (Linux, macOS, Windows) distribuído via build do repo
- Protocolo de subprocess bem definido permitindo scripts em qualquer linguagem
- SDKs por linguagem (Python, JS, Rust) que implementam o protocolo e reduzem boilerplate
- Descoberta automática no startup com validação e avisos
- Gerenciamento seguro de credenciais via OS Keychain com fluxo de auth error
- TUI híbrida: menu navegável + busca fuzzy

**Non-Goals:**
- Execução paralela / background de scripts (v1 é serial)
- Marketplace ou registro central de scripts
- Versionamento ou rollback de scripts
- Suporte a linguagens além de Python, JS e Rust na v1
- Suporte a pyenv/nvm para múltiplas versões (v1 usa PATH; será adicionado futuramente)

## Decisions

### 1. Protocolo por subprocess (não plugin system nativo)
**Decisão:** Scripts são processos externos invocados via `std::process::Command`. A CLI não carrega código de terceiros em-process.

**Alternativas consideradas:**
- WASM plugins: portável e isolado, mas exige compilar scripts para WASM — friction alto para o time
- FFI/dlopen: performance máxima, mas extremamente frágil e inseguro

**Rationale:** Subprocess é o único modelo verdadeiramente agnóstico de linguagem. Isolamento de processo é um benefício: crash no script não derruba a CLI.

---

### 2. Dois flags, dois modos: `--agrr-meta` e `--agrr-run`
**Decisão:** A CLI usa dois modos de invocação distintos para cada script:
- `--agrr-meta` → script retorna JSON de metadados via stdout + exit 0
- `--agrr-run` → script executa com credenciais e args via env vars

**Alternativas consideradas:**
- Arquivo sidecar YAML: simples, mas acoplamento temporal (manifest pode divergir do script)
- Config centralizado: um arquivo de registry no repo — qualquer PR tem que editar dois lugares

**Rationale:** Self-describing via `--agrr-meta` garante que manifest e código andam juntos. O SDK gera o JSON automaticamente a partir das declarações no próprio script.

---

### 3. Credenciais via variáveis de ambiente (não args CLI)
**Decisão:** Credenciais são injetadas como `AGRR_CRED_<NAME>=<value>` no env do subprocess.

**Alternativas consideradas:**
- Passar como args (`--password=xxx`): aparecem em `ps aux` e logs de processo — inseguro
- Stdin: requer que o script leia stdin antes de qualquer outra coisa — dificulta scripts simples

**Rationale:** Variáveis de ambiente não aparecem em listagens de processo, são suportadas universalmente, e os SDKs as extraem automaticamente.

---

### 4. Exit code 99 como sinal de AUTH_ERROR
**Decisão:** Exit code 99 é reservado para indicar credenciais incorretas. A CLI, ao receber 99, apaga a credencial do keychain e reprompta o usuário.

**Rationale:** Sem esse mecanismo, credenciais erradas ficam salvas e causam bloqueio de conta em serviços com rate limit de login. O contrato força scripts com autenticação a implementar essa sinalização — é verificado na validação do manifest (`requires_auth` presente → `auth_error_handling` em conformidade esperada).

---

### 5. Runtime resolution: pyenv/nvm primeiro, PATH como fallback
**Decisão:** 
1. Detectar pyenv/nvm no sistema
2. Se disponível: listar versões instaladas, selecionar a maior que satisfaz `>= min_version`
3. Fallback: busca hierárquica no PATH (`python3.11` → `python3` → `python`)
4. Binários Rust: campo `runtime` omitido → invocados diretamente sem verificação

**Rationale:** pyenv/nvm permitem múltiplas versões coexistentes sem conflito. PATH fallback garante que devs sem pyenv/nvm ainda conseguem rodar scripts compatíveis com o sistema.

---

### 6. SDKs no repo, não publicados
**Decisão:** `sdk/python`, `sdk/js`, `sdk/rust` vivem no mesmo repositório que `agrr`. Scripts referenciam localmente (`pip install -e ../../sdk/python`, `path = "../../sdk/rust"`).

**Rationale:** Time pequeno, sem overhead de publicação. Atualizações de protocolo e SDK chegam no mesmo PR — consistência garantida.

---

### 7. TUI: ratatui + crossterm com modo shell persistente
**Decisão:** `ratatui` para o layout, `crossterm` para eventos de teclado. A CLI inicia em modo TUI full-screen e retorna a ele após cada execução de script.

**Alternativas consideradas:**
- `blessed` via Node: não é Rust
- `cursive`: menos ativo que ratatui

**Rationale:** ratatui é o estado da arte em TUI Rust, tem suporte ativo e funciona em Windows/Linux/macOS via crossterm.

## Risks / Trade-offs

| Risco | Mitigação |
|-------|-----------|
| Script com `--agrr-meta` lento atrasa o startup | Timeout de 5s por script na fase de descoberta; scripts que excedem são marcados como inválidos com aviso |
| Keychain indisponível em alguns ambientes Linux (headless/CI) | Fallback para arquivo criptografado com AES-256 em `~/.config/agrr/credentials.enc`; master password promtada uma vez por sessão |
| Dev não implementa exit 99 corretamente | Validação estática no manifest: se `requires_auth` está preenchido, aviso na carga do script orientando implementação do exit 99 |
| Múltiplas versões do mesmo runtime (ex: python 3.11 e 3.12 coexistindo via pyenv) causam execuções inesperadas | A versão selecionada é logada no output da execução ("Running with python 3.11.6 via pyenv") |
| Windows não tem pyenv nativo | No Windows, apenas fallback PATH; pyenv-win pode ser suportado futuramente |
| Script com bug em `--agrr-meta` retorna JSON inválido | Erro capturado, script não carregado, aviso: "Script X não carregado. Motivo: manifest JSON inválido" |

## Open Questions

- Timeout de startup pode ser configurável por script ou é fixo global?
- O fallback de credenciais para arquivo criptografado deve ser opt-in (flag de configuração) ou automático?
- Devo suportar `AGRR_ARG_*` para args posicionais além de named args?
