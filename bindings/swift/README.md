# Xcelerate Swift SDK

Swift bindings for the xcelerate Rust CDP engine, generated with
[UniFFI](https://mozilla.github.io/uniffi-rs/). Swift is a built-in UniFFI
target, so no extra generator is required.

## Requirements

- Swift 5.9+ (Xcode on macOS, or the swift.org toolchain)
- The native xcelerate library for your platform (`libxcelerate.dylib` on macOS,
  `xcelerate.dll` on Windows) - built from source by the script below

## Generate / build

```bash
# from the repository root
python scripts/generate_bindings/swift.py
```

The script runs UniFFI and arranges a SwiftPM package:

```
bindings/swift/
  Package.swift
  Sources/
    Xcelerate/       xcelerate.swift        # the Swift API
    xcelerateFFI/    xcelerateFFI.h         # the low-level C FFI
                     module.modulemap
  libxcelerate.dylib / xcelerate.dll
```

or, once generated, inside this directory:

```bash
swift build
```

## Usage

```swift
import Xcelerate

let config = BrowserConfig(
    headless: true,
    detached: true,
    executablePath: nil,                // auto-discover Chrome/Edge
    plugins: ["stealth", "human"]       // opt into built-in plugins
)
let browser = try await Browser.launch(config: config)
let page = try await browser.newPage(url: "https://example.com")
print(try await page.title())
try await browser.close()
```

Every call is `async throws`. Plugins load from disk sandboxed (WebAssembly)
behind the capability gate; `loadPlugin` instantiates the `entrypoint` component
the manifest names.

## Publishing

SwiftPM has no central registry: a package is a tagged repository whose **root**
contains `Package.swift`. This repo keeps the package in `bindings/swift`, so
split it onto its own repository:

```bash
python scripts/publish/swift.py                    # git subtree split -> local branch
python scripts/publish/swift.py --push --remote swift
```

Consumers then depend on that repository:

```swift
.package(url: "https://github.com/AzzoDude/xcelerate-swift", from: "1.0.12")
```

For binary distribution, publish an XCFramework and reference it with
`.binaryTarget` so consumers do not need to build the Rust core.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
