# xcelerate Dart examples

Runnable examples for the [`xcelerate`](https://pub.dev/packages/xcelerate) Dart
bindings.

| Example | What it does |
| --- | --- |
| [`bin/quickstart.dart`](bin/quickstart.dart) | Launches Chrome, opens `example.com`, reads the title and a paragraph, and saves a screenshot. |
| [`bin/pixelscan_scroll.dart`](bin/pixelscan_scroll.dart) | Opens the Pixelscan bot-check page in a **visible** window and scrolls around it like a person, saving screenshots. |

## Prerequisites

- The Dart SDK 3.1+ on `PATH`.
- Chrome or Edge installed (auto-discovered), or pass `--executable`.
- The native library, built from this repository (see below).

## Run

```bash
cd examples/dart
dart pub get

dart run bin/quickstart.dart
dart run bin/pixelscan_scroll.dart
```

### Why the local override

`pubspec.yaml` declares `xcelerate: ^1.0.10`, but points it at
`../../bindings/dart` with a [`dependency_overrides`][override] entry. The
published 1.0.9 Dart binding cannot run:

- it target's UniFFI's *ffibuffer* ABI, which the core only exports when built
  with the `scaffolding-ffi-buffer-fns` feature (now enabled in this workspace);
- it uses the wrong FFI helper symbol prefix (`ffi_uniffi_xcelerate_…` instead of
  `ffi_xcelerate_…`);
- it stubs the `Browser.launch` async constructor with `UnsupportedError`;
- it mis-decodes error payloads and leaves a length prefix on screenshot bytes.

The generated binding in `bindings/dart/` has all of these fixed (see
`scripts/generate_bindings/dart.py`), and 1.0.10 publishes the corrected crate.
Once 1.0.10 is on pub.dev, delete the `dependency_overrides` block and the
examples use pub.dev as-is.

[override]: https://dart.dev/tools/pub/dependencies#dependency-overrides

### The native library

`lib/src/native_library.dart` locates the library for you - no path to set.
It checks `XCELERATE_LIBRARY` first, then the conventional build outputs, then
walks the project tree, accepting either `xcelerate.dll` or
`uniffi_xcelerate.dll`.

Build it either way:

```bash
# From the repository root: just the cdylib (the full-workspace build can fail
# on unrelated crates such as xcelerate-cli).
cargo build --release -p xcelerate --lib      # -> target/release/xcelerate.dll

# ...or generate the whole Dart binding package.
cargo install uniffi-bindgen-dart             # once
python scripts/generate_bindings/dart.py      # -> bindings/dart/src/xcelerate.dll
```

To point at a specific copy instead, set `XCELERATE_LIBRARY=/path/to/library`,
or drop one at `examples/dart/native/`.

### `pixelscan_scroll.dart` options

```
dart run bin/pixelscan_scroll.dart [url] [options]

  url               Page to scroll (default https://pixelscan.net/bot-check)
  --rounds N        Down/up scroll bursts to perform (default 4)
  --out DIR         Screenshot directory (default pixelscan_shots)
  --executable PATH Path to the Chrome/Edge binary
```

It always runs a real, visible window. The bot-check page reports headless
Chrome, so a headless run would be flagged before it ever scrolled; the example
therefore has no headless mode by design.

Screenshots land in `pixelscan_shots/`; `full_page.png` is the full-page capture.
