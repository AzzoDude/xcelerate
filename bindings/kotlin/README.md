# Xcelerate Kotlin SDK

Kotlin/JVM bindings for the xcelerate Rust CDP engine, generated with
[UniFFI](https://mozilla.github.io/uniffi-rs/).

## Requirements

- JDK 17+ (the build uses whatever JDK Gradle runs on)
- `gradle` on `PATH`, or installed under `tools/gradle/` by
  `python ../../scripts/install_toolchains.py`

## Generate / build

```bash
# from the repository root
python scripts/install_toolchains.py          # JDK + Gradle (uses winget where possible)
python scripts/generate_bindings/kotlin.py    # sources + native libs + Gradle build
```

or, once generated, inside this directory:

```bash
gradle build
```

The generated sources live in `src/main/kotlin/uniffi/xcelerate/xcelerate.kt`
and depend on JNA (FFI) and kotlinx-coroutines (the async surface); both are
declared in `build.gradle.kts`.

## Usage

```kotlin
import uniffi.xcelerate.Browser
import uniffi.xcelerate.BrowserConfig
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    // headless = true, detached = true, executablePath = null, plugins = null.
    // Plugins are none by default — load one from disk to opt in.
    val browser = Browser.launch(BrowserConfig())
    val page = browser.newPage("https://example.com")
    println(page.title())
    val png = page.screenshotFull()
    browser.closeBrowser()
}
```

Every call is a `suspend` function, so run inside a coroutine
(`runBlocking { ... }`, `suspend fun`, etc.).

## Plugins

Plugins are default-deny and loaded from disk; the built-in catalog is empty.
`BrowserConfig(plugins = listOf("plugins/my-plugin"))` takes **paths** (a
directory or a `plugin.json`) before launch, or call
`browser.loadPlugin("plugins/my-plugin")` after. The same fixed bridge is
available in Kotlin:

```kotlin
println(browser.pluginNames())        // [] — nothing loaded yet
browser.loadPlugin("plugins/my-plugin") // a directory or a plugin.json
println(browser.pluginNames())        // ["my-plugin"]
println(browser.availablePlugins())   // ["my-plugin"]
val myPlugin = browser.plugin("my-plugin")
println(myPlugin.invoke("info", "{}"))
```

`loadPlugin` loads a sandboxed (WebAssembly) plugin from disk, capability-gated;
see the [top-level README](../../README.md#plugins) for capabilities and the
audit log.

## Native library

The core library is bundled in the jar under `uniffi/xcelerate/`. If the
loader cannot find it (for example when the jar is shaded), point it at an
extracted copy:

```bash
java -Duniffi.component.xcelerate.libraryOverride=/abs/path/libxcelerate.so ...
```

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
