[English](README.md) | [简体中文](README.zh-CN.md) | **[日本語](README.ja-JP.md)** | [Tiếng Việt](README.vi-VN.md)

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

Xcelerate は、高性能で軽量なブラウザ自動化エンジンであり、Rust、.NET、Python、
JavaScript（Node.js）、Kotlin、Java、Swift、Ruby、Dart/Flutter、Go 向けの
慣用的なバインディングを備えています。**3 つのなじみ深い API スタイル** ——
Playwright、Puppeteer、Selenium —— を話すため、すでに書いているコードをそのまま、
より高速なエンジン上で使えます。

3 つのスタイルはすべて宣言的なプロファイルから生成され、1 つのエンジンを共有する
ため、スクリプトを書き直すことなくスタイル（や言語）を切り替えられます。ネイティブの
`xcelerate` スタイルも存在しますが、それは**後から使う選択肢**です。まずはすでに
知っているスタイルから始めてください。

## 対応エンジン

1 つのコア、2 つのエンジン: CDP 経由の Chromium、WebDriver BiDi 経由の Firefox。

| エンジン | プロトコル | バックエンド |
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

> **サーフェスごとのエンジン対応。** CLI（`xcelerate run` / `session` / 単発コマンド）
> と XCL ランナーは **CDP 経由の Chromium** のみを駆動します。Firefox 系
> （`--browser firefox`）を指定すると、明確なエラーで即座に失敗します。Firefox は
> 上記の Rust API（`xcelerate::firefox::FirefoxBrowser`）から利用できます。

## バインディング

| 言語 | パッケージ | レジストリ |
| --- | --- | --- |
| Rust | `xcelerate` | [crates.io](https://crates.io/crates/xcelerate) |
| Python | `xcelerate` | [PyPI](https://pypi.org/project/xcelerate/) |
| JavaScript | `xcelerate` | [npm](https://www.npmjs.com/package/xcelerate) |
| .NET | `Xcelerate` | [NuGet](https://www.nuget.org/packages/Xcelerate) |
| Kotlin | `io.github.azzodude:xcelerate` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate) |
| Java | `io.github.azzodude:xcelerate-java` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate-java) |
| Swift | `Xcelerate` | ソースからビルド（[README](bindings/swift/README.md)） |
| Ruby | `xcelerate` | ソースからビルド（[README](bindings/ruby/README.md)） |
| Dart / Flutter | `xcelerate` | [pub.dev](https://pub.dev/packages/xcelerate) |
| Go | `xcelerate` | ソースからビルド（[README](bindings/go/README.md)） |
| PowerShell | `Xcelerate` | [PowerShell Gallery](https://www.powershellgallery.com/packages/Xcelerate) |

## 機能

- **プロセスの自動管理** - Chrome または Edge を検出して起動し、ブラウザプロセスの
  ライフサイクルを管理します。
- **セキュリティ第一のプラグイン** - 追記専用の監査ログを備えた、デフォルト拒否の
  プラグインシステム。Xcelerate は組み込みプラグインを**一切**同梱せず、外部の
  プラグインはパスで読み込み、サンドボックス化されたケイパビリティ制限付きのホスト
  呼び出しを行います。
- **既定で人間らしい入力** - クリックとタイピングは実際のマウスとキーボードを駆動
  します。カーソルはベジェ曲線に沿って要素まで移動し、テキストは人間らしいペースで
  入力されるため、実行が機械的に見えにくくなります。
- **async ファースト** - Rust では `tokio`、すべてのバインディングでは
  `async`/`await` を基盤としています。
- **なじみ深い 3 つの API スタイルを最優先** - Playwright、Puppeteer、Selenium の
  コードを変更なしで同じエンジン上で実行でき、各スタイルは宣言的なプロファイルから
  生成されます。上級者向けにネイティブの `xcelerate` API も利用できます。
- **多言語バインディング** - 単一のコアから、`uniffi` 経由で Rust、Python、
  JavaScript（Node.js）、.NET、Kotlin、Java、Swift、Ruby、Dart/Flutter、Go 向けの
  バインディングを生成。加えて .NET SDK 上の PowerShell モジュールも提供します。
- **セッションの動画録画** - CDP スクリーンキャストを通じてページを動画として
  キャプチャします。外部ツールなしで Motion-JPEG AVI を書き出し、`ffmpeg` が
  `PATH` にある場合は H.264/VP9 の MP4/WebM を書き出します。
- **プロキシプール** - 組み込みのローカルゲートウェイを介して、ブラウザを 1 つ以上の
  上流 HTTP プロキシ（認証情報付き）経由でルーティングします。
- **永続プロファイル** - 永続的な `user-data-dir` により、実行間でログイン、Cookie、
  サイトストレージを保持します。
- **アクセシビリティスナップショット** - 堅牢なセレクターとエージェント駆動の
  自動化のための、ページの意味的な `role`/`name` ビュー。
- **エージェントスナップショット** - 操作可能なすべての要素を `[index]` で操作
  できる、インデックス付きの LLM 向けページ描画。
- **CLI と MCP サーバー** - 単発のアクション向けの `xcelerate` コマンド、および
  MCP クライアントからブラウザを操作するための `xcelerate mcp`（または
  `xcelerate-mcp` バイナリ）。
- **XCL スクリプト** - 人間と AI の両方が読み書きできる、行指向の `.xcl` スクリプト
  言語。変数、有界の関数、制御フロー、HTTP `request`、プラグイン `run`、`assert`
  を備え、デフォルト拒否のセキュリティに支えられています。

## インストール

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

Maven Central に `io.github.azzodude:xcelerate`（Kotlin）および
`io.github.azzodude:xcelerate-java`（Java）として公開されています:

```kotlin
// build.gradle.kts
dependencies {
    implementation("io.github.azzodude:xcelerate:1.0.14")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.14")   // Java
}
```

Java バインディングには **JDK 22+** が必要で、`--enable-native-access=ALL-UNNAMED`
を付けて実行します。使用方法については
[`bindings/kotlin/README.md`](bindings/kotlin/README.md) と
[`bindings/java/README.md`](bindings/java/README.md) を参照してください。

ソースからビルドする場合:

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_bindings/kotlin.py  # Kotlin sources + Gradle build
python scripts/generate_bindings/java.py    # Java sources + Gradle build
```

### Swift / Ruby / Dart / Go

これらのターゲットは、汎用ジェネレーターを使用してソースからビルドします。Swift と
Ruby は UniFFI の組み込みターゲットです。Dart には `uniffi-bindgen-dart` が必要で、
Go には NordSecurity の `uniffi-bindgen-go` ジェネレーターと Go ツールチェーンが
必要です。

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

PowerShell には UniFFI ジェネレーターがないため、このモジュールは .NET SDK を
ラップします。代わりにソースからペイロードをビルドして配置する場合:

```powershell
python scripts/generate_bindings/powershell.py
Import-Module ./bindings/powershell/Xcelerate.psd1
```

使用方法については [`bindings/powershell/README.md`](bindings/powershell/README.md)
を参照するか、[`examples/powershell/quickstart.ps1`](examples/powershell/quickstart.ps1)
のエンドツーエンドの例を実行してください。

各パッケージの README については [`bindings/`](bindings/) を参照してください。

## クイックスタート

すでに書いている API スタイルを選んでください。どのスタイルも同じエンジン上で
動くため、後からスクリプトを書き直すことなく切り替えられます。

### Playwright スタイル

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

### Puppeteer スタイル

```python
from xcelerate import use

pp = use("puppeteer")
browser = await pp.launch()
page = await browser.new_page()
await page.goto("https://example.com")
await page.click("#submit")
await browser.close()
```

### Selenium スタイル

```python
from xcelerate import use

sel = use("selenium")
driver = await sel.launch()
await driver.get("https://example.com")
element = await driver.find_element("css selector", "#submit")
await element.click()
await driver.quit()
```

Rust では `xcelerate::adapters::{playwright, puppeteer, selenium}` を通じて同じ
3 つのスタイルに到達できます。各スタイルの全サーフェスについては
[API スタイルのアダプター](#api-スタイルのアダプター) を参照してください。

### ネイティブ API（上級）

Xcelerate には独自の API（`Browser`、`Page`、`Element`）もあります - これは
3 つのスタイルがその上に構築されるエンジンです。スタイルで要件を満たせない場合に
のみ手を伸ばしてください。まずはすでに知っているスタイルから始めてください。

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

`BrowserConfig` の既定はヘッドレスかつデタッチです。これらの既定をオプトアウトし、
必要であればプラグインをパスで読み込んでください（組み込みのプラグインはありません）:

```rust
use xcelerate::BrowserConfig;

let config = BrowserConfig {
    headless: false,
    detached: false,
    executable_path: None, // auto-discover Chrome/Edge/…
    plugins: None,         // 組み込みプラグインなし
    ..Default::default()
};
```

## プラグイン

Xcelerate は**セキュリティ第一のプラグインシステム**を提供し、組み込みプラグインは
**一切**同梱しません。プラグインとは、起動時の設定、ページフック、呼び出し可能な操作を
まとめた名前付きのバンドルであり、有効化しない限り何も行いません（**デフォルト拒否**）。
信頼された Rust ライブラリとして（`install_plugins`）、またはパスで読み込む
サンドボックス化された WebAssembly コンポーネントとして（`load_plugin`）追加します。

### プラグインの読み込み

ディスクからサンドボックス化されたプラグイン（ディレクトリまたは `plugin.json`）を
読み込み、すべての言語で共通の固定されたブリッジを通じてその操作を呼び出します:

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
browser.load_plugin("path/to/plugin".to_string())?; // …/plugin.json also works
let handle = browser.plugin("acme.hello".to_string())?;
handle.invoke("ping".into(), "{}".into()).await?;
```

### 組み込みカタログ

存在しません。`available_plugins()` はこのブラウザにインストールまたは読み込み済みの
プラグインを報告し、何かを追加するまで空です。人間らしいマウスとキーボードの入力は
コアの入力パスに組み込まれており、フィンガープリント関連の作業は自作のプラグインに
属します。

各操作は呼び出し予算の下で実行され、監査ログに記録されます。プラグインは、渡された
ページに対してのみ動作します。

### ブラウザとアプリのプラグイン（共有プラグインホーム）

[`plugins/`](plugins) ディレクトリには、モデル全体を端から端まで示す 2 つの
既製 WebAssembly プラグインがあります:

| プラグイン | 公開するもの | ケイパビリティ |
| --- | --- | --- |
| [`plugins/browser`](plugins/browser)（`browser`） | ブラウザ操作面 — `open`, `click`, `fill`, `text`, `snapshot` など | `browser` |
| [`plugins/app`](plugins/app)（`app`） | ネイティブウィンドウ操作 — `launch`, `tree`, `click`, `set_value` など | `app` |

どちらも CDP・BiDi・OS を直接扱わず、**ケイパビリティで制限されたホストブリッジ**
（`host.browser` / `host.app`）を呼び出します。プラグインは意味のある動詞をホストに
要求し、ホストがそれを実行します。こうしてブラウザとアプリの操作はインタプリタから
切り離され、コアは小さく保たれます。

一度ビルドして共有しましょう。素のプラグイン *名* は、ユーザーグローバルの
`$XCELERATE_HOME/plugins`（未設定なら `~/.xcl/plugins`）、次に `./plugins`、次に
`.` の順に解決されるため、1 回のビルドをどのプロジェクトからでも読み込めます:

```bash
cd plugins/browser && xcelerate build --wasm-only
mkdir -p ~/.xcl/plugins && cp -r . ~/.xcl/plugins/browser

# どこからでも。`browser` ケイパビリティは危険なので明示的に付与します
XCELERATE_PLUGIN_ALLOW=browser \
  xcelerate --plugins browser run --allow-plugin browser job.xcl
```

```xcl
# job.xcl
run browser open {"url":"https://example.com"}
run browser find {"text":"Example"}
```

### プラグインの検査と呼び出し

すべてのバインディングが同一の小さく固定されたブリッジを公開するため、新しい
プラグインが新しいバインディングコードを必要とすることはありません:

| メソッド | 目的 |
| --- | --- |
| `available_plugins()` | このブラウザにインストール/読み込み済みのプラグイン |
| `plugin_names()` | このブラウザで有効なプラグイン |
| `use_plugin(name)` | 実行時に組み込みプラグインを有効化 |
| `install_plugins([plugin])` | Cargo ライブラリとしてコンパイル済みの信頼済みプラグインをインストール（Rust のみ） |
| `load_plugin(path)` | サンドボックス化されたプラグイン（ディレクトリまたは `plugin.json`）をロード |
| `plugin(name)` | 有効なプラグインへのハンドル |
| `plugin(name).ops()` | プラグインが公開する操作 |
| `plugin(name).invoke(op, args_json)` | JSON 引数で操作を呼び出し、JSON を返す |

```rust
let enabled = browser.plugin_names();              // e.g. ["acme.hello"]
let handle = browser.plugin("acme.hello".into())?; // error if not enabled
let info = handle.invoke("info".into(), "{}".into()).await?;
```

### プラグインの実行場所と実行可能な内容

| モデル | 実行場所 | 権限 |
| --- | --- | --- |
| 組み込み | プロセス内、コンパイル済み | 起動フラグ、バイナリパッチ、デタッチ起動、init スクリプト |
| ディスクからロード | WebAssembly、サンドボックス化、ケイパビリティで制限 | デフォルト拒否のサブセット、監査対象 |

ケイパビリティは付与される前に分類されます。`LaunchControl`、`BinaryPatch`、
`DetachedSpawn` は**組み込み専用**です。`Evaluate`、`CdpProxy`、`Browser`、`App`、
Cookie アクセス、init スクリプト、スクリーンショット、ネットワークキャプチャは
**危険**であり、明示的な同意が必要です。ディスクからロードされたプラグインは、
**周囲の権限を持たず**サンドボックスで動作する WebAssembly コンポーネントです。
ホストインポートだけが唯一の出口であり、それらはケイパビリティで制限され、すべての
呼び出しが監査されます。したがって、プラグインが単独でファイルシステム、ネットワーク、
デスクトップに到達することはできません。
危険なコールバックは**既定で拒否**され、ホストごとに `XCELERATE_PLUGIN_ALLOW`
（プラグイン単位または広範に）を介して、呼び出しごとの時間と応答サイズの予算の下で
オプトインする必要があります。詳細は
[`docs/plugins/`](docs/plugins/README.md) を参照してください。

### 監査ログ

特権的な操作（起動設定、ページフック、すべての `invoke`）は、追記専用の
ハッシュチェーン化された監査ログに記録されます。Cookie や認証情報などの秘密情報が
書き込まれることはありません。

```rust
assert!(browser.audit_verify()); // the hash chain is intact
println!("{}", browser.audit_log());
```

ディスクからロードされたプラグインを信頼するかどうかはユーザーの責任です。
エンジンの役割は、それらが*できる*ことを明示的で監査可能にし、既定では不可能に
することです。

### プラグインの作成

プラグインは外部の WebAssembly コンポーネント（または信頼されたインプロセスの
crate）です。スターターをスキャフォールドし、`.wasm` をビルドしてからロードします:

```bash
xcelerate plugin new acme.hello      # テンプレートからスキャフォールド
cd hello && xcelerate build --wasm-only
```

プレーンな Rust の操作ハンドラーを書くだけで、xcelerate が WebAssembly の配線を
処理します。ビルドしたコンポーネントは `Browser::load_plugin(path)`（ディレクトリ
または `plugin.json`）でロードするか、`~/.xcl/plugins/` に置いて名前でロードします。
詳細は
[プラグイン作成ガイド](docs/plugins/README.md)、
[WASM リファレンス](docs/plugins/WASM.md)、JSON
[schema](docs/plugins/plugin.schema.json)、
[サンプル](docs/plugins/examples) を参照してください。

## API スタイルのアダプター

アダプターレイヤーは、ネイティブエンジンの上に使い慣れた Selenium、Playwright、
Puppeteer のメソッド名を公開するため、既存のスクリプトを最小限の変更で移植できます。
各アダプターは宣言的なプロファイルから生成され、Rust と Python では完全に型付け
されています。

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

同じエンジンは `use("selenium")` と `use("puppeteer")` からも利用できます。
プロファイル形式とアダプターの拡張方法については
[`adapters/README.md`](adapters/README.md) を参照してください。

## ネイティブバインディング

生成されたバインディングは、各言語で同じ非同期 API を公開します。

Python:

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

JavaScript (Node.js):

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

## コマンドラインインターフェース

`xcelerate` コマンドは、呼び出しごとに 1 つのブラウザアクションを実行します。意図的に小さく保たれています: ナビゲートし、成果物をディスクに書き込み、インストールを管理します。*ライブ*のページ（タイトル、テキスト、HTML、メディア、索引付きスナップショット）を調べるのは、ブラウザーを開いたままにする対話型セッションの役割です。

```bash
xcelerate open https://example.com              # ナビゲートしてタイトルと URL を表示
xcelerate screenshot https://example.com -o shot.png --full
xcelerate pdf https://example.com -o page.pdf
xcelerate save https://example.com/logo.png -o logo.png
xcelerate grab https://…/playlist.m3u8 -o movie.mp4
xcelerate capture https://www.youtube.com/watch?v=… -o movie.mp4
xcelerate har https://example.com -o network.har
xcelerate record https://example.com -o video.mp4
xcelerate list                                  # 組み込みデバイス + プラグイン
xcelerate plugins
xcelerate --device "iPhone 13" screenshot https://example.com -o phone.png
```

ページを調べるには、セッションを開き、そこでコマンドを実行します:

```bash
xcelerate session
xcelerate> open example.com
xcelerate> title
xcelerate> text
xcelerate> snapshot        # 索引付き、LLM 向け
xcelerate> media           # 画像/動画/音声を JSON で
xcelerate> eval 'document.title'
```

グローバルフラグはすべてのコマンドに適用されます: `--headless`（既定でブラウザー
ウィンドウを表示）、`--detached`、`--executable-path <path>`、`--plugins <path,...>`、
`--device <name>`、`--timeout <ms>`。
`xcelerate --device <name> <command>` は組み込みのモバイル デバイスとして描画し、
`xcelerate list` はすべてのデバイスとプラグインを一覧表示します。
インストールには `cargo install --path crates/xcelerate-cli`（インストールされる
バイナリ名は `xcelerate-cli`。リリースアーカイブと winget では `xcelerate` として
配布されます）、Windows では `winget install Chaosware.Xcelerate` を使用します。
チェックアウトから実行する場合は、任意のコマンドの前に
`cargo run -p xcelerate-cli --` を付けます。

## スクリプト（XCL）

XCL（`.xcl`）は、**人間**と **AI エージェント**の両方が読み書きできるファイルに、
複数ステップの自動化実行を記述する、小さな行指向のスクリプト言語です。1 行 = 1
アクション。意図的にチューリング完全では*ありません*: ループは有界で、関数は
フラットで再帰せず、式のサブ言語も存在しません。そのため、スクリプトは出力しても
安全、読んでも安全で、ランナーをハングさせることはありません。

```bash
xcelerate run login.xcl                # run a script
xcelerate run login.xcl --param user=ada@example.com
xcelerate run login.xcl --allow-http   # enable `request` (browserless HTTP)
```

### 例

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

### コマンド

| キーワード | 意味 |
| --- | --- |
| `# comment` | 行全体のコメント（空行は無視されます）。 |
| `let <name> <value>` | 変数を定義します。`$name` として展開します。名前の後の `=` は省略可能です。 |
| `set <name> <value>` | 変数を再代入します。 |
| `param <name> [default]` | 実行時パラメーターを宣言します（`--param k=v` で上書き）。 |
| `func <name>(a, b)` … `end` | 名前付きの、戻り値のない呼び出し可能関数（深さ 1、再帰なし）。 |
| `<name> <arg>…` | 上で定義した関数を呼び出します（`call` キーワードは省略可能）。 |
| `open` / `goto` `<url>` | ナビゲートします。 |
| `back` `reload` `title` `url` `text` `markdown` `snapshot` | ページを読み取ります。 |
| `click <index\|selector\|text>` `tap` `fill <sel> <text>` `type` `press` `submit` `hover` `scroll` | 操作します。 |
| `wait <ms\|selector>` `wait-sec` `wait-min` `wait-hr` `wait-idle` `wait-stable` | 待機します。 |
| `eval <js>` | JavaScript を実行します（`--allow-unsafe` が必要）。 |
| `request <METHOD> <url> [headers] [body]` | ブラウザなしの HTTP（`--allow-http` が必要）。 |
| `import <id>` `run <plugin> <op> [json]` `plugins` `plugin-config <id>` | プラグイン / ワーカー。 |
| `repeat <n> …` `retry <n> …` `if-ok …` `if-fail …` `goto <label>` `label <name>` | 有界の制御フロー。 |
| `assert <subject> <op> <value>` | フェイルファストチェック（`url`、`title`、`status`、`contains`、`==`、…）。 |
| `print <arg>...` | 解決した引数を標準出力に書き出します（ログ用のチャネル）。 |
| `done` / `quit` | 実行を終了します。 |

## MCP サーバー

`xcelerate-mcp` は - `xcelerate mcp` としても到達可能です - stdio 上の
[Model Context Protocol](https://modelcontextprotocol.io) サーバーであり、MCP
クライアントが実際のブラウザを操作できます。ナビゲーション、タイトル、ページ内容、
スクリーンショット、PDF、クリック、タイピング、ホバー、キー押下、クエリ、
JavaScript 評価、プラグイン呼び出しをカバーする 40 個のツールを公開します。

```jsonc
{ "mcpServers": { "xcelerate": { "command": "xcelerate-mcp" } } }
```

環境変数で設定します: `XCELERATE_CHROME`（ブラウザパス）、`XCELERATE_HEADLESS`
（`1`/`true`、既定）、`XCELERATE_DETACHED`（`1`/`true`）、`XCELERATE_PLUGINS`（カンマ区切りの外部プラグインパス）。

## 動画録画

ページを動画ファイルに録画できます。録画は CDP スクリーンキャストによって駆動される
ため、キャプチャは変化駆動型です。アニメーションするページは実際の動きを生成し、
静的なページは短いクリップになります。

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

page.start_video("demo.mp4".to_string()).await?;
tokio::time::sleep(std::time::Duration::from_secs(5)).await;
let path = page.stop_video().await?;   // the path actually written
```

出力の拡張子がバックエンドを選択します:

- `.mp4` / `.mov` / `.mkv` / `.webm` は、`ffmpeg` が `PATH` にある場合、フレームの
  実際のタイムスタンプで `ffmpeg`（H.264 または VP9）によって多重化されます。存在
  しない場合、フレームは兄弟の `.avi` にフォールバックし、返されるパスがどちらが
  使われたかを示します。
- その他の拡張子（たとえば `.avi`）は常に、外部ツールを必要としない組み込みの
  Motion-JPEG AVI ライターを使用します。

`start_video_with_options` は調整用パラメーター（`quality`、`max_width`、
`max_height`、`fps`、`ffmpeg`）を受け取ります。この API は意図的にまだ UniFFI を
通じてエクスポートされていないため、生成されたバインディングのチェックサムを変更
することなく、Rust、CLI、MCP サーバーで利用できます。

CLI から:

```bash
xcelerate record https://example.com -o demo.mp4 --duration 5
```

MCP サーバーから: `browser_start_recording {path}` … `browser_wait
{milliseconds}` … `browser_stop_recording`。

## プロキシ

設定されたプロキシはブラウザ内のすべてのページに適用されます。Chrome の
`--proxy-server` は認証情報を運べず、プロキシを切り替えることもできないため、
xcelerate は `127.0.0.1` 上で小さなローカル HTTP/CONNECT ゲートウェイを実行し、
Chrome をそこに向けます。ゲートウェイは各接続を**プール**から選ばれた上流に転送し、
`Proxy-Authorization` を注入します。

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

- 上流は `http://[user:pass@]host:port` です。プールはラウンドロビンでローテーション
  されます。
- HTTPS はプロキシの `CONNECT` トンネルを使用します。プレーンな HTTP は絶対形式の
  リクエストを使用します。どちらも認証情報が注入された上流を通過します。
- SOCKS の上流は不要です: Chrome は `--proxy-server` を介して SOCKS をネイティブに
  扱うため、Chrome を直接そこに向けてください。
- `https://` の上流（プロキシへの TLS）はまだサポートされていません。

## 永続プロファイル

既定では、各ブラウザに使い捨てのプロファイルが割り当てられ、終了時に削除されます。
ディレクトリを指定すると、実行間で Cookie、ログイン、サイトストレージを保持できます:

```bash
XCELERATE_USER_DATA_DIR=~/.xcelerate/profile xcelerate open https://example.com
xcelerate --user-data-dir ./profile title https://example.com
```

```rust
xcelerate::configure_user_data_dir(Some("./profile".to_string()))?;
```

パスは正規化されます（そうしないと、相対的な `--user-data-dir` は Chrome 自身の cwd
に対して解決されてしまいます）。また、`Browser::close` は Chrome が終了する前に
プロファイルをディスクにフラッシュできるようにします。

## アクセシビリティスナップショット

`page.accessibility_snapshot()` は、ページのコンパクトな意味的ビュー -
ドキュメント順の `[{ role, name, value? }]` - を返します。これは、ページの検証や
操作において CSS セレクターよりもはるかに堅牢です。MCP ツール
`browser_accessibility`（および `xcelerate` ライブラリ）として公開されています。

## エージェントスナップショット

`page.agent_snapshot()` は、ページをインデント付きテキストとして描画し、操作可能な
各要素に安定した `[index]` を付与します:

```
[0]<link> "Home"
[1]<textbox> "Email" = "a@b.com"
[2]<button> "Sign in"
```

このスナップショットは完全に Rust で構築され、永続化された CDP セッション上で
`Accessibility.getFullAXTree` と `DOMSnapshot.captureSnapshot` をそれぞれ 1 回呼ぶ
だけで済むため、スクリプト言語で DOM をシリアライズするよりもはるかに安価で予測
可能です。インデックスを `page.click_index(n)` に渡すと、CSS セレクターを再解決
せずにその要素をクリックできます（`page.snapshot_json()` は role、name、bounds、
selector、backend node id を持つ同じ要素を返します）。DOM 属性から使える CSS
セレクター（`#id` や `[name="…"]`）を導出できる場合はそれが添えられるので、
インデックス（`click 1`）でもセレクター（`fill "#email" …`）でも操作できます。
対話型セッション（`snapshot`、そして `click <index>` または `click '<selector>'`）、
および MCP ツール `browser_snapshot`、`browser_click_index` として公開されています。

## ワークスペース構成

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

プラグイン API - `Plugin` トレイト、`Manifest`、ケイパビリティ、監査ログ、
`PageHost` インターフェース - は `crates/xcelerate-plugin/` にあります。**コアに
プラグインは組み込まれていません**: プラグインは外部にあり、`load_plugin` 経由で
サンドボックス化してロードする `.wasm` コンポーネントか、埋め込み側がインストール
するインプロセスクレートのいずれかです。ファサードは `PluginManager` を所有し、
`Page` をプラグインホストインターフェースにブリッジするため、プラグインが生の
ページやトランスポートに触れることはありません。

XCL スクリプト言語は `crates/xcelerate-cli/src/xcl/`（レキサー、パーサー、エンジン、
ランタイム、セキュリティ、およびブラウザ/プラグイン/HTTP エグゼキューター）に
あります。詳しくは [`docs/xcl.md`](docs/xcl.md) を参照してください。

## 開発

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

エンドツーエンドテストは実際のブラウザを起動し、Chrome または Edge を必要とします。
`XCELERATE_TEST_URL` と `XCELERATE_CHROME` で、別のサイトやブラウザを指定できます。

## セキュリティ

脆弱性を発見しましたか？ [SECURITY.md](SECURITY.md) に従ってください - 公開の issue
は作成しないでください。サポート対象バージョン、報告チャネル、スコープはそこに記載
されています。

## License

次のいずれかのライセンスの下で提供されています

- Apache License, Version 2.0（[LICENSE-APACHE](LICENSE-APACHE)）
- MIT ライセンス（[LICENSE-MIT](LICENSE-MIT)）

お好みで選択してください。

特に明示しない限り、Apache-2.0 ライセンスで定義されているように、あなたがこの
プロジェクトに含めるために意図的に提出した貢献は、追加の条項や条件なしに、上記の
とおりデュアルライセンスされます。
