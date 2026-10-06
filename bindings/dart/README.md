# Xcelerate Dart / Flutter SDK

> **Note:** the early `uniffi-bindgen-dart` 0.1.x generator has several codegen
> bugs for this API. `scripts/generate_bindings/dart.py` post-processes the
> output before it is written; see `_patch()` in that script. The fixes are:
>
> - async `bool` returns and the async constructor are emitted synchronously;
> - `Browser::close`/`Page::close` collide with the disposer (renamed
>   `closeBrowser`/`closePage`);
> - the `Browser.launch` async constructor (a `RustBuffer` argument) is stubbed
>   with `UnsupportedError`, so the whole binding is unusable - it is
>   implemented directly;
> - error payloads are dropped ("extra bytes remaining") - decoded here;
> - `Vec<u8>` returns keep their UniFFI length prefix - stripped here;
> - the FFI helper symbols use an `uniffi_`-prefixed library name; the generator
>   is invoked with `--crate xcelerate` so they match the exported symbols.
>
> Re-check every fix when bumping the generator.

Dart bindings for the xcelerate Rust CDP engine. Dart is not a built-in UniFFI
target, so the sources are generated with the external
[`uniffi-bindgen-dart`](https://github.com/nchapman/uniffi-bindgen-dart) generator
(pinned to a release that targets UniFFI 0.31).

## Requirements

- Dart SDK 3.2+ or Flutter
- The native xcelerate core, which the published package bundles for macOS
  (arm64/x64), Windows (x64) and Linux (x64) — no build step is needed to use it.
  From a source checkout, build it as below. It must be built with UniFFI's
  `scaffolding-ffi-buffer-fns` feature (enabled in the workspace `Cargo.toml`),
  which exports the `uniffi_ffibuffer_*` entry points this binding calls.
- To regenerate the bindings: the `uniffi-bindgen-dart` generator
  (`cargo install uniffi-bindgen-dart`).

## Generate / build

```bash
# from the repository root
cargo install uniffi-bindgen-dart      # once
python scripts/generate_bindings/dart.py
```

The script assembles a pub package. For a source checkout it stages the host
library under its own `src/<os>-<arch>/` token so local runs use the same lookup
as a published package; a release stages the other platforms in CI:

```
bindings/dart/
  pubspec.yaml
  lib/xcelerate.dart
  src/<os>-<arch>/xcelerate.dll | libxcelerate.so | libxcelerate.dylib
```

or, once generated, inside this directory:

```bash
dart pub get
dart analyze
```

## Usage

The binding loads the native library the package bundles for the current
platform. If it is missing (an unlisted platform), it falls back to the OS
search path; pass `libraryPath` to name a library explicitly:

```dart
import 'package:xcelerate/xcelerate.dart';

final config = BrowserConfig(
  headless: true,
  detached: true,
  executablePath: null,           // auto-discover Chrome/Edge
  plugins: ['stealth', 'human'],  // opt into built-in plugins
);
final browser = await Browser.launch(config);
final page = await browser.newPage('https://example.com');
print(await page.title());
await browser.closeBrowser();   // `close` is reserved by the disposer
```

Plugins load from disk sandboxed (WebAssembly) behind the capability gate;
`loadPlugin` instantiates the `entrypoint` component the manifest names.

## Publishing

```bash
# `version:` in pubspec.yaml must match the tag.
git tag vX.Y.Z && git push origin vX.Y.Z
```

Pushing a `vX.Y.Z` tag runs `.github/workflows/publish-dart.yml`, which publishes
through pub.dev's OIDC automated publishing (the `pub.dev` environment) with no
token involved. To publish by hand, run `dart pub publish` inside `bindings/dart`;
that authenticates with a Google account (OAuth on first run). The package
**bundles a prebuilt native library** for macOS (arm64/x64), Windows (x64) and
Linux (x64) under `src/<os>-<arch>/`, so `dart pub add xcelerate` works with no
build step; on an unlisted platform the loader falls back to the OS search path
(`libraryPath` overrides it). Those binaries are staged by CI and are not
committed to git.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
