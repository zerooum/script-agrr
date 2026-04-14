"""Example: all argument types and constraints supported by agrr."""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", "sdk", "python"))

from agrr_sdk import AgrrScript


class ArgTypesExample(AgrrScript):
    name = "Arg Types Example"
    description = "Demonstrates text, select, and multiselect args with all supported constraints"
    group = "Examples"
    version = "1.0.0"

    args = [
        # --- text: basic ---
        {
            "name": "name",
            "prompt": "Your name:",
            "type": "text",
        },
        # --- text: max_length + default ---
        {
            "name": "short_code",
            "prompt": "Short code (max 8 chars, default: abc):",
            "type": "text",
            "max_length": 8,
            "default": "abc",
        },
        # --- text: numeric pattern ---
        {
            "name": "age",
            "prompt": "Age (numbers only, max 3 digits):",
            "type": "text",
            "pattern": "numeric",
            "max_length": 3,
        },
        # --- text: alpha pattern, optional ---
        {
            "name": "suffix",
            "prompt": "Suffix (letters only, optional):",
            "type": "text",
            "pattern": "alpha",
            "required": False,
        },
        # --- text: alphanumeric pattern ---
        {
            "name": "code",
            "prompt": "Alphanumeric code (max 8 chars):",
            "type": "text",
            "pattern": "alphanumeric",
            "max_length": 8,
        },
        # --- select: with default ---
        {
            "name": "environment",
            "prompt": "Environment:",
            "type": "select",
            "options": ["dev", "staging", "prod"],
            "default": "dev",
        },
        # --- select: no default ---
        {
            "name": "priority",
            "prompt": "Priority:",
            "type": "select",
            "options": ["low", "medium", "high", "critical"],
        },
        # --- multiselect: required ---
        {
            "name": "regions",
            "prompt": "Deploy regions:",
            "type": "multiselect",
            "options": ["us-east-1", "us-west-2", "eu-west-1", "ap-southeast-1"],
        },
        # --- multiselect: optional with default ---
        {
            "name": "channels",
            "prompt": "Notification channels (optional):",
            "type": "multiselect",
            "options": ["email", "slack", "pagerduty", "webhook"],
            "default": "email,slack",
            "required": False,
        },
    ]

    def run(self, creds: dict, args: dict) -> None:
        print("=== Arg Types Example ===\n")

        sections = [
            ("text", ["name", "short_code", "age", "suffix", "code"]),
            ("select", ["environment", "priority"]),
            ("multiselect", ["regions", "channels"]),
        ]

        for section, keys in sections:
            print(f"  [{section}]")
            for key in keys:
                print(f"    {key}: {args.get(key)!r}")
            print()

        print("All args received successfully. ✓")


if __name__ == "__main__":
    ArgTypesExample.main()
