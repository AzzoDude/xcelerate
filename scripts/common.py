"""Shared helpers for the xcelerate build scripts.

Consolidates the paths, subprocess and profile-loading logic that used to be
duplicated across every ``scripts/*.py`` file.
"""

from __future__ import annotations

import keyword
import json
import os
import re
import shutil
import subprocess
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(SCRIPT_DIR)

PROFILES_DIR = os.path.join(ROOT, "adapters", "profiles")
DISCOVERED_DIR = os.path.join(ROOT, "adapters", "discovered")
RUNTIME_SRC = os.path.join(ROOT, "adapters", "runtime.py")
RUST_ADAPTERS_DIR = os.path.join(ROOT, "crates", "xcelerate", "src", "adapters")
BINDINGS_DIR = os.path.join(ROOT, "bindings")
PY_BINDING_DIR = os.path.join(BINDINGS_DIR, "python", "xcelerate")
NODE_BIN = os.path.join(
    os.environ.get("ProgramFiles", r"C:\Program Files"), "nodejs", "node.exe"
)

_DOLLAR_NAMES = {
    "$": "query_selector",
    "$$": "query_selector_all",
    "$eval": "eval_on_selector",
    "$$eval": "eval_on_selector_all",
    "$x": "query_selector_xpath",
}


def log(prefix, message):
    print(f"[{prefix}] {message}")


def run(cmd, cwd=None, capture=False):
    """Run a command (list or str). Returns the CompletedProcess."""
    printable = cmd if isinstance(cmd, str) else " ".join(cmd)
    print(f"[EXEC] {printable}")
    result = subprocess.run(cmd, cwd=cwd, shell=True, capture_output=capture, text=True)
    if capture:
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(f"[STDERR] {result.stderr}")
    return result


def run_checked(cmd, cwd=None, capture=False):
    result = run(cmd, cwd=cwd, capture=capture)
    if result.returncode != 0:
        log("ERROR", f"command failed ({result.returncode}): {cmd if isinstance(cmd, str) else ' '.join(cmd)}")
        sys.exit(result.returncode)
    return result


def load_profiles():
    """Load every adapter profile, sorted by file name."""
    profiles = []
    for name in sorted(os.listdir(PROFILES_DIR)):
        if name.endswith(".json"):
            with open(os.path.join(PROFILES_DIR, name), "r", encoding="utf-8") as handle:
                profiles.append(json.load(handle))
    return profiles


def load_profile(name):
    path = os.path.join(PROFILES_DIR, f"{name}.json")
    if not os.path.exists(path):
        return None
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def write_profile(name, profile):
    path = os.path.join(PROFILES_DIR, f"{name}.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(profile, handle, indent=2)
        handle.write("\n")


def member_name(method):
    """The upstream library name of a profile method/member."""
    return method["name"]


def emitted_name(method):
    """The identifier a generator should emit (``as`` if present)."""
    return method.get("as") or method["name"]


def safe_name(name):
    """A valid Python identifier for a library member name."""
    if name in _DOLLAR_NAMES:
        return _DOLLAR_NAMES[name]
    cleaned = re.sub(r"[^0-9a-zA-Z_]", "_", name).strip("_") or "member"
    if cleaned[0].isdigit():
        cleaned = "_" + cleaned
    if keyword.iskeyword(cleaned):
        cleaned = cleaned + "_"
    return cleaned


def find_tool(name, extra_dirs=None):
    """Locate a tool on PATH, falling back to common install directories."""
    found = shutil.which(name)
    if found:
        return found
    dirs = list(extra_dirs or [])
    dirs += [
        os.path.join(os.path.expanduser("~"), ".cargo", "bin"),
        os.path.join(os.environ.get("USERPROFILE", ""), ".cargo", "bin"),
        os.path.join(os.path.expanduser("~"), ".dotnet", "tools"),
    ]
    for directory in dirs:
        if directory and os.path.isdir(directory):
            for entry in os.listdir(directory):
                if name.lower() in entry.lower() and entry.endswith(".exe"):
                    return os.path.join(directory, entry)
    return name


def workspace_version():
    """Read the version from the workspace [workspace.package] section."""
    path = os.path.join(ROOT, "Cargo.toml")
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("version ="):
                return line.split("=", 1)[1].strip().strip('"')
    return "0.0.0"
