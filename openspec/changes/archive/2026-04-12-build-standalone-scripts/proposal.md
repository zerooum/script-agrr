## Why

Os usuários do agrr precisam instalar runtimes (Python, Node.js) nas suas máquinas para rodar scripts interpretados. Isso gera atrito na adoção — especialmente quando há scripts com versões de runtime conflitantes ou quando a equipe que usa os scripts não tem familiaridade com gerenciadores de versão (pyenv, nvm). O objetivo é criar um pipeline de build que compile todos os scripts em binários standalone, produzindo uma pasta de distribuição pronta para uso sem nenhum runtime instalado.

## What Changes

- **Novo script de build (`tools/build.py`)** que compila a TUI e todos os scripts em uma pasta `build/` distribuível
- **Scripts Python** são compilados via PyInstaller (`--onefile`) com suporte a `requirements.txt` por pasta e venvs isolados por script
- **Scripts Node.js** são compilados via `pkg` (`@yao-pkg/pkg`) com suporte a `package.json` por pasta
- **Scripts Rust** são compilados via `cargo build --release`
- **Binários pré-compilados** são copiados direto
- **Convenção de declaração de dependências**: `requirements.txt` (Python) e `package.json` (JS) na pasta de cada script
- **Scripts single-file sem deps** são compilados diretamente; scripts com deps externas devem ser convertidos em pasta (multi-file)
- **Resolução do `scripts_dir`** na TUI passa a ser relativa ao executável, com fallback para CWD (manter compatibilidade com modo desenvolvimento)

## Capabilities

### New Capabilities
- `build-pipeline`: Define o fluxo de compilação que transforma scripts interpretados em binários standalone, incluindo a lógica por linguagem (PyInstaller, pkg, cargo), gerenciamento de dependências (venvs isolados, npm install), e a estrutura da pasta `build/` resultante

### Modified Capabilities
- `script-discovery`: O `scripts_dir` passa a ser resolvido relativo ao executável da TUI (com fallback para CWD), permitindo que o build distribuído funcione sem que o usuário rode a TUI a partir do diretório do projeto

## Impact

- **Novo arquivo**: `tools/build.py` — script de build orquestrando compilação por linguagem
- **Alteração**: `agrr/src/main.rs` — resolução de `scripts_dir` relativa ao executável
- **Dependências de build no PATH do desenvolvedor**: Python + pip + PyInstaller, Node.js + npm + pkg, Rust toolchain
- **Sem mudanças** em: protocolo `--agrr-meta`/`--agrr-run`, manifest, SDKs, credential management, executor, UI
- **Compatibilidade**: modo desenvolvimento (CWD-based) continua funcionando; o build é aditivo
