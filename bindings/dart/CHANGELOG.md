# Changelog

## 1.0.10

- Make the binding usable: the previous release could not start a browser.
- Target the FFI ABI the core actually exports (ffibuffer scaffolding and the
  `ffi_xcelerate_*` helper symbols); generated with `--crate xcelerate`.
- Implement the async `Browser.launch` constructor (was a throwing stub).
- Decode error payloads so failures raise a readable
  `XcelerateErrorExceptionFlat` instead of "extra bytes remaining".
- Strip the UniFFI length prefix from `Vec<u8>` returns (screenshots were
  corrupted by 4 bytes).
- The native library is now named `xcelerate.dll` / `libxcelerate.so` /
  `libxcelerate.dylib` and must be built with `uniffi`'s
  `scaffolding-ffi-buffer-fns` feature.
- The `LICENSE` file now carries both the MIT and Apache-2.0 texts (the
  project is dual-licensed, at your option).

## 1.0.9

- Initial Dart/Flutter binding for the xcelerate CDP engine.
- Exposes `Browser`, `BrowserConfig`, `Page`, and `Element`, plus the plugin
  bridge (`pluginNames`, `availablePlugins`, `usePlugin`, `loadPlugin`, `plugin`).
- `Browser.launch` is async; `Browser.close` / `Page.close` are exposed as
  `closeBrowser` / `closePage` (the disposer reserves `close`).
