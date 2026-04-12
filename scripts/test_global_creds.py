"""Testa recebimento das credenciais globais (CHAVE e SENHA)."""

import sys
sys.path.insert(0, "sdk/python")

from agrr_sdk import AgrrScript, AgrrAuthError


class TestGlobalCreds(AgrrScript):
    name = "Teste Credenciais Globais"
    description = "Imprime as credenciais globais (CHAVE e SENHA) recebidas via global_auth"
    group = "testes"
    version = "1.0.0"
    runtime = {"language": "python", "min_version": "3.8"}

    global_auth = True

    args = [
        {
            "name": "simular_falha",
            "prompt": "Simular falha de autenticação?",
            "options": ["nao", "sim"],
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        print("=== Teste de Credenciais Globais ===\n")

        for chave in ("CHAVE", "SENHA"):
            valor = creds.get(chave, "")
            if valor:
                print(f"  ✓ {chave} = {valor!r}")
            else:
                print(f"  ✗ {chave}: NÃO recebida ou vazia")

        if args.get("simular_falha") == "sim":
            print("\nSimulando falha de autenticação (exit 99)...")
            raise AgrrAuthError()

        print("\nCredenciais globais recebidas com sucesso. ✓")


if __name__ == "__main__":
    TestGlobalCreds.main()
