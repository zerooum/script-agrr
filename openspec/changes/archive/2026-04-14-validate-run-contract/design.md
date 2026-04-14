## Context

Atualmente os SDKs Python e JS emitem metadata válida via `--agrr-meta` mesmo quando o script não implementa a função `run`. O Python SDK declara `run` como `@abstractmethod` em `AgrrScript`, mas a instância só é criada no path `--agrr-run` — o path `--agrr-meta` usa classmethods e nunca chama o construtor. O JS SDK recebe `run` como parâmetro de `createAgrrScript({ meta, run })`, mas não valida se `run` é uma função antes de emitir metadata. O Rust SDK não tem esse problema porque o trait `AgrrScript` exige implementação de `run()` em tempo de compilação.

O CLI (discovery.rs) já rejeita scripts cujo `--agrr-meta` retorna exit != 0 ou JSON inválido. Portanto, a correção mais simples e consistente é fazer os SDKs falharem cedo: validar a existência de `run` durante `--agrr-meta` e sair com exit != 0 se ausente.

## Goals / Non-Goals

**Goals:**
- Scripts Python sem `run` implementado DEVEM ser rejeitados no discovery (exit != 0 durante `--agrr-meta`)
- Scripts JS sem `run` passado como função DEVEM ser rejeitados no discovery (exit != 0 durante `--agrr-meta`)
- Mensagem de erro clara no stderr indicando que `run` não foi implementado
- Testes unitários cobrindo o cenário em ambos os SDKs

**Non-Goals:**
- Alterar o CLI Rust — o mecanismo de discovery com exit != 0 já trata o caso
- Alterar o SDK Rust — o compilador já garante a implementação de `run()`
- Validar conteúdo/assinatura de `run` (apenas existência)

## Decisions

### Python: Verificar se `run` foi sobrescrito na subclasse

**Decisão**: No método `main()`, antes de emitir metadata, verificar se `cls.run` é diferente de `AgrrScript.run` (ou se a classe ainda é abstrata). Se `run` não foi implementado, imprimir erro no stderr e sair com exit 1.

**Alternativa considerada**: Instanciar a classe durante `--agrr-meta` para que o ABC levante `TypeError`. Descartada porque instanciar pode ter efeitos colaterais e seria uma mudança de contrato mais agressiva.

**Implementação**: Usar `getattr(cls, 'run')` e verificar se é o método abstrato original ou se a classe tem `__abstractmethods__` contendo `'run'`.

### JS: Validar `typeof run === 'function'` no path `--agrr-meta`

**Decisão**: No início de `createAgrrScript`, antes de processar qualquer flag, verificar se `run` é uma função. Se não for, imprimir erro no stderr e sair com exit 1.

**Alternativa considerada**: Lançar exceção em vez de `process.exit(1)`. Descartada porque o script precisa comunicar falha ao CLI via exit code, não via exceção não tratada.

## Risks / Trade-offs

- **[Quebra de scripts existentes sem `run`]** → Se algum script em produção não implementa `run`, ele deixará de aparecer na TUI. Isso é o comportamento desejado — esses scripts já falhavam na execução.
- **[Falso positivo em herança multi-nível Python]** → Se uma classe intermediária implementar `run` e a subclasse herdar, a validação deve reconhecer isso. Mitigação: usar `inspect` ou checar `__abstractmethods__` no `cls`, não comparação direta de métodos.
