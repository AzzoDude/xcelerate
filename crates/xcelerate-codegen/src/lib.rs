//! Recording model and multi-language code generation for a xcelerate session.
//!
//! A recorded session is a list of [`Action`]s. [`Language::generate`] renders
//! that list as a runnable script in any of the eleven languages xcelerate ships
//! bindings for, matching each binding's own naming convention (C# `PascalCase`,
//! Kotlin/JS `camelCase`, Ruby/Python `snake_case`, ...) so the emitted code
//! lines up with the package the user would actually import.
//!
//! Selectors are modelled, not just stringly-typed. A [`Selector`] knows whether
//! it is CSS, XPath, an id, or text, and each target renders the *right* lookup
//! for that kind - `find_element` for CSS, `query_selector_xpath` for XPath,
//! `#id` for an id, and a normalised XPath expression for text. This is the
//! difference between code that pastes and code that runs: a bare XPath handed
//! to a CSS lookup silently matches nothing.
//!
//! In addition to rendering recorded *scripts*, this crate can render a *typed
//! client binding* for a plugin op from the op's JSON Schema. See the
//! [`bindings`] module and [`Language::generate_binding`].

mod bindings;

/// How a recorded selector should be looked up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectorKind {
    /// A CSS selector (`#id`, `.class`, `button[type=submit]`).
    Css,
    /// An XPath expression, relative (`//a`) or absolute / "full" (`/html/body/a`).
    XPath,
    /// An element id, recorded explicitly (`id=submit`). Rendered as `#submit`.
    Id,
    /// Visible text (`text=Sign in`). Rendered as a normalised XPath match.
    Text,
}

/// A recorded selector together with how it should be resolved.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Selector {
    kind: SelectorKind,
    value: String,
}

impl Selector {
    /// A CSS selector.
    pub fn css(value: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Css,
            value: value.into(),
        }
    }

    /// An XPath expression (relative or absolute).
    pub fn xpath(value: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::XPath,
            value: value.into(),
        }
    }

    /// An element id.
    pub fn id(value: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Id,
            value: value.into(),
        }
    }

    /// Visible text.
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            kind: SelectorKind::Text,
            value: value.into(),
        }
    }

    /// Classifies a recorded selector string.
    ///
    /// An explicit `xpath=`, `css=`, `id=` or `text=` prefix wins. Otherwise a
    /// string that looks like XPath (starts with `/` or `(`) is treated as XPath,
    /// and everything else as CSS.
    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        if let Some(value) = raw.strip_prefix("xpath=") {
            return Self::xpath(value);
        }
        if let Some(value) = raw.strip_prefix("css=") {
            return Self::css(value);
        }
        if let Some(value) = raw.strip_prefix("id=") {
            return Self::id(value);
        }
        if let Some(value) = raw.strip_prefix("text=") {
            return Self::text(value);
        }
        if raw.starts_with('/') || raw.starts_with('(') {
            return Self::xpath(raw);
        }
        Self::css(raw)
    }

    /// The selector kind.
    pub fn kind(&self) -> SelectorKind {
        self.kind
    }

    /// The raw selector value (without any scheme prefix).
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Folds the selector into the concrete lookup the targets understand.
    fn lookup(&self) -> Lookup {
        match self.kind {
            SelectorKind::Css => Lookup::Css(self.value.clone()),
            SelectorKind::Id => Lookup::Css(format!("#{}", self.value)),
            SelectorKind::XPath => Lookup::XPath(self.value.clone()),
            SelectorKind::Text => Lookup::XPath(format!(
                "//*[contains(normalize-space(.), {})]",
                xpath_literal(&self.value)
            )),
        }
    }
}

/// A resolved lookup: either CSS or XPath.
enum Lookup {
    Css(String),
    XPath(String),
}

/// One recorded browser action, in the order it happened.
#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    /// Open `url`.
    Navigate { url: String },
    /// Click the element matched by `selector`.
    Click { selector: Selector },
    /// Type `text` into the element matched by `selector`.
    Fill { selector: Selector, text: String },
    /// Press `key` while the element matched by `selector` has focus.
    Press { selector: Selector, key: String },
    /// Choose `value` in the `<select>` matched by `selector`.
    Select { selector: Selector, value: String },
    /// Move the pointer over the element matched by `selector`.
    Hover { selector: Selector },
    /// Wait for the element matched by `selector` to appear.
    WaitFor { selector: Selector },
    /// Read and log the text of the element matched by `selector`.
    Text { selector: Selector },
    /// Capture a full-page screenshot to `path`.
    Screenshot { path: String },
}

/// The codegen target: one of the languages xcelerate publishes bindings for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    CSharp,
    Kotlin,
    Java,
    Swift,
    Ruby,
    Dart,
    Go,
    PowerShell,
}

impl Language {
    /// Every target, in menu order.
    pub const ALL: [Language; 11] = [
        Language::Rust,
        Language::Python,
        Language::JavaScript,
        Language::CSharp,
        Language::Kotlin,
        Language::Java,
        Language::Swift,
        Language::Ruby,
        Language::Dart,
        Language::Go,
        Language::PowerShell,
    ];

    /// The name shown in the language menu.
    pub fn label(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::CSharp => "C# / .NET",
            Language::Kotlin => "Kotlin",
            Language::Java => "Java",
            Language::Swift => "Swift",
            Language::Ruby => "Ruby",
            Language::Dart => "Dart / Flutter",
            Language::Go => "Go",
            Language::PowerShell => "PowerShell",
        }
    }

    /// A plausible file name for the generated script.
    pub fn file_name(self) -> &'static str {
        match self {
            Language::Rust => "main.rs",
            Language::Python => "codegen.py",
            Language::JavaScript => "codegen.js",
            Language::CSharp => "Program.cs",
            Language::Kotlin => "main.kt",
            Language::Java => "Main.java",
            Language::Swift => "main.swift",
            Language::Ruby => "codegen.rb",
            Language::Dart => "main.dart",
            Language::Go => "main.go",
            Language::PowerShell => "codegen.ps1",
        }
    }

    /// Renders the recorded actions as a runnable script in this language.
    pub fn generate(self, actions: &[Action]) -> String {
        let ops = plan(actions);
        match self {
            Language::Rust => rust(&ops),
            Language::Python => python(&ops),
            Language::JavaScript => javascript(&ops),
            Language::CSharp => csharp(&ops),
            Language::Kotlin => kotlin(&ops),
            Language::Java => java(&ops),
            Language::Swift => swift(&ops),
            Language::Ruby => ruby(&ops),
            Language::Dart => dart(&ops),
            Language::Go => go(&ops),
            Language::PowerShell => powershell(&ops),
        }
    }

    /// Renders a typed client binding for a plugin op from its JSON Schema.
    ///
    /// `schema_json` is the op's input object schema (JSON Schema Draft
    /// 2020-12, flattened object form) and `defaults_json` the flat default
    /// values. The result is a source file exposing a typed client whose call
    /// signature carries real types and native defaults in this language.
    pub fn generate_binding(self, op_name: &str, schema_json: &str, defaults_json: &str) -> String {
        bindings::render(self, op_name, schema_json, defaults_json)
    }
}

/// A normalized step, independent of any target language.
enum Op<'a> {
    NewPage(&'a str),
    Navigate(&'a str),
    Click(Lookup),
    Type(Lookup, &'a str),
    Press(Lookup, &'a str),
    Select(Lookup, &'a str),
    Hover(Lookup),
    Wait(Lookup),
    Text(Lookup),
    Screenshot(&'a str),
}

/// Folds the recorded actions into ops, hoisting an opening navigation into the
/// page creation so the emitted script does not navigate twice.
fn plan(actions: &[Action]) -> Vec<Op<'_>> {
    let mut ops = Vec::with_capacity(actions.len() + 1);
    let rest: &[Action] = if let Some(Action::Navigate { url }) = actions.first() {
        ops.push(Op::NewPage(url));
        &actions[1..]
    } else {
        ops.push(Op::NewPage("about:blank"));
        actions
    };
    for action in rest {
        ops.push(match action {
            Action::Navigate { url } => Op::Navigate(url),
            Action::Click { selector } => Op::Click(selector.lookup()),
            Action::Fill { selector, text } => Op::Type(selector.lookup(), text),
            Action::Press { selector, key } => Op::Press(selector.lookup(), key),
            Action::Select { selector, value } => Op::Select(selector.lookup(), value),
            Action::Hover { selector } => Op::Hover(selector.lookup()),
            Action::WaitFor { selector } => Op::Wait(selector.lookup()),
            Action::Text { selector } => Op::Text(selector.lookup()),
            Action::Screenshot { path } => Op::Screenshot(path),
        });
    }
    ops
}

/// Accumulates an indented body with a per-language indent unit.
struct Body {
    out: String,
    unit: &'static str,
}

impl Body {
    fn new(unit: &'static str) -> Self {
        Self {
            out: String::new(),
            unit,
        }
    }

    fn l(&mut self, level: usize, text: &str) {
        self.out.push_str(&self.unit.repeat(level));
        self.out.push_str(text);
        self.out.push('\n');
    }
}

fn assemble(header: &str, body: &Body, footer: &str) -> String {
    let mut out = String::with_capacity(header.len() + body.out.len() + footer.len());
    out.push_str(header);
    out.push_str(&body.out);
    out.push_str(footer);
    out
}

/// A double-quoted, escaped string literal (valid in every target here).
fn q(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A JSON array holding one string, as `select_option` expects.
fn json_array(value: &str) -> String {
    format!("[{}]", q(value))
}

/// An XPath 1.0 string literal for `text`, choosing `"`, `'`, or `concat()` so
/// the expression stays valid whatever quotes the text contains.
fn xpath_literal(text: &str) -> String {
    if !text.contains('"') {
        return format!("\"{text}\"");
    }
    if !text.contains('\'') {
        return format!("'{text}'");
    }
    let mut parts: Vec<String> = Vec::new();
    for (index, piece) in text.split('"').enumerate() {
        if index > 0 {
            parts.push("\"'\"".to_string());
        }
        if !piece.is_empty() {
            parts.push(format!("\"{piece}\""));
        }
    }
    format!("concat({})", parts.join(", "))
}

// ---------------------------------------------------------------------------
// Rust
// ---------------------------------------------------------------------------

fn rust_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!(
            "Arc::clone(&page).find_element({}.to_string()).await?",
            q(s)
        ),
        Lookup::XPath(s) => format!(
            "Arc::clone(&page).query_selector_xpath({}.to_string()).await?",
            q(s)
        ),
    }
}

fn rust(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!(
                "let page = Arc::clone(&browser).new_page({}.to_string()).await?;",
                q(u)
            ),
            Op::Navigate(u) => format!("page.navigate({}.to_string()).await?;", q(u)),
            Op::Click(lu) => format!("{}.click().await?;", rust_elem(lu)),
            Op::Type(lu, t) => {
                format!("{}.type_text({}.to_string()).await?;", rust_elem(lu), q(t))
            }
            Op::Press(lu, k) => {
                format!("{}.press({}.to_string()).await?;", rust_elem(lu), q(k))
            }
            Op::Select(lu, v) => format!(
                "{}.select_option({}.to_string()).await?;",
                rust_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!("{}.hover().await?;", rust_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!(
                    "Arc::clone(&page).wait_for_selector({}.to_string()).await?;",
                    q(s)
                ),
                Lookup::XPath(s) => format!(
                    "Arc::clone(&page).wait_for_xpath({}.to_string(), 30_000).await?;",
                    q(s)
                ),
            },
            Op::Text(lu) => format!("println!(\"{{}}\", {}.text().await?);", rust_elem(lu)),
            Op::Screenshot(p) => {
                format!("std::fs::write({}, page.screenshot_full().await?)?;", q(p))
            }
        };
        body.l(1, &line);
    }
    assemble(
        "use std::sync::Arc;\nuse xcelerate::{Browser, BrowserConfig};\n\n\
         #[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n\
         \x20   let browser = Browser::launch(BrowserConfig::default()).await?;\n",
        &body,
        "    browser.close().await?;\n    Ok(())\n}\n",
    )
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

fn python_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("await page.find_element({})", q(s)),
        Lookup::XPath(s) => format!("await page.query_selector_xpath({})", q(s)),
    }
}

fn python(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page = await browser.new_page({})", q(u)),
            Op::Navigate(u) => format!("await page.navigate({})", q(u)),
            Op::Click(lu) => format!("await ({}).click()", python_elem(lu)),
            Op::Type(lu, t) => format!("await ({}).type_text({})", python_elem(lu), q(t)),
            Op::Press(lu, k) => format!("await ({}).press({})", python_elem(lu), q(k)),
            Op::Select(lu, v) => format!(
                "await ({}).select_option({})",
                python_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!("await ({}).hover()", python_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("await page.wait_for_selector({})", q(s)),
                Lookup::XPath(s) => format!("await page.wait_for_xpath({}, 30000)", q(s)),
            },
            Op::Text(lu) => format!("print(await ({}).text())", python_elem(lu)),
            Op::Screenshot(p) => {
                format!("open({}, \"wb\").write(await page.screenshot_full())", q(p))
            }
        };
        body.l(1, &line);
    }
    assemble(
        "import asyncio\nfrom xcelerate import Browser, BrowserConfig\n\n\n\
         async def main():\n    browser = await Browser.launch(BrowserConfig())\n",
        &body,
        "    await browser.close()\n\n\nasyncio.run(main())\n",
    )
}

// ---------------------------------------------------------------------------
// JavaScript / Node
// ---------------------------------------------------------------------------

fn javascript_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("await page.findElement({})", q(s)),
        Lookup::XPath(s) => format!("await page.querySelectorXpath({})", q(s)),
    }
}

fn javascript(ops: &[Op]) -> String {
    let mut body = Body::new("  ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("const page = await browser.newPage({});", q(u)),
            Op::Navigate(u) => format!("await page.navigate({});", q(u)),
            Op::Click(lu) => format!("await ({}).click();", javascript_elem(lu)),
            Op::Type(lu, t) => format!("await ({}).typeText({});", javascript_elem(lu), q(t)),
            Op::Press(lu, k) => format!("await ({}).press({});", javascript_elem(lu), q(k)),
            Op::Select(lu, v) => format!(
                "await ({}).selectOption(JSON.stringify({}));",
                javascript_elem(lu),
                json_array(v)
            ),
            Op::Hover(lu) => format!("await ({}).hover();", javascript_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("await page.waitForSelector({});", q(s)),
                Lookup::XPath(s) => format!("await page.waitForXpath({}, 30000);", q(s)),
            },
            Op::Text(lu) => format!("console.log(await ({}).text());", javascript_elem(lu)),
            Op::Screenshot(p) => {
                format!("fs.writeFileSync({}, await page.screenshotFull());", q(p))
            }
        };
        body.l(1, &line);
    }
    assemble(
        "const { Browser } = require('xcelerate');\nconst fs = require('fs');\n\n\
         async function main() {\n  const browser = await Browser.launch({});\n",
        &body,
        "  await browser.close();\n}\n\nmain().catch(console.error);\n",
    )
}

// ---------------------------------------------------------------------------
// C# / .NET
// ---------------------------------------------------------------------------

fn csharp_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("await page.FindElement({})", q(s)),
        Lookup::XPath(s) => format!("await page.QuerySelectorXpath({})", q(s)),
    }
}

fn csharp(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("using var page = await browser.NewPage({});", q(u)),
            Op::Navigate(u) => format!("await page.Navigate({});", q(u)),
            Op::Click(lu) => format!("await ({}).Click();", csharp_elem(lu)),
            Op::Type(lu, t) => format!("await ({}).TypeText({});", csharp_elem(lu), q(t)),
            Op::Press(lu, k) => format!("await ({}).Press({});", csharp_elem(lu), q(k)),
            Op::Select(lu, v) => format!(
                "await ({}).SelectOption({});",
                csharp_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!("await ({}).Hover();", csharp_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("await page.WaitForSelector({});", q(s)),
                Lookup::XPath(s) => format!("await page.WaitForXpath({}, 30000);", q(s)),
            },
            Op::Text(lu) => format!("Console.WriteLine(await ({}).Text());", csharp_elem(lu)),
            Op::Screenshot(p) => {
                format!("File.WriteAllBytes({}, await page.ScreenshotFull());", q(p))
            }
        };
        body.l(0, &line);
    }
    assemble(
        "using System;\nusing System.IO;\nusing Xcelerate;\n\n\
         using var browser = await Browser.Launch(new BrowserConfig());\n",
        &body,
        "\nawait browser.Close();\n",
    )
}

// ---------------------------------------------------------------------------
// Kotlin
// ---------------------------------------------------------------------------

fn kotlin_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("page.findElement({})", q(s)),
        Lookup::XPath(s) => format!("page.querySelectorXpath({})", q(s)),
    }
}

fn kotlin(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("val page = browser.newPage({})", q(u)),
            Op::Navigate(u) => format!("page.navigate({})", q(u)),
            Op::Click(lu) => format!("({}).click()", kotlin_elem(lu)),
            Op::Type(lu, t) => format!("({}).typeText({})", kotlin_elem(lu), q(t)),
            Op::Press(lu, k) => format!("({}).press({})", kotlin_elem(lu), q(k)),
            Op::Select(lu, v) => {
                format!("({}).selectOption({})", kotlin_elem(lu), q(&json_array(v)))
            }
            Op::Hover(lu) => format!("({}).hover()", kotlin_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("page.waitForSelector({})", q(s)),
                Lookup::XPath(s) => format!("page.waitForXpath({}, 30000)", q(s)),
            },
            Op::Text(lu) => format!("println(({}).text())", kotlin_elem(lu)),
            Op::Screenshot(p) => {
                format!("java.io.File({}).writeBytes(page.screenshotFull())", q(p))
            }
        };
        body.l(1, &line);
    }
    assemble(
        "import uniffi.xcelerate.Browser\nimport uniffi.xcelerate.BrowserConfig\n\
         import kotlinx.coroutines.runBlocking\n\nfun main() = runBlocking {\n\
         \x20   val browser = Browser.launch(BrowserConfig())\n",
        &body,
        "    browser.closeBrowser()\n}\n",
    )
}

// ---------------------------------------------------------------------------
// Java
// ---------------------------------------------------------------------------

fn java(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("var page = browser.newPage({});", q(u)),
            Op::Navigate(u) => format!("page.navigate({});", q(u)),
            Op::Click(lu) => format!("{}.click();", kotlin_elem(lu)),
            Op::Type(lu, t) => format!("{}.typeText({});", kotlin_elem(lu), q(t)),
            Op::Press(lu, k) => format!("{}.press({});", kotlin_elem(lu), q(k)),
            Op::Select(lu, v) => {
                format!("{}.selectOption({});", kotlin_elem(lu), q(&json_array(v)))
            }
            Op::Hover(lu) => format!("{}.hover();", kotlin_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("page.waitForSelector({});", q(s)),
                Lookup::XPath(s) => format!("page.waitForXpath({}, 30000);", q(s)),
            },
            Op::Text(lu) => format!("System.out.println({}.text());", kotlin_elem(lu)),
            Op::Screenshot(p) => {
                format!("Files.write(Paths.get({}), page.screenshotFull());", q(p))
            }
        };
        body.l(2, &line);
    }
    assemble(
        "import uniffi.xcelerate.Browser;\nimport uniffi.xcelerate.BrowserConfig;\n\
         import java.nio.file.Files;\nimport java.nio.file.Paths;\n\n\
         public class Main {\n    public static void main(String[] args) throws Exception {\n\
         \x20       var browser = Browser.launch(new BrowserConfig());\n",
        &body,
        "        browser.closeBrowser();\n    }\n}\n",
    )
}

// ---------------------------------------------------------------------------
// Swift
// ---------------------------------------------------------------------------

fn swift_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("page.findElement(selector: {})", q(s)),
        Lookup::XPath(s) => format!("page.querySelectorXpath(xpath: {})", q(s)),
    }
}

fn swift(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("let page = try await browser.newPage(url: {})", q(u)),
            Op::Navigate(u) => format!("try await page.navigate(url: {})", q(u)),
            Op::Click(lu) => format!("try await {}.click()", swift_elem(lu)),
            Op::Type(lu, t) => format!("try await {}.typeText(text: {})", swift_elem(lu), q(t)),
            Op::Press(lu, k) => format!("try await {}.press(key: {})", swift_elem(lu), q(k)),
            Op::Select(lu, v) => format!(
                "try await {}.selectOption(valuesJson: {})",
                swift_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!("try await {}.hover()", swift_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("try await page.waitForSelector(selector: {})", q(s)),
                Lookup::XPath(s) => {
                    format!(
                        "try await page.waitForXpath(xpath: {}, timeout: 30000)",
                        q(s)
                    )
                }
            },
            Op::Text(lu) => format!("print(try await {}.text())", swift_elem(lu)),
            Op::Screenshot(p) => format!(
                "try await page.screenshotFull().write(to: URL(fileURLWithPath: {}))",
                q(p)
            ),
        };
        body.l(0, &line);
    }
    assemble(
        "import Foundation\nimport Xcelerate\n\n\
         let browser = try await Browser.launch(config: BrowserConfig())\n",
        &body,
        "\ntry await browser.close()\n",
    )
}

// ---------------------------------------------------------------------------
// Ruby
// ---------------------------------------------------------------------------

fn ruby_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("page.find_element({})", q(s)),
        Lookup::XPath(s) => format!("page.query_selector_xpath({})", q(s)),
    }
}

fn ruby(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page = browser.new_page({})", q(u)),
            Op::Navigate(u) => format!("page.navigate({})", q(u)),
            Op::Click(lu) => format!("({}).click", ruby_elem(lu)),
            Op::Type(lu, t) => format!("({}).type_text({})", ruby_elem(lu), q(t)),
            Op::Press(lu, k) => format!("({}).press({})", ruby_elem(lu), q(k)),
            Op::Select(lu, v) => {
                format!("({}).select_option({})", ruby_elem(lu), q(&json_array(v)))
            }
            Op::Hover(lu) => format!("({}).hover", ruby_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("page.wait_for_selector({})", q(s)),
                Lookup::XPath(s) => format!("page.wait_for_xpath({}, 30000)", q(s)),
            },
            Op::Text(lu) => format!("puts ({}).text", ruby_elem(lu)),
            Op::Screenshot(p) => format!("File.binwrite({}, page.screenshot_full)", q(p)),
        };
        body.l(0, &line);
    }
    assemble(
        "require \"xcelerate\"\n\n\
         browser = Xcelerate::Browser.launch(Xcelerate::BrowserConfig.new)\n",
        &body,
        "browser.close\n",
    )
}

// ---------------------------------------------------------------------------
// Dart / Flutter
// ---------------------------------------------------------------------------

fn dart_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("await page.findElement({})", q(s)),
        Lookup::XPath(s) => format!("await page.querySelectorXpath({})", q(s)),
    }
}

fn dart(ops: &[Op]) -> String {
    let mut body = Body::new("  ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("final page = await browser.newPage({});", q(u)),
            Op::Navigate(u) => format!("await page.navigate({});", q(u)),
            Op::Click(lu) => format!("await ({}).click();", dart_elem(lu)),
            Op::Type(lu, t) => format!("await ({}).typeText({});", dart_elem(lu), q(t)),
            Op::Press(lu, k) => format!("await ({}).press({});", dart_elem(lu), q(k)),
            Op::Select(lu, v) => format!(
                "await ({}).selectOption({});",
                dart_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!("await ({}).hover();", dart_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("await page.waitForSelector({});", q(s)),
                Lookup::XPath(s) => format!("await page.waitForXpath({}, 30000);", q(s)),
            },
            Op::Text(lu) => format!("print(await ({}).text());", dart_elem(lu)),
            Op::Screenshot(p) => format!(
                "await File({}).writeAsBytes(await page.screenshotFull());",
                q(p)
            ),
        };
        body.l(1, &line);
    }
    assemble(
        "import 'dart:io';\nimport 'package:xcelerate/xcelerate.dart';\n\n\
         Future<void> main() async {\n  final browser = await Browser.launch(BrowserConfig());\n",
        &body,
        "  await browser.closeBrowser();\n}\n",
    )
}

// ---------------------------------------------------------------------------
// Go
// ---------------------------------------------------------------------------

fn go_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("must(page.FindElement({}))", q(s)),
        Lookup::XPath(s) => format!("must(page.QuerySelectorXpath({}))", q(s)),
    }
}

fn go(ops: &[Op]) -> String {
    let mut body = Body::new("\t");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page := must(browser.NewPage({}))", q(u)),
            Op::Navigate(u) => format!("must0(page.Navigate({}))", q(u)),
            Op::Click(lu) => format!("must0({}.Click())", go_elem(lu)),
            Op::Type(lu, t) => format!("must0({}.TypeText({}))", go_elem(lu), q(t)),
            Op::Press(lu, k) => format!("must0({}.Press({}))", go_elem(lu), q(k)),
            Op::Select(lu, v) => {
                format!("must0({}.SelectOption({}))", go_elem(lu), q(&json_array(v)))
            }
            Op::Hover(lu) => format!("must0({}.Hover())", go_elem(lu)),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("must0(page.WaitForSelector({}))", q(s)),
                Lookup::XPath(s) => format!("must0(page.WaitForXpath({}, 30000))", q(s)),
            },
            Op::Text(lu) => format!("fmt.Println(must({}.Text()))", go_elem(lu)),
            Op::Screenshot(p) => format!(
                "must0(os.WriteFile({}, must(page.ScreenshotFull()), 0o644))",
                q(p)
            ),
        };
        body.l(1, &line);
    }
    assemble(
        "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n\n\t\
         xcelerate \"github.com/AzzoDude/xcelerate/bindings/go\"\n)\n\n\
         func must[T any](value T, err error) T {\n\tif err != nil {\n\t\tpanic(err)\n\t}\n\treturn value\n}\n\n\
         func must0(err error) {\n\tif err != nil {\n\t\tpanic(err)\n\t}\n}\n\n\
         func main() {\n\tbrowser := must(xcelerate.BrowserLaunch(xcelerate.BrowserConfig{}))\n\tdefer must0(browser.Close())\n",
        &body,
        "}\n",
    )
}

// ---------------------------------------------------------------------------
// PowerShell
// ---------------------------------------------------------------------------

fn powershell_elem(lookup: &Lookup) -> String {
    match lookup {
        Lookup::Css(s) => format!("Receive-XcelerateTask $page.FindElement({})", q(s)),
        Lookup::XPath(s) => format!("Receive-XcelerateTask $page.QuerySelectorXpath({})", q(s)),
    }
}

fn powershell(ops: &[Op]) -> String {
    let mut body = Body::new("");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("$page = New-XceleratePage -Browser $browser -Url {}", q(u)),
            Op::Navigate(u) => format!("Receive-XcelerateTask $page.Navigate({})", q(u)),
            Op::Click(lu) => format!(
                "$el = {}\nReceive-XcelerateTask $el.Click()",
                powershell_elem(lu)
            ),
            Op::Type(lu, t) => format!(
                "$el = {}\nReceive-XcelerateTask $el.TypeText({})",
                powershell_elem(lu),
                q(t)
            ),
            Op::Press(lu, k) => format!(
                "$el = {}\nReceive-XcelerateTask $el.Press({})",
                powershell_elem(lu),
                q(k)
            ),
            Op::Select(lu, v) => format!(
                "$el = {}\nReceive-XcelerateTask $el.SelectOption({})",
                powershell_elem(lu),
                q(&json_array(v))
            ),
            Op::Hover(lu) => format!(
                "$el = {}\nReceive-XcelerateTask $el.Hover()",
                powershell_elem(lu)
            ),
            Op::Wait(lu) => match lu {
                Lookup::Css(s) => format!("Receive-XcelerateTask $page.WaitForSelector({})", q(s)),
                Lookup::XPath(s) => {
                    format!("Receive-XcelerateTask $page.WaitForXpath({}, 30000)", q(s))
                }
            },
            Op::Text(lu) => format!(
                "$el = {}\nWrite-Host (Receive-XcelerateTask $el.Text())",
                powershell_elem(lu)
            ),
            Op::Screenshot(p) => format!(
                "$png = Receive-XcelerateTask $page.ScreenshotFull()\n[IO.File]::WriteAllBytes({}, $png)",
                q(p)
            ),
        };
        body.l(0, &line);
    }
    assemble(
        "Import-Module Xcelerate\n\n$browser = Start-XcelerateBrowser\n",
        &body,
        "Stop-XcelerateBrowser $browser\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Action> {
        vec![
            Action::Navigate {
                url: "https://example.com".into(),
            },
            Action::Click {
                selector: Selector::css("#go"),
            },
            Action::Fill {
                selector: Selector::parse("#q"),
                text: "hello".into(),
            },
        ]
    }

    #[test]
    fn every_language_renders_every_action() {
        for language in Language::ALL {
            let code = language.generate(&sample());
            assert!(code.contains("https://example.com"), "{:?}", language);
            assert!(code.contains("#go"), "{:?}", language);
            assert!(code.contains("hello"), "{:?}", language);
        }
    }

    #[test]
    fn opening_navigation_is_not_repeated() {
        let code = Language::Python.generate(&sample());
        assert_eq!(code.matches("https://example.com").count(), 1);
    }

    #[test]
    fn empty_recording_still_launches() {
        let code = Language::Rust.generate(&[]);
        assert!(code.contains("Browser::launch"));
        assert!(code.contains("new_page(\"about:blank\""));
    }

    #[test]
    fn parse_classifies_selectors() {
        assert_eq!(Selector::parse("#q").kind(), SelectorKind::Css);
        assert_eq!(Selector::parse("//a[@id='x']").kind(), SelectorKind::XPath);
        assert_eq!(
            Selector::parse("/html/body/div[1]/a").kind(),
            SelectorKind::XPath
        );
        assert_eq!(Selector::parse("xpath=//li").kind(), SelectorKind::XPath);
        assert_eq!(Selector::parse("id=submit").kind(), SelectorKind::Id);
        assert_eq!(Selector::parse("text=Sign in").kind(), SelectorKind::Text);
        assert_eq!(Selector::parse("css=.card").kind(), SelectorKind::Css);
    }

    #[test]
    fn xpath_uses_the_xpath_lookup_and_css_uses_css() {
        let actions = vec![
            Action::Click {
                selector: Selector::parse("//button[text()='Go']"),
            },
            Action::Click {
                selector: Selector::css("#go"),
            },
        ];
        for language in Language::ALL {
            let code = language.generate(&actions);
            assert!(
                code.contains("Xpath") || code.contains("xpath"),
                "{:?} should use an xpath lookup",
                language
            );
            assert!(
                code.contains("#go"),
                "{:?} should still emit the css selector",
                language
            );
        }
    }

    #[test]
    fn rust_uses_arc_clone_not_a_moved_page() {
        let actions = vec![
            Action::Navigate {
                url: "https://example.com".into(),
            },
            Action::Click {
                selector: Selector::css("#go"),
            },
        ];
        let code = Language::Rust.generate(&actions);
        assert!(code.contains("Arc::clone(&page).find_element"), "{code}");
        assert!(code.contains("Arc::clone(&browser).new_page"), "{code}");
    }

    #[test]
    fn javascript_uses_camel_case_binding_names() {
        let code = Language::JavaScript.generate(&sample());
        assert!(code.contains("findElement"), "{code}");
        assert!(!code.contains("find_element"), "{code}");
    }

    #[test]
    fn text_selector_becomes_a_normalised_xpath() {
        let actions = vec![Action::Click {
            selector: Selector::text("Sign in"),
        }];
        let code = Language::Python.generate(&actions);
        assert!(code.contains("normalize-space"), "{code}");
        assert!(!code.contains("find_element"), "{code}");
    }

    #[test]
    fn xpath_literal_handles_quotes() {
        assert_eq!(xpath_literal("plain"), "\"plain\"");
        assert_eq!(xpath_literal("has 'single'"), "\"has 'single'\"");
        assert_eq!(xpath_literal("has \"double\""), "'has \"double\"'");
        assert_eq!(
            xpath_literal("both ' and \" quotes"),
            "concat(\"both ' and \", \"'\", \" quotes\")"
        );
    }
}
