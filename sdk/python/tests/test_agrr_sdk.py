"""Tests for the agrr Python SDK dispatcher."""
import json
import os
import sys
import unittest
from io import StringIO
from unittest.mock import patch


# Ensure the SDK package on sys.path (works when run from sdk/python/)
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from agrr_sdk import AgrrAuthError, AgrrScript  # noqa: E402


# ─── Concrete test subclass ────────────────────────────────────────────────────

class GreetScript(AgrrScript):
    name = "Greet"
    description = "Greets someone by name"
    group = "demos"
    version = "1.0.0"
    requires_auth = ["API_KEY"]
    args = [{"name": "user_name", "prompt": "Name?", "type": "text"}]

    def run(self, creds, args):
        if creds.get("API_KEY") == "bad":
            raise AgrrAuthError()
        print(f"Hello, {args['user_name']}!")


class NoCredsScript(AgrrScript):
    name = "NoCredsScript"
    description = "Script without creds"
    group = "demos"
    version = "2.0.0"

    def run(self, creds, args):
        pass


# ─── Meta dispatch tests ───────────────────────────────────────────────────────

class TestMetaDispatch(unittest.TestCase):
    def test_meta_outputs_valid_json(self):
        """--agrr-meta should print a valid JSON manifest to stdout."""
        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit) as ctx:
                    GreetScript.main()
        self.assertEqual(ctx.exception.code, 0)
        data = json.loads(captured.getvalue().strip())
        self.assertEqual(data["name"], "Greet")
        self.assertEqual(data["version"], "1.0.0")
        self.assertEqual(data["group"], "demos")

    def test_meta_includes_requires_auth(self):
        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit):
                    GreetScript.main()
        data = json.loads(captured.getvalue().strip())
        self.assertIn("requires_auth", data)
        self.assertIn("API_KEY", data["requires_auth"])

    def test_meta_omits_empty_optional_fields(self):
        """Manifest should omit requires_auth and args when empty."""
        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit):
                    NoCredsScript.main()
        data = json.loads(captured.getvalue().strip())
        self.assertNotIn("requires_auth", data)
        self.assertNotIn("args", data)

    def test_meta_includes_runtime_when_set(self):
        class PythonScript(AgrrScript):
            name = "PythonScript"
            description = "needs python 3.11"
            group = "g"
            version = "1.0.0"
            runtime = {"language": "python", "min_version": "3.11"}

            def run(self, creds, args):
                pass

        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit):
                    PythonScript.main()
        data = json.loads(captured.getvalue().strip())
        self.assertEqual(data["runtime"]["language"], "python")
        self.assertEqual(data["runtime"]["min_version"], "3.11")


# ─── Run dispatch tests ────────────────────────────────────────────────────────

class TestRunDispatch(unittest.TestCase):
    def test_run_exits_0_on_success(self):
        env = {"AGRR_CRED_API_KEY": "good", "AGRR_ARG_USER_NAME": "Alice"}
        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with patch.dict(os.environ, env):
                with self.assertRaises(SystemExit) as ctx:
                    GreetScript.main()
        self.assertEqual(ctx.exception.code, 0)

    def test_run_exits_99_on_auth_error(self):
        env = {"AGRR_CRED_API_KEY": "bad", "AGRR_ARG_USER_NAME": "Bob"}
        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with patch.dict(os.environ, env):
                with self.assertRaises(SystemExit) as ctx:
                    GreetScript.main()
        self.assertEqual(ctx.exception.code, 99)

    def test_run_exits_1_on_unexpected_exception(self):
        class BrokenScript(AgrrScript):
            name = "Broken"
            description = "always fails"
            group = "g"
            version = "1.0.0"

            def run(self, creds, args):
                raise RuntimeError("something went wrong")

        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with self.assertRaises(SystemExit) as ctx:
                BrokenScript.main()
        self.assertEqual(ctx.exception.code, 1)

    def test_creds_injected_from_env(self):
        collected = {}

        class InspectCreds(AgrrScript):
            name = "Inspector"
            description = "inspects creds"
            group = "g"
            version = "1.0.0"
            requires_auth = ["DB_PASS"]

            def run(self, creds, args):
                collected.update(creds)

        env = {"AGRR_CRED_DB_PASS": "secret123"}
        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with patch.dict(os.environ, env):
                with self.assertRaises(SystemExit):
                    InspectCreds.main()
        self.assertEqual(collected.get("DB_PASS"), "secret123")

    def test_args_injected_from_env(self):
        collected = {}

        class InspectArgs(AgrrScript):
            name = "InspectArgs"
            description = "inspects args"
            group = "g"
            version = "1.0.0"
            args = [{"name": "target", "prompt": "Target?", "type": "text"}]

            def run(self, creds, args):
                collected.update(args)

        env = {"AGRR_ARG_TARGET": "staging"}
        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with patch.dict(os.environ, env):
                with self.assertRaises(SystemExit):
                    InspectArgs.main()
        self.assertEqual(collected.get("target"), "staging")


# ─── No-flag fallback ─────────────────────────────────────────────────────────

class TestNoFlagsFallback(unittest.TestCase):
    def test_no_flags_exits_1(self):
        with patch("sys.argv", ["script.py"]):
            with self.assertRaises(SystemExit) as ctx:
                GreetScript.main()
        self.assertEqual(ctx.exception.code, 1)


# ─── Arg constraint field tests ───────────────────────────────────────────────

class TestArgConstraintFields(unittest.TestCase):
    def _get_meta(self, script_cls):
        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit):
                    script_cls.main()
        return json.loads(captured.getvalue().strip())

    def test_meta_includes_arg_type(self):
        class S(AgrrScript):
            name = "S"; description = "d"; group = "g"; version = "1.0.0"
            args = [{"name": "x", "prompt": "X?", "type": "text"}]
            def run(self, creds, args): pass

        data = self._get_meta(S)
        self.assertEqual(data["args"][0]["type"], "text")

    def test_meta_includes_select_arg_with_options(self):
        class S(AgrrScript):
            name = "S"; description = "d"; group = "g"; version = "1.0.0"
            args = [{"name": "env", "prompt": "Env?", "type": "select", "options": ["prod", "staging"]}]
            def run(self, creds, args): pass

        data = self._get_meta(S)
        arg = data["args"][0]
        self.assertEqual(arg["type"], "select")
        self.assertEqual(arg["options"], ["prod", "staging"])

    def test_meta_includes_multiselect_arg(self):
        class S(AgrrScript):
            name = "S"; description = "d"; group = "g"; version = "1.0.0"
            args = [{"name": "tags", "prompt": "Tags?", "type": "multiselect", "options": ["a", "b", "c"]}]
            def run(self, creds, args): pass

        data = self._get_meta(S)
        self.assertEqual(data["args"][0]["type"], "multiselect")
        self.assertEqual(len(data["args"][0]["options"]), 3)

    def test_meta_includes_text_constraints(self):
        class S(AgrrScript):
            name = "S"; description = "d"; group = "g"; version = "1.0.0"
            args = [{"name": "code", "prompt": "Code?", "type": "text",
                     "max_length": 6, "pattern": "numeric",
                     "required": False, "default": "000"}]
            def run(self, creds, args): pass

        data = self._get_meta(S)
        arg = data["args"][0]
        self.assertEqual(arg["max_length"], 6)
        self.assertEqual(arg["pattern"], "numeric")
        self.assertFalse(arg["required"])
        self.assertEqual(arg["default"], "000")

    def test_multiselect_args_arrive_comma_separated(self):
        collected = {}

        class S(AgrrScript):
            name = "S"; description = "d"; group = "g"; version = "1.0.0"
            args = [{"name": "tags", "prompt": "Tags?", "type": "multiselect",
                     "options": ["alpha", "beta", "rc"]}]
            def run(self, creds, args): collected.update(args)

        env = {"AGRR_ARG_TAGS": "alpha,rc"}
        with patch("sys.argv", ["script.py", "--agrr-run"]):
            with patch.dict(os.environ, env):
                with self.assertRaises(SystemExit):
                    S.main()
        self.assertEqual(collected.get("tags"), "alpha,rc")
        self.assertEqual(collected["tags"].split(","), ["alpha", "rc"])


# ─── Run contract validation ───────────────────────────────────────────────────

class TestRunContractValidation(unittest.TestCase):
    def test_meta_exits_1_when_run_not_implemented(self):
        """--agrr-meta should exit 1 with an error message if run is not overridden."""
        class NoRunScript(AgrrScript):
            name = "NoRun"
            description = "missing run"
            group = "g"
            version = "1.0.0"
            # run() intentionally NOT implemented

        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            err = StringIO()
            with patch("sys.stderr", err):
                with self.assertRaises(SystemExit) as ctx:
                    NoRunScript.main()
        self.assertEqual(ctx.exception.code, 1)
        self.assertIn("'run' method not implemented", err.getvalue())

    def test_meta_succeeds_when_run_is_implemented(self):
        """--agrr-meta should exit 0 and emit valid JSON when run is properly overridden."""
        with patch("sys.argv", ["script.py", "--agrr-meta"]):
            captured = StringIO()
            with patch("sys.stdout", captured):
                with self.assertRaises(SystemExit) as ctx:
                    GreetScript.main()
        self.assertEqual(ctx.exception.code, 0)
        data = json.loads(captured.getvalue().strip())
        self.assertEqual(data["name"], "Greet")


if __name__ == "__main__":
    unittest.main()
