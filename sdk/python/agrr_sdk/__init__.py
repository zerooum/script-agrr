"""
agrr-sdk: Python SDK for building agrr-compatible scripts.

Usage
-----
from agrr_sdk import AgrrScript, AgrrAuthError

class MyScript(AgrrScript):
    name = "My Script"
    description = "Does something useful"
    group = "tools"
    version = "1.0.0"
    requires_auth = ["SERVICE_USER", "SERVICE_PASS"]
    args = [
        # Required fields: name, prompt, type ("text", "select", "multiselect")
        # For select/multiselect: options list with at least 2 entries
        {"name": "env", "prompt": "Environment?", "type": "select", "options": ["prod", "staging"]},
        # Optional constraint fields for text args:
        #   max_length (int), pattern ("numeric"|"alpha"|"alphanumeric"),
        #   required (bool, default True), default (str)
        {"name": "code", "prompt": "Code?", "type": "text", "pattern": "numeric", "max_length": 6},
        # Multiselect: user can pick one or more; values arrive comma-separated
        {"name": "tags", "prompt": "Tags?", "type": "multiselect", "options": ["alpha", "beta", "rc"]},
    ]
    # For interpreted scripts, declare runtime requirement:
    # runtime = {"language": "python", "min_version": "3.11"}

    def run(self, creds: dict, args: dict) -> None:
        env = args["env"]           # "prod" or "staging"
        code = args["code"]         # numeric string, max 6 chars
        tags = args["tags"].split(",") if args["tags"] else []
        # ... if login fails:
        # raise AgrrAuthError()

if __name__ == "__main__":
    MyScript.main()
"""

from __future__ import annotations

import json
import os
import sys
from abc import ABC, abstractmethod
from typing import Any


class AgrrAuthError(Exception):
    """Raise this to signal credential failure (exit code 99).

    The agrr CLI will delete stored credentials and re-prompt the user.
    """


class AgrrScript(ABC):
    """Base class for agrr-compatible Python scripts.

    Subclass this and implement :meth:`run`. Set class attributes to
    declare metadata.
    """

    #: Human-readable display name (required).
    name: str
    #: One-line description (required).
    description: str
    #: TUI menu group key, kebab-case (required).
    group: str
    #: Semver string (required).
    version: str
    #: Runtime requirement. Set to use a specific Python version.
    #: Example: {"language": "python", "min_version": "3.11"}
    runtime: dict[str, str] | None = None
    #: Named credential keys injected as AGRR_CRED_<KEY>.
    requires_auth: list[str] = []
    #: Argument specs: list of dicts with required keys ``name``, ``prompt``, ``type``
    #: (``"text"`` | ``"select"`` | ``"multiselect"``).
    #: ``select``/``multiselect`` also require ``options`` (list of ≥ 2 strings).
    #: Optional keys for ``text``: ``max_length`` (int), ``pattern``
    #: (``"numeric"`` | ``"alpha"`` | ``"alphanumeric"``), ``required`` (bool, default True),
    #: ``default`` (str). For ``multiselect``, values arrive as a comma-separated string.
    args: list[dict[str, Any]] = []
    #: If True, CHAVE and SENHA global credentials are injected as AGRR_CRED_CHAVE / AGRR_CRED_SENHA.
    global_auth: bool = False

    @abstractmethod
    def run(self, creds: dict[str, str], args: dict[str, str]) -> None:
        """Execute the script.

        Parameters
        ----------
        creds:
            Mapping of credential key → value as injected by the CLI.
        args:
            Mapping of arg name → value as collected by the CLI.

        Raises
        ------
        AgrrAuthError
            If the provided credentials are rejected by the remote service.
        """

    # ------------------------------------------------------------------ #
    # Internal helpers                                                     #
    # ------------------------------------------------------------------ #

    @classmethod
    def _build_meta(cls) -> dict[str, Any]:
        meta: dict[str, Any] = {
            "name": cls.name,
            "description": cls.description,
            "group": cls.group,
            "version": cls.version,
        }
        if cls.runtime:
            meta["runtime"] = cls.runtime
        if cls.requires_auth:
            meta["requires_auth"] = cls.requires_auth
        if cls.args:
            meta["args"] = cls.args
        if cls.global_auth:
            meta["global_auth"] = True
        return meta

    @classmethod
    def _collect_creds(cls) -> dict[str, str]:
        creds = {
            key: os.environ.get(f"AGRR_CRED_{key.upper()}", "")
            for key in cls.requires_auth
        }
        if cls.global_auth:
            for key in ("CHAVE", "SENHA"):
                creds[key] = os.environ.get(f"AGRR_CRED_{key}", "")
        return creds

    @classmethod
    def _collect_args(cls) -> dict[str, str]:
        return {
            arg["name"]: os.environ.get(f"AGRR_ARG_{arg['name'].upper()}", "")
            for arg in cls.args
        }

    @classmethod
    def main(cls) -> None:
        """Dispatch based on CLI flags. Call this from ``__main__``."""
        argv = sys.argv[1:]

        if "--agrr-meta" in argv:
            print(json.dumps(cls._build_meta()))
            sys.exit(0)

        if "--agrr-run" in argv:
            instance = cls()
            try:
                instance.run(cls._collect_creds(), cls._collect_args())
                sys.exit(0)
            except AgrrAuthError:
                sys.exit(99)
            except Exception as exc:  # noqa: BLE001
                print(f"Error: {exc}", file=sys.stderr)
                sys.exit(1)

        print("agrr-sdk: use --agrr-meta or --agrr-run", file=sys.stderr)
        sys.exit(1)
