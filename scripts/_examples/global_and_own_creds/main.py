"""Example: script using both global credentials and its own credentials."""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript, AgrrAuthError


class GlobalAndOwnCredsExample(AgrrScript):
    name = "Global + Own Credentials Example"
    description = "Example script using global credentials (CHAVE/SENHA) plus its own (ORG_TOKEN)"
    group = "Examples"
    version = "1.0.0"

    global_auth = True
    requires_auth = ["ORG_TOKEN"]

    args = [
        {
            "name": "resource",
            "prompt": "Resource name:",
            "type": "text",
        },
        {
            "name": "action",
            "prompt": "Action:",
            "type": "select",
            "options": ["read", "write", "delete"],
            "default": "read",
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        chave = creds.get("CHAVE", "")
        senha = creds.get("SENHA", "")
        org_token = creds.get("ORG_TOKEN", "")

        if not chave or not senha or not org_token:
            raise AgrrAuthError()

        print("=== Global + Own Credentials Example ===\n")
        print("  Global credentials:")
        print(f"    CHAVE:     {chave!r}")
        print(f"    SENHA:     {senha!r}")
        print("  Own credentials:")
        print(f"    ORG_TOKEN: {org_token!r}")
        print("  Args:")
        print(f"    resource: {args.get('resource')!r}")
        print(f"    action:   {args.get('action')!r}")
        print("\nAll credentials received successfully. ✓")


if __name__ == "__main__":
    GlobalAndOwnCredsExample.main()
