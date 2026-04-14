"""Example: script using global shared credentials (CHAVE and SENHA)."""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript, AgrrAuthError


class GlobalCredsExample(AgrrScript):
    name = "Global Credentials Example"
    description = "Example script using global shared credentials (CHAVE and SENHA)"
    group = "Examples"
    version = "1.0.0"

    global_auth = True

    args = [
        {
            "name": "action",
            "prompt": "Action:",
            "type": "select",
            "options": ["list", "status", "sync"],
            "default": "list",
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        chave = creds.get("CHAVE", "")
        senha = creds.get("SENHA", "")

        if not chave or not senha:
            raise AgrrAuthError()

        print("=== Global Credentials Example ===\n")
        print(f"  CHAVE:  {chave!r}")
        print(f"  SENHA:  {senha!r}")
        print(f"  action: {args.get('action')!r}")
        print("\nGlobal credentials received successfully. ✓")


if __name__ == "__main__":
    GlobalCredsExample.main()
