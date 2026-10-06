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

- Dart SDK 3.1+ or Flutter
- The `uniffi-bindgen-dart` generator: `cargo install uniffi-bindgen-dart`
- The native xcelerate library, shipped as `xcelerate.dll` / `libxcelerate.so`
  / `libxcelerate.dylib` (copied by the script). It must be built with UniFFI's
  `scaffolding-ffi-buffer-fns` feature (enabled in the workspace `Cargo.toml`),
  which exports the `uniffi_ffibuffer_*` entry points this binding calls.

## Generate / build

```bash
# from the repository root
cargo install uniffi-bindgen-dart      # once
python scripts/generate_bindings/dart.py
```

The script assembles a pub package:

```
bindings/dart/
  pubspec.yaml
  lib/xcelerate.dart
  src/xcelerate.dll / libxcelerate.so / libxcelerate.dylib   # local only
```

or, once generated, inside this directory:

```bash
dart pub get
dart analyze
```

## Usage

The generated binding loads the native library by name (`xcelerate`); pass
`libraryPath` when the loader cannot find it:

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
that authenticates with a Google account (OAuth on first run). The published
package is **source-only**: the native library
is platform-specific, so it is not bundled in the pub package - bundle it with
your app and pass `libraryPath` (or place it where `DynamicLibrary.open` finds
it). The generator still copies the host library under `src/` for local runs.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
