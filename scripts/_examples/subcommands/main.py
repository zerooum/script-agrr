"""Example: script with multiple subcommands, each with its own args."""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript


class SubcommandsExample(AgrrScript):
    name = "Subcommands Example"
    description = "Demonstrates scripts with multiple subcommands, each with its own args"
    group = "Examples"
    version = "1.0.0"

    subcommands = [
        {
            "name": "deploy",
            "description": "Deploy the application to an environment",
            "args": [
                {
                    "name": "environment",
                    "prompt": "Target environment:",
                    "type": "select",
                    "options": ["dev", "staging", "prod"],
                    "default": "dev",
                },
                {
                    "name": "version",
                    "prompt": "Version to deploy (e.g. 1.2.3):",
                    "type": "text",
                    "pattern": "alphanumeric",
                    "max_length": 20,
                },
            ],
        },
        {
            "name": "rollback",
            "description": "Roll back to the previous deployment",
            "args": [
                {
                    "name": "environment",
                    "prompt": "Target environment:",
                    "type": "select",
                    "options": ["dev", "staging", "prod"],
                },
                {
                    "name": "confirm",
                    "prompt": "Confirm rollback:",
                    "type": "select",
                    "options": ["yes", "no"],
                },
            ],
        },
        {
            "name": "status",
            "description": "Show deployment status",
        },
    ]

    def deploy(self, creds: dict, args: dict) -> None:
        env = args.get("environment", "")
        version = args.get("version", "")
        print(f"Deploying version '{version}' to '{env}'...")
        print(f"  environment : {env}")
        print(f"  version     : {version}")
        print("Deploy complete (example — no real action performed).")

    def rollback(self, creds: dict, args: dict) -> None:
        env = args.get("environment", "")
        confirm = args.get("confirm", "no")
        if confirm != "yes":
            print("Rollback cancelled.")
            return
        print(f"Rolling back '{env}'...")
        print("Rollback complete (example — no real action performed).")

    def status(self, creds: dict, args: dict) -> None:
        print("Deployment status:")
        print("  dev     : running v1.2.3")
        print("  staging : running v1.2.2")
        print("  prod    : running v1.2.1")


if __name__ == "__main__":
    SubcommandsExample.main()
