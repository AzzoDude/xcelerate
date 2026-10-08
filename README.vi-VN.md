[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | **[Tiếng Việt](README.vi-VN.md)**

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

Xcelerate là một engine tự động hóa trình duyệt hiệu năng cao, nhẹ, với các binding
idiomatic cho Rust, .NET, Python, JavaScript (Node.js), Kotlin, Java, Swift, Ruby,
Dart/Flutter và Go. Nó hỗ trợ **ba kiểu API quen thuộc** — Playwright, Puppeteer và
Selenium — để bạn giữ nguyên đoạn mã mình vẫn viết, trên một engine nhanh hơn.

Cả ba kiểu đều được sinh từ các profile khai báo và dùng chung một engine, nên bạn có
thể chuyển đổi kiểu (hoặc ngôn ngữ) mà không phải viết lại script. Ngoài ra còn có kiểu
`xcelerate` gốc, nhưng đó là một **lựa chọn về sau**: hãy bắt đầu với kiểu bạn đã biết.

## Các engine được hỗ trợ

Một lõi, hai engine: Chromium qua CDP, Firefox qua WebDriver BiDi.

| Engine | Giao thức | Backend |
| --- | --- | --- |
| ![Chromium](https://img.shields.io/badge/Chromium-4285F4?logo=googlechrome&logoColor=white) Chromium, Chrome, Edge | Chrome DevTools Protocol (CDP) | `xcelerate::Browser` |
| ![Firefox](https://img.shields.io/badge/Firefox-FF7139?logo=firefoxbrowser&logoColor=white) Firefox | [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/) | `xcelerate::firefox` |

```rust
// Chromium (CDP)
let browser = xcelerate::Browser::launch(Default::default()).await?;

// Firefox (WebDriver BiDi)
let browser = xcelerate::firefox::FirefoxBrowser::launch(Default::default()).await?;
let page = browser.clone().new_page("https://example.com".to_string()).await?;
println!("{}", page.title().await?);
```

## Các binding

| Ngôn ngữ | Gói | Kho |
| --- | --- | --- |
| Rust | `xcelerate` | [crates.io](https://crates.io/crates/xcelerate) |
| Python | `xcelerate` | [PyPI](https://pypi.org/project/xcelerate/) |
| JavaScript | `xcelerate` | [npm](https://www.npmjs.com/package/xcelerate) |
| .NET | `Xcelerate` | [NuGet](https://www.nuget.org/packages/Xcelerate) |
| Kotlin | `io.github.azzodude:xcelerate` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate) |
| Java | `io.github.azzodude:xcelerate-java` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate-java) |
| Swift | `Xcelerate` | build từ mã nguồn ([readme](bindings/swift/README.md)) |
| Ruby | `xcelerate` | build từ mã nguồn ([readme](bindings/ruby/README.md)) |
| Dart / Flutter | `xcelerate` | [pub.dev](https://pub.dev/packages/xcelerate) |
| Go | `xcelerate` | build từ mã nguồn ([readme](bindings/go/README.md)) |
| PowerShell | `Xcelerate` | [PowerShell Gallery](https://www.powershellgallery.com/packages/Xcelerate) |

## Tính năng

- **Quản lý tiến trình tự động** - tự phát hiện và khởi chạy Chrome hoặc Edge, đồng
  thời quản lý vòng đời của tiến trình trình duyệt.
- **Plugin ưu tiên bảo mật** - hệ thống plugin từ chối theo mặc định (default-deny)
  với nhật ký kiểm toán chỉ ghi thêm (append-only). Các plugin tích hợp bạn có thể
  chọn tham gia: `stealth` và `human`.
- **Plugin stealth và human** - `stealth` áp dụng việc vá nhị phân (binary patching)
  và một payload JavaScript chạy lúc runtime giúp giảm dấu vết tự động hóa; `human`
  làm cho thao tác nhập liệu hành xử như người thật (chuột di chuyển theo đường
  Bezier, gõ theo nhịp, cuộn không đều). Cả hai đều được bật theo từng trình duyệt
  thông qua `BrowserConfig.plugins`.
- **Ưu tiên async** - xây dựng trên `tokio` trong Rust và `async`/`await` trong mọi
  binding.
- **Ưu tiên ba kiểu API quen thuộc** - viết mã Playwright, Puppeteer hoặc Selenium
  nguyên vẹn trên cùng một engine, mỗi kiểu được sinh từ một profile khai báo. Có sẵn
  API `xcelerate` gốc cho các trường hợp nâng cao.
- **Binding đa ngôn ngữ** - một lõi duy nhất, các binding được sinh cho Rust, Python,
  JavaScript (Node.js), .NET, Kotlin, Java, Swift, Ruby, Dart/Flutter và Go thông qua
  `uniffi`, cùng một module PowerShell trên .NET SDK.
- **Ghi video phiên làm việc** - ghi lại trang thành video thông qua screencast của
  CDP; ghi định dạng Motion-JPEG AVI mà không cần công cụ bên ngoài, hoặc H.264/VP9
  MP4/WebM khi `ffmpeg` có trên `PATH`.
- **Pool proxy** - định tuyến trình duyệt qua một hoặc nhiều proxy HTTP thượng nguồn
  (kèm thông tin xác thực) thông qua một gateway cục bộ tích hợp.
- **Profile bền vững** - giữ lại thông tin đăng nhập, cookie và dữ liệu lưu trữ của
  trang giữa các lần chạy bằng một `user-data-dir` lâu dài.
- **Ảnh chụp nhanh accessibility** - một góc nhìn ngữ nghĩa `role`/`name` của trang
  để có các selector vững chắc và tự động hóa do agent điều khiển.
- **Ảnh chụp nhanh cho agent** - bản hiển thị trang có chỉ mục, thân thiện với LLM,
  nơi mọi phần tử tương tác đều có thể được thao tác bằng `[index]`.
- **CLI và máy chủ MCP** - lệnh `xcelerate` cho các tác vụ một lần, và `xcelerate mcp`
  (hoặc binary `xcelerate-mcp`) để điều khiển trình duyệt từ một client MCP.
- **XCL** - một ngôn ngữ kịch bản `.xcl` theo hướng dòng mà cả con
  người lẫn AI đều có thể đọc và viết, với các biến, hàm có giới hạn, luồng
  điều khiển, HTTP `request`, plugin `run` và `assert`, được hỗ trợ bởi bảo mật
  default-deny.

## Cài đặt

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

Được phát hành lên Maven Central với tên `io.github.azzodude:xcelerate` (Kotlin) và
`io.github.azzodude:xcelerate-java` (Java):

```kotlin
// build.gradle.kts
dependencies {
    implementation("io.github.azzodude:xcelerate:1.0.14")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.14")   // Java
}
```

Các binding Java yêu cầu **JDK 22+** và chạy với `--enable-native-access=ALL-UNNAMED`.
Xem [`bindings/kotlin/README.md`](bindings/kotlin/README.md) và
[`bindings/java/README.md`](bindings/java/README.md) để biết cách sử dụng.

Build chúng từ mã nguồn:

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_bindings/kotlin.py  # Kotlin sources + Gradle build
python scripts/generate_bindings/java.py    # Java sources + Gradle build
```

### Swift / Ruby / Dart / Go

Các target này được build từ mã nguồn bằng universal generator. Swift và Ruby là các
target UniFFI tích hợp sẵn; Dart cần `uniffi-bindgen-dart`; Go cần generator
`uniffi-bindgen-go` của NordSecurity và một toolchain Go.

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

PowerShell không có generator UniFFI, nên module này bọc .NET SDK. Để build và đóng
gói payload từ mã nguồn thay thế:

```powershell
python scripts/generate_bindings/powershell.py
Import-Module ./bindings/powershell/Xcelerate.psd1
```

Xem [`bindings/powershell/README.md`](bindings/powershell/README.md) để biết cách sử
dụng, hoặc chạy ví dụ end-to-end tại
[`examples/powershell/quickstart.ps1`](examples/powershell/quickstart.ps1).

Xem [`bindings/`](bindings/) để biết README của từng gói.

## Bắt đầu nhanh

Chọn kiểu API bạn vốn đã viết. Mọi kiểu đều chạy trên cùng một engine, nên bạn có thể
chuyển đổi sau mà không phải viết lại script.

### Kiểu Playwright

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

### Kiểu Puppeteer

```python
from xcelerate import use

pp = use("puppeteer")
browser = await pp.launch()
page = await browser.new_page()
await page.goto("https://example.com")
await page.click("#submit")
await browser.close()
```

### Kiểu Selenium

```python
from xcelerate import use

sel = use("selenium")
driver = await sel.launch()
await driver.get("https://example.com")
element = await driver.find_element("css selector", "#submit")
await element.click()
await driver.quit()
```

Rust tiếp cận cùng ba kiểu này thông qua
`xcelerate::adapters::{playwright, puppeteer, selenium}`. Xem
[Adapter kiểu API](#adapter-kiểu-api) để biết toàn bộ bề mặt của mỗi kiểu.

### API gốc (nâng cao)

Xcelerate còn có API riêng (`Browser`, `Page`, `Element`) - chính là engine mà ba kiểu
được xây dựng trên đó. Chỉ dùng đến nó khi một kiểu không đáp ứng nhu cầu của bạn; hãy
bắt đầu với kiểu bạn đã biết.

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

Theo mặc định, `BrowserConfig` dùng chế độ headless và detached, **không** có stealth.
Hãy bật plugin một cách tường minh, hoặc tắt các mặc định khác:

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

## Plugin

Xcelerate cung cấp một **hệ thống plugin ưu tiên bảo mật**. Một plugin là một gói
được đặt tên gồm cấu hình lúc khởi chạy, các page hook và các thao tác có thể gọi,
và nó không làm gì cả trừ khi bạn bật nó (**từ chối theo mặc định**). `stealth` là
plugin tích hợp được xây dựng trên hệ thống này.

### Bật plugin

Các plugin được liệt kê trong `BrowserConfig.plugins` và được bật trước khi trình
duyệt khởi chạy, nên chúng có thể đóng góp vào chính quá trình khởi chạy (ví dụ:
vá nhị phân):

```rust
use xcelerate::{Browser, BrowserConfig};

let config = BrowserConfig {
    plugins: Some(vec!["stealth".to_string()]),
    ..Default::default()
};
let browser = Browser::launch(config).await?;
```

Danh sách tương tự cũng đi qua `BrowserConfig` trong mọi ngôn ngữ:

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

### Danh mục tích hợp

| Plugin | Chức năng | Thao tác |
| --- | --- | --- |
| `stealth` | Vá nhị phân trình duyệt lúc khởi chạy và chèn payload chống dấu vết (anti-fingerprint) vào mọi document. | `info` |
| `human` | Nhập liệu như người thật: chuột di chuyển theo đường Bezier kèm dao động nhỏ, các cú nhấp dừng và giữ, độ trễ gõ theo từng phím, các bước cuộn không đều. | `info`, `move`, `click`, `type`, `scroll`, `delay` |

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

Mỗi op chạy trong hạn mức (budget) của lần gọi và được ghi vào nhật ký kiểm toán;
một plugin chỉ tác động lên những page mà nó được giao.

### Kiểm tra và gọi plugin

Mọi binding đều cung cấp cùng một cầu nối nhỏ gọn, cố định, nên một plugin mới không
bao giờ cần mã binding mới:

| Phương thức | Mục đích |
| --- | --- |
| `available_plugins()` | Tên của các plugin trong danh mục tích hợp đã được biên dịch sẵn |
| `plugin_names()` | Các plugin được bật trên trình duyệt này |
| `use_plugin(name)` | Bật một plugin tích hợp lúc runtime |
| `install_plugins([plugin])` | Cài đặt các plugin tin cậy được biên dịch sẵn dưới dạng thư viện Cargo (chỉ Rust) |
| `load_plugin(path)` | Tải một plugin chạy trong sandbox (một thư mục hoặc `plugin.json`) |
| `plugin(name)` | Một handle tới một plugin đã được bật |
| `plugin(name).ops()` | Các thao tác mà plugin cung cấp |
| `plugin(name).invoke(op, args_json)` | Gọi một thao tác với các đối số JSON, trả về JSON |

```rust
let enabled = browser.plugin_names();            // e.g. ["stealth", "human"]
let catalog = browser.available_plugins();       // ["stealth", "human"]
let stealth = browser.plugin("stealth".into())?; // error if not enabled
let info = stealth.invoke("info".into(), "{}".into()).await?;
```

### Plugin chạy ở đâu và chúng được phép làm gì

| Mô hình | Nơi chạy | Quyền hạn |
| --- | --- | --- |
| Tích hợp | Trong tiến trình, được biên dịch sẵn | Cờ khởi chạy, vá nhị phân, spawn tách rời, script khởi tạo |
| Tải từ đĩa | WebAssembly, chạy trong sandbox, bị giới hạn bởi capability | Tập con từ chối theo mặc định, được kiểm toán |

Các capability được phân loại trước khi chúng có thể được cấp. `LaunchControl`,
`BinaryPatch` và `DetachedSpawn` **chỉ dành cho plugin tích hợp**; `Evaluate`,
`CdpProxy`, truy cập cookie, script khởi tạo, ảnh chụp màn hình và bắt lưu lượng
mạng là **nguy hiểm** và yêu cầu sự đồng ý tường minh. Một plugin tải từ đĩa là một
component WebAssembly chạy trong sandbox với **không có quyền hạn xung quanh (ambient
authority)**: các import từ host là lối thoát duy nhất, chúng bị giới hạn bởi
capability, và mọi lời gọi đều được kiểm toán - nên một plugin không thể tự truy cập
hệ thống tệp hay mạng. Các callback nguy hiểm **bị từ chối theo mặc định** và phải
được chọn tham gia theo từng host thông qua `XCELERATE_PLUGIN_ALLOW` (theo từng
plugin, hoặc phạm vi rộng), trong hạn mức thời gian và kích thước phản hồi cho mỗi
lần gọi. Xem [`docs/plugins/`](docs/plugins/README.md).

### Nhật ký kiểm toán

Mọi hành động đặc quyền (cấu hình khởi chạy, page hook và mọi `invoke`) đều được ghi
vào một nhật ký kiểm toán chỉ ghi thêm, được liên kết bằng hash. Các bí mật như cookie
và thông tin xác thực không bao giờ được ghi vào đó.

```rust
assert!(browser.audit_verify()); // the hash chain is intact
println!("{}", browser.audit_log());
```

Các plugin tải từ đĩa vẫn thuộc trách nhiệm tin cậy của người dùng: nhiệm vụ của
engine là làm cho những gì chúng *có thể* làm trở nên tường minh, có thể kiểm toán và
bất khả thi theo mặc định.

### Tạo một plugin

Plugin là một thành phần WebAssembly bên ngoài (hoặc một crate trong tiến trình đáng
tin cậy). Tạo bộ khung, build `.wasm`, rồi nạp nó:

```bash
xcelerate plugin new acme.hello      # tạo khung từ template
cd hello && ./build.sh               # Windows:  .\build.ps1
```

Bạn viết các op handler bằng Rust thuần; xcelerate lo phần kết nối WebAssembly.
Nạp thành phần đã build bằng `Browser::load_plugin(path)` (một thư mục hoặc
`plugin.json`). Xem [hướng dẫn viết plugin](docs/plugins/README.md),
[tham chiếu WASM](docs/plugins/WASM.md), JSON
[schema](docs/plugins/plugin.schema.json), và
[ví dụ](docs/plugins/examples).

## Adapter kiểu API

Lớp adapter cung cấp các tên phương thức quen thuộc của Selenium, Playwright và
Puppeteer trên nền engine gốc, nên các script hiện có có thể được chuyển đổi với ít
thay đổi. Mỗi adapter được sinh ra từ một profile khai báo và được định kiểu đầy đủ
trong Rust và Python.

Rust:

```rust
use xcelerate::adapters::playwright;

let browser = playwright::launch(None).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

let paragraphs = page.query_selector_all("p".to_string()).await?;
println!("{} paragraph(s)", paragraphs.len());

browser.close().await?;
```

Python:

```python
from xcelerate import use

pw = use("playwright")
browser = await pw.launch()
page = await browser.new_page()
await page.goto("https://example.com")
print(await page.title())
await browser.close()
```

Cùng engine đó cũng có sẵn thông qua `use("selenium")` và `use("puppeteer")`. Xem
[`adapters/README.md`](adapters/README.md) để biết định dạng profile và cách mở rộng
một adapter.

## Binding gốc

Các binding được sinh ra cung cấp cùng một API bất đồng bộ trong mỗi ngôn ngữ.

Python:

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

JavaScript (Node.js):

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

## Giao diện dòng lệnh

Lệnh `xcelerate` thực hiện một hành động trình duyệt mỗi lần gọi:

```bash
xcelerate title https://example.com
xcelerate screenshot https://example.com -o shot.png --full
xcelerate query https://example.com h1 --attr href
xcelerate query-all https://example.com 'a'   # text of every match
xcelerate evaluate https://example.com 'document.title'
xcelerate list                                  # built-in devices + plugins
xcelerate --device "iPhone 13" screenshot https://example.com -o phone.png
xcelerate plugins
xcelerate snapshot https://example.com          # ảnh chụp nhanh có chỉ mục, thân thiện LLM
xcelerate click-index https://example.com 2     # nhấp phần tử [2] từ ảnh chụp nhanh
```

Các cờ toàn cục áp dụng cho mọi lệnh: `--no-headless`, `--detached`,
`--executable-path <path>`, `--plugins stealth,human`, `--device <name>`, và `--timeout <ms>`.
`xcelerate --device <name> <command>` hiển thị như một thiết bị di động có sẵn, và
`xcelerate list` liệt kê mọi thiết bị và plugin. Cài đặt
nó bằng `cargo install --path crates/xcelerate-cli` (binary được cài đặt tên là
`xcelerate-cli`; các bản phát hành và winget phân phối nó với tên `xcelerate`), hoặc
`winget install Chaosware.Xcelerate` trên Windows; khi làm việc từ bản checkout, thêm
tiền tố `cargo run -p xcelerate-cli --` trước mọi lệnh.

## XCL

XCL (`.xcl`) là một ngôn ngữ kịch bản nhỏ, theo hướng dòng, ghi lại một lượt tự
động hóa nhiều bước trong một tệp mà cả **con người** lẫn **AI agent** đều
có thể đọc và viết. Một dòng = một hành động. Nó được thiết kế có chủ đích là
*không* Turing-complete: vòng lặp có giới hạn, hàm phẳng và không đệ quy, và
không có tiểu ngôn ngữ biểu thức — nên một kịch bản là an toàn để phát ra,
an toàn để đọc, và không thể làm treo trình chạy.

```bash
xcelerate run login.xcl                # run a script
xcelerate run login.xcl --param user=ada@example.com
xcelerate run login.xcl --allow-http   # enable `request` (browserless HTTP)
```

### Ví dụ

```
# register-then-login.xcl
param base "https://www.practicesoftwaretesting.com"

func fill_field(id, value)
  fill $id $value
end

open $base
wait 2s
click-text "Register"
fill_field "#email" "ada@example.com"
fill_field "#password" "correct-horse-battery"
submit
wait-idle

assert url contains "/login"
request GET "{base}/api/health"
assert $STATUS == 200
done
```

### Lệnh

| Keyword | Ý nghĩa |
| --- | --- |
| `# comment` | Chú thích toàn dòng (bỏ qua dòng trống). |
| `let <name> <value>` | Định nghĩa một biến; nội suy thành `$name`. Dấu `=` sau tên là tùy chọn. |
| `set <name> <value>` | Gán lại một biến. |
| `param <name> [default]` | Khai báo một tham số runtime (ghi đè bằng `--param k=v`). |
| `func <name>(a, b)` … `end` | Một hàm có tên, không trả về, có thể gọi (độ sâu 1, không đệ quy). |
| `<name> <arg>…` | Gọi một hàm đã định nghĩa ở trên (`call` là tùy chọn). |
| `open` / `goto` `<url>` | Điều hướng. |
| `back` `reload` `title` `url` `text` `markdown` `snapshot` | Đọc trang. |
| `click <index\|selector>` `click-text <text>` `tap` `fill <sel> <text>` `type` `press` `submit` `hover` `scroll` | Tương tác. |
| `wait <ms\|s\|selector>` `wait-idle` `wait-stable` | Chờ. |
| `eval <js>` | Chạy JavaScript (yêu cầu `--allow-unsafe`). |
| `request <METHOD> <url> [headers] [body]` | HTTP không cần trình duyệt (yêu cầu `--allow-http`). |
| `import <id>` `run <plugin> <op> [json]` `plugins` `plugin-config <id>` | Plugin / worker. |
| `repeat <n> …` `retry <n> …` `if-ok …` `if-fail …` `goto <label>` `label <name>` | Luồng điều khiển có giới hạn. |
| `assert <subject> <op> <value>` | Kiểm tra fail-fast (`url`, `title`, `status`, `contains`, `==`, …). |
| `done` / `quit` | Kết thúc lượt chạy. |

## Máy chủ MCP

`xcelerate-mcp` - cũng có thể truy cập qua `xcelerate mcp` - là một máy chủ
[Model Context Protocol](https://modelcontextprotocol.io) qua stdio, để một client
MCP có thể điều khiển một trình duyệt thật. Nó cung cấp 40 công cụ bao gồm điều
hướng, tiêu đề, nội dung trang, ảnh chụp màn hình, PDF, nhấp chuột, gõ phím, hover,
nhấn phím, truy vấn, đánh giá JavaScript và gọi plugin.

```jsonc
{ "mcpServers": { "xcelerate": { "command": "xcelerate-mcp" } } }
```

Cấu hình nó bằng biến môi trường: `XCELERATE_CHROME` (đường dẫn trình duyệt),
`XCELERATE_HEADLESS` (`1`/`true`, mặc định), `XCELERATE_DETACHED` (`1`/`true`), và
`XCELERATE_PLUGINS` (đường dẫn plugin bên ngoài, phân tách bằng dấu phẩy).

## Ghi video

Một page có thể được ghi thành một tệp video. Việc ghi được điều khiển bởi screencast
của CDP, nên bản ghi dựa trên thay đổi: một trang có hoạt ảnh tạo ra chuyển động thật,
còn một trang tĩnh cho ra một clip ngắn.

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

page.start_video("demo.mp4".to_string()).await?;
tokio::time::sleep(std::time::Duration::from_secs(5)).await;
let path = page.stop_video().await?;   // the path actually written
```

Phần mở rộng của tệp đầu ra quyết định backend:

- `.mp4` / `.mov` / `.mkv` / `.webm` được mux bằng `ffmpeg` (H.264 hoặc VP9) theo mốc
  thời gian thật của các frame khi `ffmpeg` có trên `PATH`. Nếu thiếu, các frame sẽ
  chuyển sang dùng một tệp `.avi` cùng thư mục, và đường dẫn trả về cho biết đã dùng
  tệp nào.
- Bất kỳ phần mở rộng nào khác (ví dụ `.avi`) luôn dùng bộ ghi Motion-JPEG AVI tích
  hợp, vốn không cần công cụ bên ngoài.

`start_video_with_options` nhận các tham số tinh chỉnh (`quality`, `max_width`,
`max_height`, `fps`, `ffmpeg`). API này cố ý chưa được xuất qua UniFFI, nên nó có sẵn
trong Rust, CLI và máy chủ MCP mà không làm thay đổi checksum của các binding được
sinh ra.

Từ CLI:

```bash
xcelerate record https://example.com -o demo.mp4 --duration 5
```

Từ máy chủ MCP: `browser_start_recording {path}` … `browser_wait {milliseconds}` …
`browser_stop_recording`.

## Proxy

Một proxy được cấu hình sẽ áp dụng cho mọi page trong trình duyệt. `--proxy-server` của
Chrome không thể mang theo thông tin xác thực và không thể chuyển đổi proxy, nên
xcelerate chạy một gateway HTTP/CONNECT cục bộ nhỏ trên `127.0.0.1` và trỏ Chrome vào
đó; gateway này chuyển tiếp mỗi kết nối tới một upstream được chọn từ một **pool** và
chèn `Proxy-Authorization`.

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

- Các upstream có dạng `http://[user:pass@]host:port`; pool được luân chuyển theo kiểu
  round-robin.
- HTTPS dùng đường hầm `CONNECT` của proxy; HTTP thuần dùng các yêu cầu dạng
  absolute-form - cả hai đều đi qua upstream với thông tin xác thực được chèn.
- Các upstream SOCKS là không cần thiết: Chrome hỗ trợ SOCKS gốc qua `--proxy-server`,
  nên hãy trỏ Chrome trực tiếp vào đó.
- Các upstream `https://` (TLS tới proxy) chưa được hỗ trợ.

## Profile bền vững

Theo mặc định, mỗi trình duyệt nhận một profile tạm dùng một lần và bị xóa khi đóng.
Hãy trỏ nó tới một thư mục để giữ cookie, thông tin đăng nhập và dữ liệu lưu trữ của
trang giữa các lần chạy:

```bash
XCELERATE_USER_DATA_DIR=~/.xcelerate/profile xcelerate title https://example.com
xcelerate --user-data-dir ./profile title https://example.com
```

```rust
xcelerate::configure_user_data_dir(Some("./profile".to_string()))?;
```

Đường dẫn được chuẩn hóa (nếu không, một `--user-data-dir` tương đối sẽ được phân giải
dựa trên cwd của chính Chrome), và `Browser::close` cho phép Chrome ghi profile xuống
đĩa trước khi thoát.

## Ảnh chụp nhanh accessibility

`page.accessibility_snapshot()` trả về một góc nhìn ngữ nghĩa thu gọn của trang -
`[{ role, name, value? }]` theo thứ tự document - vốn vững chắc hơn nhiều so với các
selector CSS khi cần khẳng định (assert) hoặc điều khiển một trang. Nó được cung cấp
dưới dạng lệnh CLI `xcelerate accessibility <url>` và công cụ MCP
`browser_accessibility`.

## Ảnh chụp nhanh cho agent

`page.agent_snapshot()` hiển thị trang dưới dạng văn bản thụt lề, trong đó mỗi phần tử
có thể tương tác được gắn một `[index]` ổn định:

```
[0]<link> "Home"
[1]<textbox> "Email" = "a@b.com"
[2]<button> "Sign in"
```

Ảnh chụp nhanh được dựng hoàn toàn bằng Rust, chỉ với một lần gọi
`Accessibility.getFullAXTree` và một lần gọi `DOMSnapshot.captureSnapshot` trên phiên
CDP liên tục, nên rẻ và ổn định hơn nhiều so với việc tuần tự hóa DOM bằng ngôn ngữ
script. Truyền chỉ số cho `page.click_index(n)` để nhấp vào phần tử đó mà không cần
phân giải lại selector CSS (`page.snapshot_json()` trả về đúng các phần tử đó kèm
role, name, bounds và backend node id). Nó được cung cấp dưới dạng lệnh CLI
`xcelerate snapshot <url>`, `xcelerate click-index <url> <index>`, và công cụ MCP
`browser_snapshot`, `browser_click_index`.

## Bố cục workspace

```
xcelerate/
  crates/
    xcelerate-core/         # WebSocket transport and typed CDP command layer
    xcelerate-plugin/       # plugin trait, manifest, capabilities, audit, host interface
    xcelerate/              # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/      # uniffi bindgen helper binary
    xcelerate-cli/          # CLI (binary `xcelerate`), incl. the XCL runner + interpreter
    xcelerate-mcp/          # `xcelerate-mcp` Model Context Protocol server
    xcelerate-codegen/      # script + typed-binding code generation (11 languages)
  adapters/                 # adapter profiles, runtime, and generator inputs
  bindings/                 # generated Python/JS/C#/Kotlin/Java/Swift/Ruby/Dart/Go packages (+ PowerShell)
  docs/plugins/             # plugin authoring guide, JSON schema, examples
  docs/xcl.md               # the XCL scripting language reference
  scripts/                  # code generation, harvesting, and release tooling
```

API plugin - trait `Plugin`, `Manifest`, các capability, nhật ký kiểm toán và
giao diện `PageHost` - nằm trong `crates/xcelerate-plugin/`.
**Không có plugin nào được tích hợp vào core**: plugin là bên ngoài — hoặc là
các thành phần `.wasm` được nạp trong sandbox qua `load_plugin`, hoặc là các
crate trong tiến trình do bên nhúng cài đặt. Facade sở hữu `PluginManager` và
làm cầu nối giữa `Page` với giao diện host của plugin, nên plugin không bao giờ
chạm vào một page thô hay transport.

Ngôn ngữ kịch bản XCL nằm trong `crates/xcelerate-cli/src/xcl/` (lexer,
parser, engine, runtime, bảo mật và bộ thực thi trình duyệt/plugin/HTTP). Xem
[`docs/xcl.md`](docs/xcl.md).

## Phát triển

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

Các bài kiểm thử end-to-end khởi chạy một trình duyệt thật và yêu cầu Chrome hoặc
Edge. Hãy trỏ chúng tới một trang hoặc trình duyệt khác bằng `XCELERATE_TEST_URL` và
`XCELERATE_CHROME`.

## Bảo mật

Phát hiện lỗ hổng bảo mật? Vui lòng làm theo [SECURITY.md](SECURITY.md) - không mở
một issue công khai. Các phiên bản được hỗ trợ, kênh báo cáo và phạm vi được liệt kê
tại đó.

## License

Được cấp phép theo một trong hai giấy phép sau:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

theo lựa chọn của bạn.

Trừ khi bạn tuyên bố rõ ràng điều ngược lại, mọi đóng góp mà bạn cố ý gửi để đưa vào
dự án này, như được định nghĩa trong giấy phép Apache-2.0, sẽ được cấp phép kép như
trên, không kèm theo bất kỳ điều khoản hay điều kiện bổ sung nào.
