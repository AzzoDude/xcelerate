"""Library of per-language UniFFI binding generators.

Each module exposes a ``main()`` that generates one binding together with the
packaging metadata around it. Import them here so callers (for example
``scripts/generate_bindings.py``) can drive the pipeline programmatically::

    from generate_bindings import csharp

    csharp.main()

``PIPELINE`` lists the generators in the order the orchestrator runs them.
"""

from __future__ import annotations

from . import (
    csharp,
    dart,
    go,
    java,
    javascript,
    kotlin,
    powershell,
    python,
    ruby,
    swift,
)

# Ordered generators for the binding pipeline.
PIPELINE = [
    csharp,
    python,
    javascript,
    kotlin,
    java,
    swift,
    ruby,
    dart,
    go,
    powershell,
]

__all__ = [
    "PIPELINE",
    "csharp",
    "dart",
    "go",
    "java",
    "javascript",
    "kotlin",
    "powershell",
    "python",
    "ruby",
    "swift",
]
