"""Testa recebimento das credenciais próprias do script (requires_auth)."""

import sys
sys.path.insert(0, "sdk/python")

from agrr_sdk import AgrrScript, AgrrAuthError


class TestOwnCreds(AgrrScript):
    name = "Teste Credenciais Próprias"
    description = "Imprime as credenciais próprias do script recebidas via requires_auth"
    group = "testes"
    version = "1.0.0"
    runtime = {"language": "python", "min_version": "3.8"}

    requires_auth = ["USUARIO", "SENHA_OWN", "API_KEY", "TOKEN"]

    args = [
        {
            "name": "simular_falha",
            "prompt": "Simular falha de autenticação?",
            "options": ["nao", "sim"],
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        print("=== Teste de Credenciais Próprias ===\n")

        for chave in self.requires_auth:
            valor = creds.get(chave, "")
            if valor:
                print(f"  ✓ {chave} = {valor!r}")
            else:
                print(f"  ✗ {chave}: NÃO recebida ou vazia")

        if args.get("simular_falha") == "sim":
            print("\nSimulando falha de autenticação (exit 99)...")
            raise AgrrAuthError()

        print("\nCredenciais próprias recebidas com sucesso. ✓")


if __name__ == "__main__":
    TestOwnCreds.main()
