#!/usr/bin/env python3
"""
agrr build script — compiles the TUI and all scripts into a self-contained
build/ directory. No runtime (Python, Node.js, Rust toolchain) is needed on
the end-user's machine after distribution.

Usage (run from the project root):
    python tools/build.py
    python tools/build.py --skip-tui
    python tools/build.py --help

Output:
    build/
    ├── agrr                      ← compiled TUI binary
    └── scripts/
        ├── <name>                ← compiled single-file script
        ├── <name>/
        │   └── main              ← compiled folder script
        └── ...
"""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from enum import Enum, auto
from pathlib import Path
from typing import Optional

# ── Constants ─────────────────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
SDK_PYTHON = PROJECT_ROOT / "sdk" / "python"
SDK_JS = PROJECT_ROOT / "sdk" / "js"
BUILD_DIR = PROJECT_ROOT / "build"
BUILD_SCRIPTS_DIR = BUILD_DIR / "scripts"

IS_WINDOWS = platform.system() == "Windows"
TUI_BINARY_NAME = "agrr.exe" if IS_WINDOWS else "agrr"


# ── Script Types ──────────────────────────────────────────────────────────────

class ScriptType(Enum):
    PYTHON_SINGLE = auto()   # scripts/foo.py
    PYTHON_FOLDER = auto()   # scripts/foo/main.py
    JS_SINGLE = auto()       # scripts/foo.js
    JS_FOLDER = auto()       # scripts/foo/main.js  or  main.mjs
    RUST_FOLDER = auto()     # scripts/foo/Cargo.toml
    PREBUILT = auto()        # scripts/foo/main  (binary, no extension)


@dataclass
class ScriptCandidate:
    script_type: ScriptType
    # Path to the entry-point file (main.py, main.js, *.py, *.js, or binary)
    entry: Path
    # Human-readable label for log output
    display_name: str
    # Output path under build/scripts/ (file, not directory)
    output: Path


# ── Discovery ─────────────────────────────────────────────────────────────────

def collect_candidates(scripts_dir: Path) -> list[ScriptCandidate]:
    """Collect all compilable script candidates from scripts_dir."""
    candidates: list[ScriptCandidate] = []

    for item in sorted(scripts_dir.iterdir()):
        # Ignore hidden / underscore-prefixed items
        if item.name.startswith("_") or item.name.startswith("."):
            continue

        if item.is_file():
            ext = item.suffix.lower()
            if ext == ".py":
                candidates.append(ScriptCandidate(
                    script_type=ScriptType.PYTHON_SINGLE,
                    entry=item,
                    display_name=item.name,
                    output=BUILD_SCRIPTS_DIR / item.stem,
                ))
            elif ext in (".js", ".mjs"):
                candidates.append(ScriptCandidate(
                    script_type=ScriptType.JS_SINGLE,
                    entry=item,
                    display_name=item.name,
                    output=BUILD_SCRIPTS_DIR / item.stem,
                ))

        elif item.is_dir():
            _try_folder_candidate(item, candidates)

    return candidates


def _try_folder_candidate(folder: Path, candidates: list[ScriptCandidate]) -> None:
    """Detect and append a folder-based script candidate."""
    # Rust: presence of Cargo.toml
    if (folder / "Cargo.toml").is_file():
        candidates.append(ScriptCandidate(
            script_type=ScriptType.RUST_FOLDER,
            entry=folder / "Cargo.toml",
            display_name=folder.name,
            output=BUILD_SCRIPTS_DIR / folder.name / "main",
        ))
        return

    # Python folder
    main_py = folder / "main.py"
    if main_py.is_file():
        candidates.append(ScriptCandidate(
            script_type=ScriptType.PYTHON_FOLDER,
            entry=main_py,
            display_name=folder.name,
            output=BUILD_SCRIPTS_DIR / folder.name / "main",
        ))
        return

    # JS folder (main.js or main.mjs)
    for js_name in ("main.js", "main.mjs"):
        main_js = folder / js_name
        if main_js.is_file():
            candidates.append(ScriptCandidate(
                script_type=ScriptType.JS_FOLDER,
                entry=main_js,
                display_name=folder.name,
                output=BUILD_SCRIPTS_DIR / folder.name / "main",
            ))
            return

    # Pre-built binary: folder/main with no extension and executable bit
    main_bin = folder / "main"
    if main_bin.is_file() and (IS_WINDOWS or os.access(main_bin, os.X_OK)):
        candidates.append(ScriptCandidate(
            script_type=ScriptType.PREBUILT,
            entry=main_bin,
            display_name=folder.name,
            output=BUILD_SCRIPTS_DIR / folder.name / "main",
        ))


# ── Build steps ───────────────────────────────────────────────────────────────

@dataclass
class BuildResult:
    name: str
    success: bool
    error: str = ""


def build_tui() -> bool:
    """Compile the agrr TUI binary via cargo."""
    print("\n── Building TUI (cargo) ──────────────────────────────────────────")
    result = subprocess.run(
        ["cargo", "build", "--release", "-p", "agrr"],
        cwd=PROJECT_ROOT,
    )
    if result.returncode != 0:
        print("✗ cargo build failed", file=sys.stderr)
        return False

    src = PROJECT_ROOT / "target" / "release" / TUI_BINARY_NAME
    dst = BUILD_DIR / TUI_BINARY_NAME
    shutil.copy2(src, dst)
    if not IS_WINDOWS:
        dst.chmod(dst.stat().st_mode | 0o111)
    print(f"✓ build/{TUI_BINARY_NAME}")
    return True


def build_python_single(c: ScriptCandidate) -> BuildResult:
    c.output.parent.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="agrr-venv-") as venv_dir:
        venv_python = (
            Path(venv_dir) / "Scripts" / "python.exe"
            if IS_WINDOWS
            else Path(venv_dir) / "bin" / "python"
        )

        r = subprocess.run([sys.executable, "-m", "venv", venv_dir])
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "venv creation failed")

        r = subprocess.run(
            [str(venv_python), "-m", "pip", "install", "pyinstaller", "-q"]
        )
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "pyinstaller install failed")

        result = subprocess.run(
            [
                str(venv_python), "-m", "PyInstaller",
                "--onefile",
                f"--paths={SDK_PYTHON}",
                f"--name={c.output.name}",
                f"--distpath={c.output.parent}",
                "--workpath=/tmp/agrr-pyinstaller-work",
                "--specpath=/tmp/agrr-pyinstaller-spec",
                "--noconfirm",
                str(c.entry),
            ],
            cwd=PROJECT_ROOT,
        )
    if result.returncode != 0:
        return BuildResult(c.display_name, False, "PyInstaller failed")
    return BuildResult(c.display_name, True)


def build_python_folder(c: ScriptCandidate) -> BuildResult:
    folder = c.entry.parent
    requirements = folder / "requirements.txt"

    with tempfile.TemporaryDirectory(prefix="agrr-venv-") as venv_dir:
        venv_python = (
            Path(venv_dir) / "Scripts" / "python.exe"
            if IS_WINDOWS
            else Path(venv_dir) / "bin" / "python"
        )

        # Create venv
        r = subprocess.run([sys.executable, "-m", "venv", venv_dir])
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "venv creation failed")

        # Install requirements if present
        if requirements.is_file():
            r = subprocess.run(
                [str(venv_python), "-m", "pip", "install", "-r", str(requirements), "-q"]
            )
            if r.returncode != 0:
                return BuildResult(c.display_name, False, "pip install failed")

        # Install PyInstaller into the venv
        r = subprocess.run(
            [str(venv_python), "-m", "pip", "install", "pyinstaller", "-q"]
        )
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "pyinstaller install failed")

        c.output.parent.mkdir(parents=True, exist_ok=True)

        r = subprocess.run(
            [
                str(venv_python), "-m", "PyInstaller",
                "--onefile",
                f"--paths={SDK_PYTHON}",
                f"--paths={folder}",
                f"--name={c.output.name}",
                f"--distpath={c.output.parent}",
                "--workpath=/tmp/agrr-pyinstaller-work",
                "--specpath=/tmp/agrr-pyinstaller-spec",
                "--noconfirm",
                str(c.entry),
            ],
            cwd=folder,
        )
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "PyInstaller failed")

    return BuildResult(c.display_name, True)


def build_js_single(c: ScriptCandidate) -> BuildResult:
    """Compile a single-file JS script via pkg."""
    c.output.parent.mkdir(parents=True, exist_ok=True)

    # pkg needs to be able to resolve the SDK's relative require() path.
    # For single-file scripts the require uses a path relative to scripts/,
    # so we set the working directory to PROJECT_ROOT so the relative path
    # "../sdk/js/index.js" resolves correctly.
    pkg = _find_pkg()
    if not pkg:
        return BuildResult(c.display_name, False, "pkg not found (install with: npm install -g @yao-pkg/pkg)")

    result = subprocess.run(
        [pkg, str(c.entry), "--output", str(c.output)],
        cwd=PROJECT_ROOT,
    )
    if result.returncode != 0:
        return BuildResult(c.display_name, False, "pkg failed")
    return BuildResult(c.display_name, True)


def build_js_folder(c: ScriptCandidate) -> BuildResult:
    """Compile a folder-based JS script via pkg."""
    folder = c.entry.parent
    package_json = folder / "package.json"
    pkg = _find_pkg()
    if not pkg:
        return BuildResult(c.display_name, False, "pkg not found (install with: npm install -g @yao-pkg/pkg)")

    # npm install if package.json present
    if package_json.is_file():
        r = subprocess.run(["npm", "install", "--silent"], cwd=folder)
        if r.returncode != 0:
            return BuildResult(c.display_name, False, "npm install failed")

    # Copy SDK into a temporary location inside the folder so that the
    # relative require("../../sdk/js/...") resolves correctly when pkg
    # bundles from inside the folder directory.
    sdk_tmp = folder / "_agrr_sdk_js"
    if sdk_tmp.exists():
        shutil.rmtree(sdk_tmp)
    shutil.copytree(SDK_JS, sdk_tmp, ignore=shutil.ignore_patterns("node_modules", ".git"))

    try:
        c.output.parent.mkdir(parents=True, exist_ok=True)
        result = subprocess.run(
            [pkg, str(c.entry), "--output", str(c.output)],
            cwd=folder,
        )
    finally:
        shutil.rmtree(sdk_tmp, ignore_errors=True)

    if result.returncode != 0:
        return BuildResult(c.display_name, False, "pkg failed")
    return BuildResult(c.display_name, True)


def build_rust_folder(c: ScriptCandidate) -> BuildResult:
    """Compile a Rust folder script via cargo build --release."""
    folder = c.entry.parent
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=folder,
    )
    if result.returncode != 0:
        return BuildResult(c.display_name, False, "cargo build failed")

    # Find the release binary (bin name declared in Cargo.toml, defaulting to "main")
    release_dir = folder / "target" / "release"
    bin_name = "main.exe" if IS_WINDOWS else "main"
    src = release_dir / bin_name
    if not src.is_file():
        return BuildResult(c.display_name, False, f"binary not found at {src}")

    c.output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, c.output)
    if not IS_WINDOWS:
        c.output.chmod(c.output.stat().st_mode | 0o111)
    return BuildResult(c.display_name, True)


def copy_prebuilt(c: ScriptCandidate) -> BuildResult:
    """Copy a pre-built binary into build/scripts/."""
    c.output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(c.entry, c.output)
    if not IS_WINDOWS:
        c.output.chmod(c.output.stat().st_mode | 0o111)
    return BuildResult(c.display_name, True)


def build_script(c: ScriptCandidate) -> BuildResult:
    """Dispatch compilation to the appropriate handler."""
    dispatch = {
        ScriptType.PYTHON_SINGLE: build_python_single,
        ScriptType.PYTHON_FOLDER: build_python_folder,
        ScriptType.JS_SINGLE:     build_js_single,
        ScriptType.JS_FOLDER:     build_js_folder,
        ScriptType.RUST_FOLDER:   build_rust_folder,
        ScriptType.PREBUILT:      copy_prebuilt,
    }
    return dispatch[c.script_type](c)


# ── Helpers ───────────────────────────────────────────────────────────────────

def _find_pkg() -> Optional[str]:
    """Return the path to the pkg executable, or None if not found."""
    for candidate in ("pkg", "pkg.cmd"):
        found = shutil.which(candidate)
        if found:
            return found
    return None


def _check_prerequisites(skip_tui: bool) -> bool:
    """Warn about missing build-time tools. Returns True if all required tools
    are available."""
    ok = True

    if not skip_tui and not shutil.which("cargo"):
        print("✗ cargo not found — install Rust toolchain", file=sys.stderr)
        ok = False

    if not _find_pkg():
        print(
            "⚠  pkg not found — JS scripts will fail.\n"
            "   Install with: npm install -g @yao-pkg/pkg",
            file=sys.stderr,
        )

    if not shutil.which("npm"):
        print(
            "⚠  npm not found — JS folder scripts with package.json will fail.",
            file=sys.stderr,
        )

    return ok


def _validate_cwd() -> bool:
    """Ensure the script is run from the project root."""
    if not SCRIPTS_DIR.is_dir():
        print(
            f"✗ Expected scripts/ directory at {SCRIPTS_DIR}\n"
            "  Run this script from the project root.",
            file=sys.stderr,
        )
        return False
    if not (PROJECT_ROOT / "agrr").is_dir():
        print(
            f"✗ Expected agrr/ directory at {PROJECT_ROOT / 'agrr'}\n"
            "  Run this script from the project root.",
            file=sys.stderr,
        )
        return False
    return True


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build agrr TUI + scripts into a standalone build/ directory.",
    )
    parser.add_argument(
        "--skip-tui",
        action="store_true",
        help="Skip building the TUI binary (cargo build step).",
    )
    parser.add_argument(
        "--scripts-only",
        metavar="NAME",
        nargs="*",
        help="Build only the specified script names (omit to build all).",
    )
    args = parser.parse_args()

    if not _validate_cwd():
        return 1

    _check_prerequisites(args.skip_tui)

    # Clean and recreate build/
    if BUILD_DIR.exists():
        shutil.rmtree(BUILD_DIR)
    BUILD_DIR.mkdir()
    BUILD_SCRIPTS_DIR.mkdir()

    tui_ok = True
    if not args.skip_tui:
        tui_ok = build_tui()
        if not tui_ok:
            print("\n✗ TUI build failed — aborting.", file=sys.stderr)
            return 1

    # Collect and optionally filter candidates
    candidates = collect_candidates(SCRIPTS_DIR)
    if args.scripts_only is not None:
        filter_set = set(args.scripts_only)
        candidates = [c for c in candidates if c.display_name in filter_set]

    print(f"\n── Building {len(candidates)} script(s) ──────────────────────────────────────")

    results: list[BuildResult] = []
    for c in candidates:
        label = f"{c.script_type.name.lower().replace('_', ' ')} · {c.display_name}"
        print(f"  {label} ...", end="", flush=True)
        result = build_script(c)
        results.append(result)
        if result.success:
            print(f"\r  ✓ {label}                         ")
        else:
            print(f"\r  ✗ {label}: {result.error}        ")

    # Summary
    success_count = sum(1 for r in results if r.success)
    fail_count = len(results) - success_count

    print("\n── Build Summary ─────────────────────────────────────────────────")
    if not args.skip_tui:
        tui_status = "✓" if tui_ok else "✗"
        print(f"  {tui_status} agrr (TUI)")
    print(f"  Scripts: {success_count} succeeded, {fail_count} failed")

    if fail_count > 0:
        print("\nFailed scripts:")
        for r in results:
            if not r.success:
                print(f"  ✗ {r.name}: {r.error}")
        return 1

    print(f"\nDone! Distribution: {BUILD_DIR}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
