# Xcelerate Java SDK

Java/JVM bindings for the xcelerate Rust CDP engine. Java is not a built-in
UniFFI target, so the sources are generated with the third-party
[`uniffi-bindgen-java`](https://github.com/IronCoreLabs/uniffi-bindgen-java)
generator (pinned to the release that targets UniFFI 0.31).

## Requirements

- **JDK 22+** - the generated code uses the Foreign Function & Memory API
  (Project Panama), so there are no third-party runtime dependencies.
- `gradle` on `PATH`, or installed under `tools/gradle/` by
  `python ../../scripts/install_toolchains.py`

## Generate / build

```bash
# from the repository root
python scripts/install_toolchains.py        # JDK 22+, Gradle, uniffi-bindgen-java
python scripts/generate_java_bindings.py    # sources + native libs + Gradle build
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
        // headless, detached, executablePath, plugins
        Browser browser = Browser.launch(
                new BrowserConfig(true, false, null, java.util.List.of("stealth"))).get();
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

Plugins are default-deny. Enable the first-party `stealth` plugin with the fifth
`BrowserConfig` argument (`java.util.List.of("stealth")`) before launch. The same
fixed bridge is available in Java:

```java
System.out.println(browser.pluginNames());       // ["stealth"]
System.out.println(browser.availablePlugins());  // ["stealth", "human"]
var stealth = browser.plugin("stealth");
System.out.println(stealth.invoke("info", "{}").get());
```

`loadPlugin` refuses third-party plugins until the sandboxed runner ships; see the
[top-level README](../../README.md#plugins) for trust tiers and the audit log.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
