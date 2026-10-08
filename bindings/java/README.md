# Xcelerate Java SDK

Java/JVM bindings for the xcelerate Rust CDP engine. Java is not a built-in
UniFFI target, so the sources are generated with the external
[`uniffi-bindgen-java`](https://github.com/IronCoreLabs/uniffi-bindgen-java)
generator (pinned to the release that targets UniFFI 0.31).

## Requirements

- **JDK 22+** - the generated code uses the Foreign Function & Memory API
  (Project Panama), so there are no external runtime dependencies.
- `gradle` on `PATH`, or installed under `tools/gradle/` by
  `python ../../scripts/install_toolchains.py`

## Generate / build

```bash
# from the repository root
python scripts/install_toolchains.py        # JDK 22+, Gradle, uniffi-bindgen-java
python scripts/generate_bindings/java.py    # sources + native libs + Gradle build
```

or, once generated, inside this directory:

```bash
gradle build
```

The generated sources live in `src/main/java/uniffi/xcelerate/`.

## Usage

```java
import uniffi.xcelerate.Browser;
import uniffi.xcelerate.BrowserConfig;
import uniffi.xcelerate.Page;

public class Demo {
    public static void main(String[] args) throws Exception {
        // headless, detached, executablePath, plugins (none by default)
        Browser browser = Browser.launch(
                new BrowserConfig(true, false, null, null)).get();
        Page page = browser.newPage("https://example.com").get();
        System.out.println(page.title().get());
        byte[] png = page.screenshotFull().get();
        browser.closeBrowser().get();
    }
}
```

Every call returns a `CompletableFuture`, so `get()` (or `thenApply(...)`) is
required. Run with the native access flag:

```bash
java --enable-native-access=ALL-UNNAMED \
     -Duniffi.component.xcelerate.libraryOverride=/abs/path/xcelerate.dll \
     Demo
```

`libraryOverride` may be an absolute path (loaded with `System.load`) or a bare
name (resolved via `java.library.path`). Without it the generated code calls
`System.loadLibrary("xcelerate")`.

## Plugins

Plugins are default-deny and loaded from disk; the built-in catalog is empty.
`BrowserConfig` takes plugin **paths** (a directory or a `plugin.json`) as its
fourth argument, or call `browser.loadPlugin("plugins/my-plugin")` after launch.
The same fixed bridge is available in Java:

```java
System.out.println(browser.pluginNames());       // [] — nothing loaded yet
browser.loadPlugin("plugins/my-plugin");        // a directory or a plugin.json
System.out.println(browser.pluginNames());       // ["my-plugin"]
System.out.println(browser.availablePlugins());  // ["my-plugin"]
var myPlugin = browser.plugin("my-plugin");
System.out.println(myPlugin.invoke("info", "{}").get());
```

`loadPlugin` loads a sandboxed (WebAssembly) plugin from disk, capability-gated;
see the [top-level README](../../README.md#plugins) for capabilities and the
audit log.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
