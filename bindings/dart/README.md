# Xcelerate Dart / Flutter SDK

> **Note:** the early `uniffi-bindgen-dart` 0.1.x generator has a few codegen
> bugs (async `bool` returns, the async constructor, and a `close` name clash).
> `generate_dart_bindings.py` applies targeted post-processing so the generated
> package analyzes clean; see `_patch()` in that script. Re-check the fixes when
> bumping the generator.

Dart bindings for the xcelerate Rust CDP engine. Dart is not a built-in UniFFI
target, so the sources are generated with the external
[`uniffi-bindgen-dart`](https://github.com/nchapman/uniffi-bindgen-dart) generator
(pinned to a release that targets UniFFI 0.31).

## Requirements

- Dart SDK 3.1+ or Flutter
- The `uniffi-bindgen-dart` generator: `cargo install uniffi-bindgen-dart`
- The native xcelerate library, shipped as `uniffi_xcelerate.dll` /
  `libuniffi_xcelerate.so` / `libuniffi_xcelerate.dylib` (copied by the script)

## Generate / build

```bash
# from the repository root
cargo install uniffi-bindgen-dart      # once
python scripts/generate_dart_bindings.py
```

The script assembles a pub package:

```
bindings/dart/
  pubspec.yaml
  lib/xcelerate.dart
  src/uniffi_xcelerate.dll / libuniffi_xcelerate.so / libuniffi_xcelerate.dylib   # local only
```

or, once generated, inside this directory:

```bash
dart pub get
dart analyze
```

## Usage

The generated binding loads the native library by name; point it at the bundled
copy with `libraryPath` when the loader cannot find it:

```dart
import 'package:xcelerate/xcelerate.dart';

final config = BrowserConfig(
  headless: true,
  detached: true,
  executablePath: null,           // auto-discover Chrome/Edge
  plugins: ['stealth', 'human'],  // opt into built-in plugins
);
final browser = await Browser.launch(config: config);
final page = await browser.newPage(url: 'https://example.com');
print(await page.title());
await browser.closeBrowser();   // `close` is reserved by the disposer
```

Plugins load from disk out-of-process behind the capability gate; `loadPlugin`
runs the `entrypoint` the manifest names.

## Publishing

```bash
python scripts/publish_dart.py          # regenerate + `dart pub publish --dry-run`
python scripts/publish_dart.py --push   # `dart pub publish --force`
```

`dart pub publish` authenticates with a Google account (OAuth on first run) or a
`PUB_TOKEN` in CI. The published package is **source-only**: the native library
is platform-specific, so it is not bundled in the pub package - bundle it with
your app and pass `libraryPath` (or place it where `DynamicLibrary.open` finds
it). The generator still copies the host library under `src/` for local runs.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
