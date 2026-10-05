"""Library of per-target publish scripts.

Each module exposes a ``main()`` that builds one binding and, with ``--push``,
publishes it (RubyGems, pub.dev, the Go module proxy, the PowerShell Gallery or a
SwiftPM repository). Import them here so they can be driven programmatically::

    from publish import ruby

    ruby.main()
"""

from __future__ import annotations

from . import dart, go, powershell, ruby, swift

__all__ = ["dart", "go", "powershell", "ruby", "swift"]
