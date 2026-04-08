#!/usr/bin/env python3
"""Multi-file hello world example for agrr.

Demonstrates that a script can live in a folder with helper modules.
The `greetings` module is imported relatively from the same directory.
"""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "sdk", "python"))

# Allow importing siblings from the same folder.
sys.path.insert(0, os.path.dirname(__file__))
from greetings import greet

from agrr_sdk import AgrrScript


class HelloWorldMulti(AgrrScript):
    name = "Hello World Multi"
    description = "Multi-file hello world — demonstrates folder-based scripts"
    group = "examples"
    version = "1.0.0"
    runtime = {"language": "python", "min_version": "3.8"}

    args = [
        {"name": "name", "prompt": "Name to greet"},
    ]

    def run(self, creds: dict, args: dict) -> None:
        name = args.get("name") or "World"
        print(greet(name))


if __name__ == "__main__":
    HelloWorldMulti.main()
