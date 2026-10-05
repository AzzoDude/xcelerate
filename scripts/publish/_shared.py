"""Shared helpers for the per-target publish scripts.

The publish scripts live in ``scripts/publish/`` and are invoked directly, so
this module puts the parent ``scripts/`` directory on ``sys.path`` before
re-exporting the workspace-level helpers. It also centralises the small pieces
of publish logic (flag parsing, git, regenerating a binding) that every target
used to repeat.
"""

from __future__ import annotations

import os
import subprocess
import sys
from typing import NamedTuple

# The workspace-level ``common`` module is one directory up from this package.
_SCRIPTS_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if _SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, _SCRIPTS_DIR)

# Imported after the sys.path fix above so ``common`` resolves.
from common import (
    ROOT,
    find_tool,
    log,
    run_checked,
    workspace_version,
)

# The binding generators live in the sibling ``generate_bindings`` package.
GENERATORS_DIR = os.path.join(_SCRIPTS_DIR, "generate_bindings")

__all__ = [
    "GENERATORS_DIR",
    "ROOT",
    "Options",
    "find_tool",
    "git",
    "log",
    "parse_args",
    "regenerate",
    "run_checked",
    "workspace_version",
]


class Options(NamedTuple):
    """Flags shared by the publish scripts."""

    push: bool
    remote: str | None


def parse_args(argv=None):
    """Parse ``--push`` and ``--remote <name>`` from ``argv`` (default ``sys.argv``)."""
    argv = list(sys.argv[1:] if argv is None else argv)
    remote = None
    if "--remote" in argv:
        index = argv.index("--remote")
        if index + 1 < len(argv):
            remote = argv[index + 1]
    return Options(push="--push" in argv, remote=remote)


def regenerate(language):
    """Regenerate a binding with its generator before packaging it."""
    run_checked([sys.executable, os.path.join(GENERATORS_DIR, f"{language}.py")], cwd=ROOT)


def git(*args):
    """Run a git command in the repository root, capturing its output."""
    return subprocess.run(
        ["git", *args], cwd=ROOT, text=True, capture_output=True, check=False
    )
