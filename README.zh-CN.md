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
[![pub.dev](https://img.shields.io/pub/v/xcelerate.svg)](https://pub.dev/packages/xcelerate)
[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.azzodude/xcelerate.svg)](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate)
[![PowerShell Gallery](https://img.shields.io/powershellgallery/v/Xcelerate.svg)](https://www.powershellgallery.com/packages/Xcelerate)

[![docs.rs](https://img.shields.io/docsrs/xcelerate.svg)](https://docs.rs/xcelerate)
[![Rust](https://img.shields.io/badge/rust-1.99%2B-dea584.svg)](https://github.com/ChaoswareHQ/xcelerate/blob/master/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Xcelerate 是一个高性能、轻量级的浏览器自动化引擎，为 Rust、.NET、Python、
JavaScript（Node.js）、Kotlin、Java、Swift、Ruby、Dart/Flutter 和 Go 提供了
符合语言习惯的绑定。它支持**三种熟悉的 API 风格**——Playwright、Puppeteer 和
Selenium——让你可以继续使用已有的代码，运行在更快的引擎之上。

三种风格都由声明式配置（profile）生成，并共享同一个引擎，因此你无需重写脚本，
就能切换风格（或语言）。此外还有原生的 `xcelerate` 风格，但它是一个**稍后的选项**：
请从你已经熟悉的风格开始。

## 支持的引擎

一个内核，两个引擎：基于 CDP 的 Chromium，基于 WebDriver BiDi 的 Firefox。

| 引擎 | 协议 | 后端 |
| --- | --- | --- |
| ![Chromium](https://img.shields.io/badge/Chromium-4285F4?logo=googlechrome&logoColor=white) Chromium、Chrome、Edge | Chrome DevTools Protocol（CDP） | `xcelerate::Browser` |
| ![Firefox](https://img.shields.io/badge/Firefox-FF7139?logo=firefoxbrowser&logoColor=white) Firefox | [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/) | `xcelerate::firefox` |

```rust
// Chromium (CDP)
let browser = xcelerate::Browser::launch(Default::default()).await?;

// Firefox (WebDriver BiDi)
let browser = xcelerate::firefox::FirefoxBrowser::launch(Default::default()).await?;
let page = browser.clone().new_page("https://example.com".to_string()).await?;
println!("{}", page.title().await?);
```

> **各界面支持的引擎。** CLI（`xcelerate run` / `session` / 一次性命令）和 XCL 运行器
> 只驱动 **基于 CDP 的 Chromium**；传入 Firefox 系列浏览器（`--browser firefox`）
> 会立即以清晰的错误信息失败。Firefox 可通过上面的 Rust API
> （`xcelerate::firefox::FirefoxBrowser`）使用。

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
| Dart / Flutter | `xcelerate` | [pub.dev](https://pub.dev/packages/xcelerate) |
| Go | `xcelerate` | 从源码构建（[说明](bindings/go/README.md)） |
| PowerShell | `Xcelerate` | [PowerShell Gallery](https://www.powershellgallery.com/packages/Xcelerate) |

## 功能特性

- **自动化的进程管理** —— 发现并启动 Chrome 或 Edge，并管理浏览器进程的生命周期。
- **安全优先的插件** —— 默认拒绝（default-deny）的插件系统，并带有仅追加
  （append-only）的审计日志。Xcelerate **不**附带任何内置插件；外部插件通过
  路径加载，其宿主调用经过沙箱化和按能力（capability）授权。
- **默认使用类人输入** —— 点击和打字会驱动真实的鼠标和键盘：光标沿贝塞尔
  路径移动到元素上，文本以人的节奏输入，因此运行过程看起来不那么机械。
- **异步优先** —— 在 Rust 中构建于 `tokio` 之上，并在每个绑定中使用
  `async`/`await`。
- **优先支持三种熟悉的 API 风格** —— 无需改动即可在同一引擎上运行 Playwright、
  Puppeteer 或 Selenium 代码，每种风格都由声明式配置（profile）生成。对于高级
  场景，还提供原生的 `xcelerate` API。
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
- **Agent 快照** —— 页面的索引化、面向 LLM 的渲染，可交互元素均可通过其
  `[index]` 进行操作。
- **CLI 与 MCP 服务器** —— `xcelerate` 命令用于一次性操作，`xcelerate mcp`
  （或 `xcelerate-mcp` 二进制文件）用于从 MCP 客户端驱动浏览器。
- **XCL 脚本** —— 一种面向行的 `.xcl` 脚本语言，人类和 AI 都能读写，具有
  变量、有界函数、控制流、HTTP `request`、插件 `run` 和 `assert`，并由默认拒绝
  （default-deny）安全机制支撑。

## 安装

### Rust

```toml
[dependencies]
xcelerate = "1.0.14"
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
    implementation("io.github.azzodude:xcelerate:1.0.14")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.14")   // Java
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

## 快速开始

选择你已经在写的 API 风格。所有风格都运行在同一个引擎上，因此你之后无需重写脚本
即可切换。

### Playwright 风格

```python
from xcelerate import use

pw = use("playwright")
browser = await pw.launch()
page = await browser.new_page()
await page.goto("https://example.com")
await page.click("#submit")
print(await page.inner_text("h1"))
await browser.close()
```

### Puppeteer 风格

```python
from xcelerate import use

pp = use("puppeteer")
browser = await pp.launch()
page = await browser.new_page()
await page.goto("https://example.com")
await page.click("#submit")
await browser.close()
```

### Selenium 风格

```python
from xcelerate import use

sel = use("selenium")
driver = await sel.launch()
await driver.get("https://example.com")
element = await driver.find_element("css selector", "#submit")
await element.click()
await driver.quit()
```

Rust 通过 `xcelerate::adapters::{playwright, puppeteer, selenium}` 访问同样的三种
风格。各风格的完整接口参见 [API 风格适配器](#api-风格适配器)。

### 原生 API（高级）

Xcelerate 也有自己的 API（`Browser`、`Page`、`Element`）——它是三种风格所构建于其上的
引擎。只有当某种风格未覆盖你的需求时才使用它；请从你已经熟悉的风格开始。

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

`BrowserConfig` 默认为无头（headless）且分离（detached）模式。你可以选择退出
这些默认设置，并在需要时通过路径加载插件（没有内置插件）：

```rust
use xcelerate::BrowserConfig;

let config = BrowserConfig {
    headless: false,
    detached: false,
    executable_path: None, // auto-discover Chrome/Edge
    plugins: None,         // 没有内置插件
    ..Default::default()
};
```

## 插件

Xcelerate 提供了**安全优先的插件系统**，且**不附带任何内置插件**。插件是启动时
配置、页面钩子和可调用操作的命名集合，除非你启用它，否则它不会执行任何操作
（**默认拒绝**）。你可以将其作为受信任的 Rust 库添加（`install_plugins`），
也可以作为通过路径加载的沙箱化 WebAssembly 组件添加（`load_plugin`）。

### 加载插件

从磁盘加载沙箱化的插件（一个目录或一个 `plugin.json`），然后通过每种语言中
相同的固定桥接接口调用它的操作：

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
browser.load_plugin("path/to/plugin".to_string())?; // …/plugin.json also works
let handle = browser.plugin("acme.hello".to_string())?;
handle.invoke("ping".into(), "{}".into()).await?;
```

### 内置目录

没有。`available_plugins()` 会报告此浏览器上已安装或加载的内容，在你添加之前
它是空的。类人的鼠标和键盘输入已内置于核心的输入路径中，而指纹相关工作应放在
你自己的插件里。

每个操作都在调用预算（invocation budget）下运行，并写入审计日志；插件只会
作用于交给它的页面。

### 浏览器与应用插件（共享插件主目录）

浏览器与原生应用控制都是**插件**。[`plugins/browser`](plugins/browser)
（`browser`）与 [`plugins/app`](plugins/app)（`app`）是现成的
WebAssembly 组件，把整套操作面暴露为 ops：

| 插件 | 暴露的能力 | Capability |
| --- | --- | --- |
| [`plugins/browser`](plugins/browser)（`browser`） | 浏览器操作面—— `open`、`click`、`fill`、`text`、`snapshot` 等 | `browser` |
| [`plugins/app`](plugins/app)（`app`） | 原生窗口控制—— `launch`、`window`、`tree`、`find`、`click`、`set_value` 等 | `app` |

两者都不直接使用 CDP、BiDi 或操作系统，而是调用**受能力门控的宿主桥接**
（`host.browser` / `host.app`）：插件向宿主请求一个语义动词，由宿主执行。如此，
浏览器与应用控制便与解释器解耦，核心保持精简。

**构建由 CLI 负责。** `xcelerate build --wasm-only` 会写入 `wit/plugin.wit`
（规范宿主 ABI）并把 `.wasm` 放到 `plugin.json` 旁边，因此插件无需手动复制或
维护接口：

```bash
cd plugins/browser && xcelerate build --wasm-only   # -> browser.wasm
```

**一次安装，按名使用。** 把构建好的插件放入用户全局目录
（`$XCELERATE_HOME/plugins`，否则 `~/.xcl/plugins`）或 `./plugins`，`xcelerate run`
会**自动加载**标准的 `browser` 与 `app` 插件。脚本随即使用**普通动词**，永不提及
插件名：

```bash
# 原生应用：`app` 插件会被自动加载；授予其能力以及窗口
XCELERATE_PLUGIN_ALLOW=app \
  xcelerate run --native --allow-app "Calculator" app.xcl
```

```xcl
# app.xcl - 插件只是这些动词背后的库
launch "calc" "Calculator"     # 启动（或附着）并选中它
window "Calculator"            # ……或选中一个已在运行的窗口
find "Equals"
click "Equals"
```

同样的工作也可直接通过插件的 ops 完成
（`run browser open {"url":"…"}`）；普通动词是更可取、与插件无关的操作面。

### 检查与调用插件

每个绑定都暴露同样小巧而固定的桥接接口，因此新增插件永远不需要新的绑定代码：

| 方法 | 用途 |
| --- | --- |
| `available_plugins()` | 此浏览器上已安装或加载的插件 |
| `plugin_names()` | 该浏览器上已启用的插件 |
| `use_plugin(name)` | 在运行时启用内置插件 |
| `install_plugins([plugin])` | 安装作为 Cargo 库编译进来的受信任插件（仅限 Rust） |
| `load_plugin(path)` | 加载沙箱化的插件（一个目录或 `plugin.json`） |
| `plugin(name)` | 指向已启用插件的句柄 |
| `plugin(name).ops()` | 插件所暴露的操作 |
| `plugin(name).invoke(op, args_json)` | 使用 JSON 参数调用操作，并返回 JSON |

```rust
let enabled = browser.plugin_names();              // e.g. ["acme.hello"]
let handle = browser.plugin("acme.hello".into())?; // error if not enabled
let info = handle.invoke("info".into(), "{}".into()).await?;
```

### 插件的运行位置及其可执行的操作

| 模型 | 运行位置 | 权限 |
| --- | --- | --- |
| 内置 | 进程内，已编译进去 | 启动标志、二进制修补、分离式启动、初始化脚本 |
| 从磁盘加载 | WebAssembly，沙箱化，按能力（capability）授权 | 默认拒绝的子集，经过审计 |

能力在被授予之前会先经过分类。`LaunchControl`、`BinaryPatch` 和
`DetachedSpawn` **仅限内置**；`Evaluate`、`CdpProxy`、`Browser`、`App`、cookie 访问、
初始化脚本、截图和网络捕获属于**危险**能力，需要明确同意。从磁盘加载的插件是一个
WebAssembly 组件，在沙箱中运行，**没有任何环境权限（ambient authority）**：
宿主（host）导入是唯一的出路，它们按能力授权，且每次调用都会被审计——因此
插件无法自行访问文件系统、网络或桌面。危险回调**默认被拒绝**，必须在每次调用时间
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

### 创建插件

插件是外部的 WebAssembly 组件（或可信的进程内 crate）。搭建一个入门插件、构建
`.wasm`，然后加载它：

```bash
xcelerate plugin new acme.hello          # 从模板搭建
cd hello && xcelerate build --wasm-only  # 写入 wit/plugin.wit 并构建 .wasm
```

你只需编写普通的 Rust 操作处理器；xcelerate 负责处理 WebAssembly 的底层管道，
并为你写入 `wit/plugin.wit`。
用 `Browser::load_plugin(path)`（一个目录或 `plugin.json`）加载构建好的组件，
或将其放入 `~/.xcl/plugins/` 并按名称加载。
参见[插件编写指南](docs/plugins/README.md)、[WASM 参考](docs/plugins/WASM.md)、
JSON [schema](docs/plugins/plugin.schema.json) 以及
[示例](docs/plugins/examples)。

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
    browser = await Browser.launch(BrowserConfig())
    page = await browser.new_page("https://example.com")
    print(await page.title())
    await browser.close()

asyncio.run(main())
```

JavaScript (Node.js)：

```javascript
const { Browser } = require("xcelerate");

async function main() {
    const browser = await Browser.launch();
    const page = await browser.newPage("https://example.com");
    console.log(await page.title());
    await browser.close();
}

main();
```

## 命令行界面

`xcelerate` 命令每次调用执行一个浏览器操作。它刻意保持精简：导航、将产物写入磁盘，以及管理安装。查看*实时*页面（标题、文本、HTML、媒体或索引化快照）由保持浏览器打开的交互式会话来负责。

```bash
xcelerate open https://example.com              # 导航；打印标题和 URL
xcelerate screenshot https://example.com -o shot.png --full
xcelerate pdf https://example.com -o page.pdf
xcelerate save https://example.com/logo.png -o logo.png
xcelerate grab https://…/playlist.m3u8 -o movie.mp4
xcelerate capture https://www.youtube.com/watch?v=… -o movie.mp4
xcelerate har https://example.com -o network.har
xcelerate record https://example.com -o video.mp4
xcelerate list                                  # 内置设备 + 插件
xcelerate plugins
xcelerate --device "iPhone 13" screenshot https://example.com -o phone.png
```

要查看页面，请打开会话并在其中运行命令：

```bash
xcelerate session
xcelerate> open example.com
xcelerate> title
xcelerate> text
xcelerate> snapshot        # 索引化，LLM 友好
xcelerate> media           # 以 JSON 输出图片/视频/音频
xcelerate> eval 'document.title'
```

全局标志适用于每个命令：`--headless`（默认显示浏览器窗口）、`--detached`、
`--executable-path <path>`、`--plugins <path,...>`、`--device <name>` 和 `--timeout <ms>`。
`xcelerate --device <name> <command>` 会以内置移动设备渲染，`xcelerate list`
会列出所有设备和插件。
使用 `cargo install --path crates/xcelerate-cli` 安装（安装后的可执行文件名为
`xcelerate-cli`；发布归档和 winget 会将其命名为 `xcelerate`），或在 Windows 上使用
`winget install Chaosware.Xcelerate`；从代码检出运行时，请在任何命令前加上
`cargo run -p xcelerate-cli --`。

## 脚本（XCL）

XCL（`.xcl`）是一种小巧的、面向行的脚本语言，可将多步骤自动化运行记录到一个
文件中，**人类**和 **AI 智能体**都能读写它。一行 = 一个动作。它刻意*不是*
图灵完备的：循环是有界的，函数是扁平的且不递归，也没有表达式子语言 —— 因此
脚本可安全地生成、可安全地阅读，且不会让运行器挂起。

```bash
xcelerate run login.xcl                # run a script
xcelerate run login.xcl --param user=ada@example.com
xcelerate run login.xcl --allow-http   # enable `request` (browserless HTTP)
```

### 示例

```
# register-then-login.xcl
param base "https://www.practicesoftwaretesting.com"

func fill_field(id, value)
  fill $id $value
end

open $base
wait 2000
click "Register"
fill_field "#email" "ada@example.com"
fill_field "#password" "correct-horse-battery"
submit
wait-idle

assert url contains "/login"
request GET "{base}/api/health"
assert $STATUS == 200
done
```

### 命令

| 关键字 | 含义 |
| --- | --- |
| `# comment` | 整行注释（空行会被忽略）。 |
| `let <name> <value>` | 定义变量；以 `$name` 进行插值。名称后的 `=` 可选。 |
| `set <name> <value>` | 重新赋值变量。 |
| `param <name> [default]` | 声明运行时参数（可用 `--param k=v` 覆盖）。 |
| `func <name>(a, b)` … `end` | 具名、无返回值的可调用单元（深度 1，不递归）。 |
| `<name> <arg>…` | 调用上方定义的函数（`call` 关键字可选）。 |
| `open` / `goto` `<url>` | 导航。 |
| `back` `reload` `title` `url` `text` `markdown` `snapshot` | 读取页面。 |
| `click <index\|selector\|text>` `tap` `fill <sel> <text>` `type` `press` `submit` `hover` `scroll` | 交互。 |
| `wait <ms\|selector>` `wait-sec` `wait-min` `wait-hr` `wait-idle` `wait-stable` | 等待。 |
| `eval <js>` | 运行 JavaScript（需要 `--allow-unsafe`）。 |
| `request <METHOD> <url> [headers] [body]` | 无需浏览器的 HTTP（需要 `--allow-http`）。 |
| `import <id>` `run <plugin> <op> [json]` `plugins` `plugin-config <id>` | 插件 / worker。 |
| `repeat <n> …` `retry <n> …` `if-ok …` `if-fail …` `goto <label>` `label <name>` | 有界控制流。 |
| `assert <subject> <op> <value>` | 快速失败检查（`url`、`title`、`status`、`contains`、`==` 等）。 |
| `print <arg>...` | 将解析后的参数输出到标准输出（日志通道）。 |
| `done` / `quit` | 结束本次运行。 |

## MCP 服务器

`xcelerate-mcp` —— 也可以通过 `xcelerate mcp` 调用 —— 是一个基于 stdio 的
[Model Context Protocol](https://modelcontextprotocol.io) 服务器，因此
MCP 客户端可以驱动真实的浏览器。它暴露 40 个工具，涵盖导航、标题、页面内容、
截图、PDF、点击、打字、悬停、按键、查询、JavaScript 求值和插件调用。

```jsonc
{ "mcpServers": { "xcelerate": { "command": "xcelerate-mcp" } } }
```

使用环境变量进行配置：`XCELERATE_CHROME`（浏览器路径）、
`XCELERATE_HEADLESS`（`1`/`true`，默认）、`XCELERATE_DETACHED`（`1`/`true`）以及
`XCELERATE_PLUGINS`（以逗号分隔的外部插件路径）。

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
XCELERATE_PROXY=http://user:pass@proxy.example:8080 xcelerate open https://example.com
XCELERATE_PROXY_POOL=http://a:8080,http://b:8080 ./your-app      # round-robin

# CLI flag (repeatable)
xcelerate --proxy http://user:pass@proxy.example:8080 --proxy http://backup:8080 \
  open https://example.com
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
XCELERATE_USER_DATA_DIR=~/.xcelerate/profile xcelerate open https://example.com
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
它以 MCP 工具 `browser_accessibility`（以及 `xcelerate` 库）的形式暴露。

## Agent 快照

`page.agent_snapshot()` 将页面渲染为缩进文本，其中每个可交互元素都带有稳定的
`[index]` 标记：

```
[0]<link> "Home"
[1]<textbox> "Email" = "a@b.com"
[2]<button> "Sign in"
```

该快照完全在 Rust 中构建，只需在持久化的 CDP 会话上分别调用一次
`Accessibility.getFullAXTree` 和一次 `DOMSnapshot.captureSnapshot`，因此比在脚本
语言中序列化 DOM 更廉价、更可预测。把索引传给 `page.click_index(n)` 即可点击该
元素，无需重新解析 CSS 选择器（`page.snapshot_json()` 会返回带有 role、name、
bounds、selector 和 backend node id 的相同元素）；若能从 DOM 属性推导出可用的
CSS 选择器（`#id` 或 `[name="…"]`），元素还会带上它，因此既可按索引（`click 1`）
也可按选择器（`fill "#email" …`）操作。它以交互式会话（`snapshot`，然后 `click <index>` 或 `click '<selector>'`）以及 MCP 工具 `browser_snapshot`、
`browser_click_index` 的形式暴露。

## 工作区布局

```
xcelerate/
  crates/
    xcelerate-core/         # WebSocket transport and typed CDP command layer
    xcelerate-plugin/       # plugin trait, manifest, capabilities, audit, host interface
    xcelerate/              # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/      # uniffi bindgen helper binary
    xcelerate-interpreter/  # the XCL language: lexer, parser, engine, security, executor
    xcelerate-cli/          # CLI (binary `xcelerate`), incl. the XCL runner
    xcelerate-mcp/          # `xcelerate-mcp` Model Context Protocol server
    xcelerate-codegen/      # script + typed-binding code generation (11 languages)
    adapters/                 # adapter profiles, runtime, and generator inputs
  bindings/                 # generated Python/JS/C#/Kotlin/Java/Swift/Ruby/Dart/Go packages (+ PowerShell)
  docs/plugins/             # plugin authoring guide, JSON schema, examples
  docs/xcl.md               # the XCL scripting language reference
  plugins/                  # browser + app plugins (browser and native-app APIs as .wasm)
  scripts/                  # code generation, harvesting, and release tooling
```

插件 API —— `Plugin` trait、`Manifest`、能力、审计日志以及 `PageHost` 接口 ——
位于 `crates/xcelerate-plugin/`。**核心不内置任何插件**：插件是外部的 —— 要么是
通过 `load_plugin` 以沙箱方式加载的 `.wasm` 组件，要么是嵌入方在进程内安装的
crate。门面（facade）拥有 `PluginManager`，并将 `Page` 桥接到插件宿主接口，
因此插件永远不会接触原始页面或传输层。

XCL 脚本语言位于 `crates/xcelerate-cli/src/xcl/`（词法分析器、解析器、引擎、
运行时、安全，以及浏览器/插件/HTTP 执行器）。参见 [`docs/xcl.md`](docs/xcl.md)。

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
