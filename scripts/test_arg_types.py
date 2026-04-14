"""Script de teste — exercita todos os tipos e restrições de arg."""

import sys
sys.path.insert(0, "sdk/python")

from agrr_sdk import AgrrScript


class TestArgTypes(AgrrScript):
    name = "Test Arg Types"
    description = "Testa text, select e multiselect com todas as restrições"
    group = "testes"
    version = "1.0.0"

    args = [
        # --- text simples ---
        {
            "name": "texto_livre",
            "prompt": "Digite qualquer texto:",
            "type": "text",
        },
        # --- text com max_length e default ---
        {
            "name": "apelido",
            "prompt": "Apelido (máx. 10 chars, padrão: dev):",
            "type": "text",
            "max_length": 10,
            "default": "dev",
        },
        # --- text com pattern numeric ---
        {
            "name": "idade",
            "prompt": "Idade (somente números):",
            "type": "text",
            "pattern": "numeric",
            "max_length": 3,
        },
        # --- text com pattern alpha e opcional ---
        {
            "name": "sufixo",
            "prompt": "Sufixo (letras, opcional):",
            "type": "text",
            "pattern": "alpha",
            "required": False,
        },
        # --- text com pattern alphanumeric ---
        {
            "name": "codigo",
            "prompt": "Código alfanumérico:",
            "type": "text",
            "pattern": "alphanumeric",
            "max_length": 8,
        },
        # --- select ---
        {
            "name": "ambiente",
            "prompt": "Ambiente de execução:",
            "type": "select",
            "options": ["dev", "staging", "prod"],
            "default": "dev",
        },
        # --- select sem default ---
        {
            "name": "prioridade",
            "prompt": "Prioridade:",
            "type": "select",
            "options": ["baixa", "media", "alta", "critica"],
        },
        # --- multiselect ---
        {
            "name": "regioes",
            "prompt": "Regiões a implantar:",
            "type": "multiselect",
            "options": ["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"],
        },
        # --- multiselect com default e opcional ---
        {
            "name": "notificacoes",
            "prompt": "Canais de notificação (opcional):",
            "type": "multiselect",
            "options": ["email", "slack", "pagerduty", "webhook"],
            "default": "email,slack",
            "required": False,
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        sep = "─" * 44

        print("╔══════════════════════════════════════════╗")
        print("║       RESULTADO — Test Arg Types         ║")
        print("╚══════════════════════════════════════════╝")
        print()

        sections = [
            ("text simples",       ["texto_livre"]),
            ("text c/ restrições", ["apelido", "idade", "sufixo", "codigo"]),
            ("select",             ["ambiente", "prioridade"]),
            ("multiselect",        ["regioes", "notificacoes"]),
        ]

        for title, keys in sections:
            print(f"  [{title}]")
            for key in keys:
                val = args.get(key)
                display = repr(val) if val is not None else "(vazio)"
                print(f"    {key:<18} = {display}")
            print()

        print(sep)
        print("  Todos os tipos de input recebidos com sucesso. ✓")


if __name__ == "__main__":
    TestArgTypes.main()
