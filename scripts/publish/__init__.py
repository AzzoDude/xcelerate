"""Library of per-target publish scripts.

Each module exposes a ``main()`` that builds one binding and, with ``--push``,
publishes it (RubyGems, the Go module proxy, the PowerShell Gallery or a SwiftPM
repository). Dart is deliberately absent: pub.dev publishes from CI on a tag push
(see ``.github/workflows/publish.yml``). Import them so they can be driven
programmatically::

    from publish import ruby

    ruby.main()
"""

from __future__ import annotations

from . import go, powershell, ruby, swift

__all__ = ["go", "powershell", "ruby", "swift"]
