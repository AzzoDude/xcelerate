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


def run(cmd, cwd=None, capture=False, env=None):
    """Run a command (list or str). Returns the CompletedProcess."""
    printable = cmd if isinstance(cmd, str) else " ".join(cmd)
    print(f"[EXEC] {printable}")
    # Only a string command needs a shell (it may contain pipes/redirects); a
    # list is executed directly, which also avoids `shell=True` injection.
    result = subprocess.run(
        cmd, cwd=cwd, shell=isinstance(cmd, str), capture_output=capture, text=True, env=env
    )
    if capture:
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(f"[STDERR] {result.stderr}")
    return result


def run_checked(cmd, cwd=None, capture=False, env=None):
    result = run(cmd, cwd=cwd, capture=capture, env=env)
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
    """Locate a tool on PATH, falling back to common install directories.

    Only exact executable names are accepted. A substring match would resolve
    ``uniffi-bindgen`` to ``uniffi-bindgen-cs`` (or ``-node-js``), which would
    silently invoke the wrong generator.
    """
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
                stem = entry[:-4] if entry.lower().endswith(".exe") else entry
                if stem.lower() == name.lower():
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


def uniffi_bindgen():
    """Command prefix for the bundled Rust uniffi-bindgen helper.

    Prefers a pre-built binary over ``cargo run`` so CI (and local runs) do not
    recompile the helper on every invocation.
    """
    for candidate in (
        os.path.join(ROOT, "target", "debug", "uniffi-bindgen.exe"),
        os.path.join(ROOT, "target", "release", "uniffi-bindgen.exe"),
        os.path.join(ROOT, "target", "debug", "uniffi-bindgen"),
        os.path.join(ROOT, "target", "release", "uniffi-bindgen"),
    ):
        if os.path.exists(candidate):
            return [candidate]
    found = find_tool("uniffi-bindgen")
    if os.path.exists(found):
        return [found]
    return ["cargo", "run", "-p", "xcelerate-bindgen", "--"]


def jdk_major(path):
    """Major version of a JDK install (from its ``release`` file), or None."""
    if not path:
        return None
    release = os.path.join(path, "release")
    try:
        with open(release, "r", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("JAVA_VERSION="):
                    value = line.split("=", 1)[1].strip().strip('"')
                    return int(value.split(".")[0])
    except (OSError, ValueError):
        pass
    return None


def java_home():
    """Locate a JDK home, preferring a Gradle-compatible 22-24 build.

    The Java bindings need Java 22+ (Project Panama), but Gradle 8.10 only runs
    on JDK 24 or lower, so a 22-24 JDK is preferred when several are installed.
    """
    def _is_jdk(path):
        return path and (
            os.path.exists(os.path.join(path, "bin", "javac.exe"))
            or os.path.exists(os.path.join(path, "bin", "javac"))
        )

    candidates = []
    if os.environ.get("JAVA_HOME"):
        candidates.append(os.environ["JAVA_HOME"])
    javac = shutil.which("javac") or shutil.which("java")
    if javac:
        candidates.append(os.path.dirname(os.path.dirname(os.path.abspath(javac))))
    program_files = os.environ.get("ProgramFiles", r"C:\Program Files")
    for base in (
        os.path.join(program_files, "Microsoft"),
        os.path.join(program_files, "Eclipse Adoptium"),
        os.path.join(program_files, "Java"),
        os.path.join(program_files, "Amazon Corretto"),
    ):
        if os.path.isdir(base):
            for entry in sorted(os.listdir(base), reverse=True):
                candidates.append(os.path.join(base, entry))
    jdks = [candidate for candidate in candidates if _is_jdk(candidate)]
    for preferred in (22, 23, 24):
        for candidate in jdks:
            if jdk_major(candidate) == preferred:
                return candidate
    return jdks[0] if jdks else None


def gradle_tool():
    """Locate the Gradle launcher (PATH or ``tools/gradle/**/bin``)."""
    found = shutil.which("gradle")
    if found:
        return found
    tools = os.path.join(ROOT, "tools", "gradle")
    if os.path.isdir(tools):
        for entry in sorted(os.listdir(tools), reverse=True):
            for name in ("gradle.bat", "gradle"):
                candidate = os.path.join(tools, entry, "bin", name)
                if os.path.exists(candidate):
                    return candidate
    return None


def jvm_env():
    """A copy of the environment with JAVA_HOME/PATH pointed at the JDK."""
    env = dict(os.environ)
    home = java_home()
    if home:
        env["JAVA_HOME"] = home
        env["PATH"] = os.path.join(home, "bin") + os.pathsep + env.get("PATH", "")
    return env
