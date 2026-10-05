[English](README.md) | **[简体中文](README.zh-CN.md)** | [日本語](README.ja-JP.md) | [Tiếng Việt](README.vi-VN.md)

# Xcelerate

[![CI](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/ci.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/ci.yml)
[![CodeQL](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/codeql.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/codeql.yml)
[![Semgrep](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/semgrep.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/semgrep.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ChaoswareHQ/xcelerate/badge)](https://securityscorecards.dev/viewer/?uri=github.com/ChaoswareHQ/xcelerate)

[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Crates.io downloads](https://img.shields.io/crates/d/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![PyPI](https://img.shields.io/pypi/v/xcelerate.svg)](https://pypi.org/project/xcelerate/)
[![npm](https://img.shields.io/npm/v/xcelerate.svg)](https://www.npmjs.com/package/xcelerate)
[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.azzodude/xcelerate.svg)](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate)
[![PowerShell Gallery](https://img.shields.io/powershellgallery/v/Xcelerate.svg)](https://www.powershellgallery.com/packages/Xcelerate)

[![docs.rs](https://img.shields.io/docsrs/xcelerate.svg)](https://docs.rs/xcelerate)
[![Rust](https://img.shields.io/badge/rust-1.99%2B-dea584.svg)](https://github.com/ChaoswareHQ/xcelerate/blob/master/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Xcelerate 是一个高性能、轻量级的 Chrome DevTools Protocol（CDP）客户端，
为 Rust、.NET、Python、JavaScript（Node.js）、Kotlin、Java、Swift、Ruby、
Dart/Flutter 和 Go 提供了符合语言习惯的绑定。它将快速的 Rust 核心与
异步优先（async-first）的 API 以及数据驱动的适配器层相结合，让现有的
Selenium、Playwright 和 Puppeteer 脚本能够在同一引擎上运行。

## 绑定

| 语言 | 包 | 注册表 |
| --- | --- | --- |
| Rust | `xcelerate` | [crates.io](https://crates.io/crates/xcelerate) |
| Python | `xcelerate` | [PyPI](https://pypi.org/project/xcelerate/) |
| JavaScript | `xcelerate` | [npm](https://www.npmjs.com/package/xcelerate) |
| .NET | `Xcelerate` | [NuGet](https://www.nuget.org/packages/Xcelerate) |
| Kotlin | `io.github.azzodude:xcelerate` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate) |
| Java | `io.github.azzodude:xcelerate-java` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate-java) |
| Swift | `Xcelerate` | 从源码构建（[说明](bindings/swift/README.md)） |
| Ruby | `xcelerate` | 从源码构建（[说明](bindings/ruby/README.md)） |
| Dart / Flutter | `xcelerate` | 从源码构建（[说明](bindings/dart/README.md)） |
| Go | `xcelerate` | 从源码构建（[说明](bindings/go/README.md)） |
| PowerShell | `Xcelerate` | [PowerShell Gallery](https://www.powershellgallery.com/packages/Xcelerate) |

## 功能特性

- **自动化的进程管理** —— 发现并启动 Chrome 或 Edge，并管理浏览器进程的生命周期。
- **安全优先的插件** —— 默认拒绝（default-deny）的插件系统，并带有仅追加
  （append-only）的审计日志。你可选择启用的内置插件有：`stealth` 和 `human`。
- **Stealth 与 human 插件** —— `stealth` 会应用二进制补丁和运行时的
  JavaScript 载荷，以减少自动化指纹；`human` 让输入表现得像真人（贝塞尔鼠标
  移动、按节奏打字、不均匀滚动）。二者都通过 `BrowserConfig.plugins` 按浏览器
  启用。
- **异步优先** —— 在 Rust 中构建于 `tokio` 之上，并在每个绑定中使用
  `async`/`await`。
- **API 风格适配器** —— 在原生引擎之上暴露 Selenium、Playwright 和 Puppeteer
  的方法名，由声明式配置（profile）生成。
- **多语言绑定** —— 单一核心，通过 `uniffi` 为 Rust、Python、
  JavaScript（Node.js）、.NET、Kotlin、Java、Swift、Ruby、Dart/Flutter 和 Go
  生成绑定，此外还有一个基于 .NET SDK 的 PowerShell 模块。
- **会话视频录制** —— 通过 CDP 屏幕录制（screencast）将页面捕获为视频；
  无需外部工具即可写出 Motion-JPEG AVI，当 `ffmpeg` 位于 `PATH` 中时可写出
  H.264/VP9 的 MP4/WebM。
- **代理池** —— 通过内置的本地网关，将浏览器路由到一个或多个上游 HTTP 代理
  （可带凭据）。
- **持久化配置（profile）** —— 使用持久的 `user-data-dir` 在多次运行之间保留
  登录状态、cookie 和站点存储。
- **无障碍快照** —— 页面的语义化 `role`/`name` 视图，用于稳健的选择器和由
  智能体（agent）驱动的自动化。
- **CLI 与 MCP 服务器** —— `xcelerate` 命令用于一次性操作，`xcelerate mcp`
  （或 `xcelerate-mcp` 二进制文件）用于从 MCP 客户端驱动浏览器。

## 安装

### Rust

```toml
[dependencies]
xcelerate = "1.0.9"
tokio = { version = "1", features = ["full"] }
```

### Python

```bash
pip install xcelerate
```

### JavaScript (Node.js)

```bash
npm install xcelerate
```

### .NET / C#

```powershell
dotnet add package Xcelerate
```

### Kotlin / Java

已发布到 Maven Central，坐标为 `io.github.azzodude:xcelerate`（Kotlin）和
`io.github.azzodude:xcelerate-java`（Java）：

```kotlin
// build.gradle.kts
dependencies {
    implementation("io.github.azzodude:xcelerate:1.0.9")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.9")   // Java
}
```

Java 绑定需要 **JDK 22+**，并需以 `--enable-native-access=ALL-UNNAMED` 运行。
用法参见 [`bindings/kotlin/README.md`](bindings/kotlin/README.md) 和
[`bindings/java/README.md`](bindings/java/README.md)。

从源码构建它们：

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_bindings/kotlin.py  # Kotlin sources + Gradle build
python scripts/generate_bindings/java.py    # Java sources + Gradle build
```

### Swift / Ruby / Dart / Go

这些目标使用通用生成器从源码构建。Swift 和 Ruby 是 UniFFI 的内置目标；
Dart 需要 `uniffi-bindgen-dart`；Go 需要 NordSecurity 的 `uniffi-bindgen-go`
生成器和 Go 工具链。

```bash
python scripts/generate_bindings/swift.py    # Swift sources + SwiftPM package
python scripts/generate_bindings/ruby.py     # Ruby sources + gemspec
cargo install uniffi-bindgen-dart            # once
python scripts/generate_bindings/dart.py     # Dart sources + pubspec
go install github.com/NordSecurity/uniffi-bindgen-go/v2/uniffi-bindgen-go@latest
python scripts/generate_bindings/go.py       # Go sources + go.mod
```

### PowerShell

```powershell
Install-PSResource Xcelerate      # PSResourceGet (PowerShell 7.4+)
# or, with PowerShellGet:
Install-Module Xcelerate
```

PowerShell 没有 UniFFI 生成器，因此该模块封装了 .NET SDK。若要从源码构建并
暂存（stage）载荷，请改为执行：

```powershell
python scripts/generate_bindings/powershell.py
Import-Module ./bindings/powershell/Xcelerate.psd1
```

用法参见 [`bindings/powershell/README.md`](bindings/powershell/README.md)，或
运行位于 [`examples/powershell/quickstart.ps1`](examples/powershell/quickstart.ps1) 的端到端示例。

各包的 README 参见 [`bindings/`](bindings/)。

## 快速开始（Rust）

```rust
use xcelerate::{Browser, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), xcelerate::XcelerateError> {
    let browser = Browser::launch(BrowserConfig::default()).await?;
    let page = browser.new_page("https://example.com".to_string()).await?;

    println!("Title: {}", page.title().await?);

    let heading = page.query_selector("h1".to_string()).await?;
    println!("Heading: {}", heading.text().await?);

    browser.close().await?;
    Ok(())
}
```

`BrowserConfig` 默认为无头（headless）且分离（detached）模式，**不启用**
stealth。请显式启用插件，或选择退出其他默认设置：

```rust
use xcelerate::BrowserConfig;

let config = BrowserConfig {
    headless: false,
    detached: false,
    executable_path: None,                      // auto-discover Chrome/Edge
    plugins: Some(vec!["stealth".to_string()]), // opt into the stealth plugin
    ..Default::default()
};
```

## 插件

Xcelerate 提供了**安全优先的插件系统**。插件是启动时配置、页面钩子和可调用
操作的命名集合，除非你启用它，否则它不会执行任何操作（**默认拒绝**）。
`stealth` 就是基于该系统构建的内置插件。

### 启用插件

插件列在 `BrowserConfig.plugins` 中，并在浏览器启动前启用，因此它们能够参与
启动过程本身（例如修补二进制文件）：

```rust
use xcelerate::{Browser, BrowserConfig};

let config = BrowserConfig {
    plugins: Some(vec!["stealth".to_string()]),
    ..Default::default()
};
let browser = Browser::launch(config).await?;
```

同一份列表在每种语言中都通过 `BrowserConfig` 传递：

```python
config = BrowserConfig(plugins=["stealth"])
```

```javascript
const browser = await Browser.launch({ plugins: ["stealth"] });
```

```csharp
var browser = await Browser.Launch(new BrowserConfig(Plugins: new[] { "stealth" }));
```

```kotlin
val config = BrowserConfig(plugins = listOf("stealth"))
```

```java
var config = new BrowserConfig(false, false, true, null, List.of("stealth"));
```

### 内置目录

| 插件 | 功能 | 操作 |
| --- | --- | --- |
| `stealth` | 在启动时修补浏览器二进制文件，并向每个文档注入反指纹载荷。 | `info` |
| `human` | 类人输入：带抖动的贝塞尔鼠标移动、会停顿并保持的点击、逐键打字延迟、不均匀的滚动步进。 | `info`, `move`, `click`, `type`, `scroll`, `delay` |

```rust
use xcelerate::{Browser, BrowserConfig};

let config = BrowserConfig {
    plugins: Some(vec!["stealth".to_string(), "human".to_string()]),
    ..Default::default()
};
let browser = Browser::launch(config).await?;

// Drive the human plugin through the same cross-language bridge.
let human = browser.plugin("human".into())?;
human.invoke("move".into(), r#"{"x": 320, "y": 240}"#.into()).await?;
human.invoke("type".into(), r#"{"text": "hello"}"#.into()).await?;
```

每个操作都在调用预算（invocation budget）下运行，并写入审计日志；插件只会
作用于交给它的页面。

### 检查与调用插件

每个绑定都暴露同样小巧而固定的桥接接口，因此新增插件永远不需要新的绑定代码：

| 方法 | 用途 |
| --- | --- |
| `available_plugins()` | 已编译进程序的内置目录中的名称 |
| `plugin_names()` | 该浏览器上已启用的插件 |
| `use_plugin(name)` | 在运行时启用内置插件 |
| `install_plugins([plugin])` | 安装作为 Cargo 库编译进来的受信任插件（仅限 Rust） |
| `load_plugin(path)` | 加载沙箱化的插件（一个目录或 `plugin.json`） |
| `plugin(name)` | 指向已启用插件的句柄 |
| `plugin(name).ops()` | 插件所暴露的操作 |
| `plugin(name).invoke(op, args_json)` | 使用 JSON 参数调用操作，并返回 JSON |

```rust
let enabled = browser.plugin_names();            // e.g. ["stealth", "human"]
let catalog = browser.available_plugins();       // ["stealth", "human"]
let stealth = browser.plugin("stealth".into())?; // error if not enabled
let info = stealth.invoke("info".into(), "{}".into()).await?;
```

### 插件的运行位置及其可执行的操作

| 模型 | 运行位置 | 权限 |
| --- | --- | --- |
| 内置 | 进程内，已编译进去 | 启动标志、二进制修补、分离式启动、初始化脚本 |
| 从磁盘加载 | WebAssembly，沙箱化，按能力（capability）授权 | 默认拒绝的子集，经过审计 |

能力在被授予之前会先经过分类。`LaunchControl`、`BinaryPatch` 和
`DetachedSpawn` **仅限内置**；`Evaluate`、`CdpProxy`、cookie 访问、初始化脚本、
截图和网络捕获属于**危险**能力，需要明确同意。从磁盘加载的插件是一个
WebAssembly 组件，在沙箱中运行，**没有任何环境权限（ambient authority）**：
宿主（host）导入是唯一的出路，它们按能力授权，且每次调用都会被审计——因此
插件无法自行访问文件系统或网络。危险回调**默认被拒绝**，必须在每次调用时间
与响应大小预算之内，通过 `XCELERATE_PLUGIN_ALLOW` 按宿主（针对单个插件或
广泛地）选择启用。参见 [`docs/plugins/`](docs/plugins/README.md)。

### 审计日志

每个特权操作（启动配置、页面钩子以及每次 `invoke`）都会记录在仅追加、哈希链式
的审计日志中。cookie 和凭据等机密信息绝不会写入其中。

```rust
assert!(browser.audit_verify()); // the hash chain is intact
println!("{}", browser.audit_log());
```

从磁盘加载的插件是否可信仍然由用户负责：引擎的职责是让它们*能够*做什么变得
明确、可审计，并在默认情况下不可行。

### 制作 mod

一步搭建一个入门 mod 并构建它——参见
[指南](docs/plugins/MAKING_A_MOD.md)：

```bash
xcelerate plugin new acme.hello
cd hello && ./build.sh          # Windows:  .\build.ps1
```

你只需编写普通的 Rust 操作处理器；xcelerate 负责处理 WebAssembly 的底层管道。

## API 风格适配器

适配器层在原生引擎之上暴露熟悉的 Selenium、Playwright 和 Puppeteer 方法名，
因此现有脚本只需极少改动即可移植。每个适配器都由声明式配置（profile）生成，
并在 Rust 和 Python 中完全类型化。

Rust：

```rust
use xcelerate::adapters::playwright;

let browser = playwright::launch(None).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

let paragraphs = page.query_selector_all("p".to_string()).await?;
println!("{} paragraph(s)", paragraphs.len());

browser.close().await?;
```

Python：

```python
from xcelerate import use

pw = use("playwright")
browser = await pw.launch()
page = await browser.new_page()
await page.goto("https://example.com")
print(await page.title())
await browser.close()
```

同一引擎也可通过 `use("selenium")` 和 `use("puppeteer")` 使用。配置（profile）
格式以及如何扩展适配器，参见
[`adapters/README.md`](adapters/README.md)。

## 原生绑定

生成的绑定在每种语言中暴露相同的异步 API。

Python：

```python
import asyncio
from xcelerate import Browser, BrowserConfig

async def main():
    # Opt into the stealth plugin - nothing runs unless it is enabled.
    browser = await Browser.launch(BrowserConfig(plugins=["stealth"]))
    page = await browser.new_page("https://example.com")
    print(await page.title())
    print(await browser.plugin_names())   # ["stealth"]
    await browser.close()

asyncio.run(main())
```

JavaScript (Node.js)：

```javascript
const { Browser } = require("xcelerate");

async function main() {
    const browser = await Browser.launch({ plugins: ["stealth"] });
    const page = await browser.newPage("https://example.com");
    console.log(await page.title());
    await browser.close();
}

main();
```

## 命令行界面

`xcelerate` 命令每次调用执行一个浏览器操作：

```bash
xcelerate title https://example.com
xcelerate screenshot https://example.com -o shot.png --full
xcelerate query https://example.com h1 --attr href
xcelerate query-all https://example.com 'a'   # text of every match
xcelerate evaluate https://example.com 'document.title'
xcelerate list                                  # built-in devices + plugins
xcelerate --device "iPhone 13" screenshot https://example.com -o phone.png
xcelerate plugins
```

全局标志适用于每个命令：`--no-headless`、`--detached`、
`--executable-path <path>`、`--plugins stealth,human`、`--device <name>` 和 `--timeout <ms>`。
`xcelerate --device <name> <command>` 会以内置移动设备渲染，`xcelerate list`
会列出所有设备和插件。
使用 `cargo install --path crates/xcelerate-cli` 安装，或在 Windows 上使用
`winget install Chaosware.Xcelerate`；从代码检出运行时，请在任何命令前加上
`cargo run -p xcelerate-cli --`。

## MCP 服务器

`xcelerate-mcp` —— 也可以通过 `xcelerate mcp` 调用 —— 是一个基于 stdio 的
[Model Context Protocol](https://modelcontextprotocol.io) 服务器，因此
MCP 客户端可以驱动真实的浏览器。它暴露 21 个工具，涵盖导航、标题、页面内容、
截图、PDF、点击、打字、悬停、按键、查询、JavaScript 求值和插件调用。

```jsonc
{ "mcpServers": { "xcelerate": { "command": "xcelerate-mcp" } } }
```

使用环境变量进行配置：`XCELERATE_CHROME`（浏览器路径）、
`XCELERATE_HEADLESS`（`1`/`true`，默认）、`XCELERATE_DETACHED`（`1`/`true`）以及
`XCELERATE_PLUGINS`（以逗号分隔，例如 `stealth,human`）。

## 视频录制

可以将页面录制为视频文件。录制由 CDP 屏幕录制（screencast）驱动，因此捕获是
变化驱动的：动画页面会产生真实的运动画面，而静态页面则得到一段很短的片段。

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

page.start_video("demo.mp4".to_string()).await?;
tokio::time::sleep(std::time::Duration::from_secs(5)).await;
let path = page.stop_video().await?;   // the path actually written
```

输出扩展名决定所用的后端：

- 当 `ffmpeg` 位于 `PATH` 中时，`.mp4` / `.mov` / `.mkv` / `.webm` 会按帧的真实
  时间戳用 `ffmpeg`（H.264 或 VP9）封装。如果缺少 `ffmpeg`，帧会回退到同名的
  `.avi`，而返回的路径会表明实际使用的是哪一种。
- 任何其他扩展名（例如 `.avi`）始终使用内置的 Motion-JPEG AVI 写入器，无需
  外部工具。

`start_video_with_options` 接受调优参数（`quality`、`max_width`、
`max_height`、`fps`、`ffmpeg`）。该 API 有意暂不通过 UniFFI 导出，因此它能在
Rust、CLI 和 MCP 服务器中使用，而不会改变生成绑定的校验和。

从 CLI 使用：

```bash
xcelerate record https://example.com -o demo.mp4 --duration 5
```

从 MCP 服务器使用：`browser_start_recording {path}` … `browser_wait
{milliseconds}` … `browser_stop_recording`。

## 代理

配置好的代理会应用于浏览器中的每个页面。Chrome 的 `--proxy-server` 无法携带
凭据，也无法切换代理，因此 xcelerate 在 `127.0.0.1` 上运行一个小型本地
HTTP/CONNECT 网关，并让 Chrome 指向它；该网关将每个连接转发到从**代理池**
中选择的一个上游，并注入 `Proxy-Authorization`。

```text
Chrome --(HTTP/CONNECT)--> xcelerate gateway (127.0.0.1) --> upstream pool --> internet
```

```bash
# environment (works from every language binding)
XCELERATE_PROXY=http://user:pass@proxy.example:8080 xcelerate title https://example.com
XCELERATE_PROXY_POOL=http://a:8080,http://b:8080 ./your-app      # round-robin

# CLI flag (repeatable)
xcelerate --proxy http://user:pass@proxy.example:8080 --proxy http://backup:8080 \
  title https://example.com
```

```rust
xcelerate::configure_proxy(&["http://user:pass@proxy.example:8080".to_string()])?;
let browser = Browser::launch(BrowserConfig::default()).await?;
```

- 上游的格式为 `http://[user:pass@]host:port`；代理池以轮询（round-robin）方式
  轮换。
- HTTPS 使用代理的 `CONNECT` 隧道；普通 HTTP 使用绝对形式（absolute-form）
  请求 —— 两者都经由上游转发并注入凭据。
- 不需要 SOCKS 上游：Chrome 通过 `--proxy-server` 原生支持 SOCKS，因此可以
  直接让 Chrome 指向它。
- 尚不支持 `https://` 上游（即到代理的 TLS）。

## 持久化配置（profile）

默认情况下，每个浏览器都会获得一个临时的配置目录，并在关闭时删除。将其指向
某个目录即可在多次运行之间保留 cookie、登录状态和站点存储：

```bash
XCELERATE_USER_DATA_DIR=~/.xcelerate/profile xcelerate title https://example.com
xcelerate --user-data-dir ./profile title https://example.com
```

```rust
xcelerate::configure_user_data_dir(Some("./profile".to_string()))?;
```

该路径会被规范化（否则相对的 `--user-data-dir` 会相对于 Chrome 自身的工作
目录解析），并且 `Browser::close` 会让 Chrome 在退出前将配置刷新到磁盘。

## 无障碍快照

`page.accessibility_snapshot()` 返回页面的紧凑语义视图 —— 按文档顺序排列的
`[{ role, name, value? }]` —— 在断言或驱动页面时，它比 CSS 选择器稳健得多。
它以 CLI 命令 `xcelerate accessibility <url>` 和 MCP 工具
`browser_accessibility` 的形式暴露。

## 工作区布局

```
xcelerate/
  crates/
    xcelerate-core/        # WebSocket transport and typed CDP command layer
    xcelerate-plugin-api/  # plugin trait, manifest, capabilities, audit, host interface
    xcelerate-plugins/     # built-in plugin catalog: name -> implementation lookup
    xcelerate/             # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/     # uniffi bindgen helper binary
    xcelerate-cli/         # CLI (binary `xcelerate`)
    xcelerate-mcp/         # `xcelerate-mcp` Model Context Protocol server
  plugins/
    stealth/               # stealth plugin: binary patching + anti-fingerprint payload
    human/                 # human plugin: human-like mouse, typing, and scrolling
  adapters/             # adapter profiles, runtime, and generator inputs
  bindings/             # generated Python, JavaScript, C#, Kotlin, Java, Swift, Ruby, Dart, and Go packages (plus the PowerShell module)
  docs/plugins/         # plugin authoring guide, JSON schema, examples
  scripts/              # code generation, harvesting, and release tooling
```

插件 API —— `Plugin` trait、`Manifest`、能力、审计日志以及 `PageHost` 接口 ——
位于 `crates/xcelerate-plugin-api/`。每个内置插件都是 `plugins/` 下的一个独立
crate（`stealth`、`human`）；`crates/xcelerate-plugins/` 只是将它们名称映射到
实现的目录。`stealth` crate 拥有自己的二进制修补器和反指纹载荷；引擎拥有
浏览器进程控制（`crates/xcelerate/src/process.rs`）。门面（facade）拥有
`PluginManager`，并将 `Page` 桥接到插件宿主接口，因此插件永远不会接触原始
页面或传输层。

## 开发

```bash
# Generate the API-style adapters (Python and Rust) from the profiles.
python scripts/generate_adapters.py

# Validate the profiles and print coverage.
python scripts/harvest_adapters.py --check

# Build, lint, and test.
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p xcelerate --lib                    # plugin + unit tests (no browser)
cargo test -p xcelerate --test adapters_e2e -- --nocapture

# JVM bindings (JDK 22+, Gradle, uniffi-bindgen-java).
python scripts/install_toolchains.py
```

端到端测试会启动真实浏览器，并需要 Chrome 或 Edge。可使用
`XCELERATE_TEST_URL` 和 `XCELERATE_CHROME` 让它们指向不同的站点或浏览器。

## 安全

发现了漏洞？请遵循 [SECURITY.md](SECURITY.md) —— 不要提交公开 issue。
受支持的版本、报告渠道和范围都列在那里。

## License

在以下两种许可下之一授权：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

由你自行选择。

除非你明确声明另有约定，否则根据 Apache-2.0 许可证的定义，你有意提交以纳入
本项目的任何贡献，都将按上述双许可授权，且不附加任何额外条款或条件。
