"""Example: script using its own credentials via requires_auth."""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript, AgrrAuthError


class OwnCredsExample(AgrrScript):
    name = "Own Credentials Example"
    description = "Example script that requires its own credentials (API_KEY and TOKEN)"
    group = "Examples"
    version = "1.0.0"

    requires_auth = ["API_KEY", "TOKEN"]

    args = [
        {
            "name": "action",
            "prompt": "Action:",
            "type": "select",
            "options": ["list", "create", "delete"],
            "default": "list",
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        api_key = creds.get("API_KEY", "")
        token = creds.get("TOKEN", "")

        if not api_key or not token:
            raise AgrrAuthError()

        print("=== Own Credentials Example ===\n")
        print(f"  API_KEY: {api_key!r}")
        print(f"  TOKEN:   {token!r}")
        print(f"  action:  {args.get('action')!r}")
        print("\nCredentials received successfully. ✓")


if __name__ == "__main__":
    OwnCredsExample.main()
