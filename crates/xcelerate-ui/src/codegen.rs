//! Recording model and multi-language code generation for the codegen overlay.
//!
//! The overlay records what the human does in the browser and renders it as a
//! runnable script in any of the eleven languages xcelerate ships bindings for.
//! Each target follows its own binding's naming convention (C# `PascalCase`,
//! Kotlin `camelCase`, Ruby `snake_case`, ...) so the emitted code matches the
//! package the user would actually import.

/// One recorded browser action, in the order it happened.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Open `url`.
    Navigate { url: String },
    /// Click the element matched by `selector`.
    Click { selector: String },
    /// Type `text` into the element matched by `selector`.
    Fill { selector: String, text: String },
    /// Press `key` while the element matched by `selector` has focus.
    Press { selector: String, key: String },
    /// Choose `value` in the `<select>` matched by `selector`.
    Select { selector: String, value: String },
    /// Move the pointer over the element matched by `selector`.
    Hover { selector: String },
    /// Wait for the element matched by `selector` to appear.
    WaitFor { selector: String },
    /// Read and log the text of the element matched by `selector`.
    Text { selector: String },
    /// Capture a full-page screenshot to `path`.
    Screenshot { path: String },
}

/// A codegen target: one of the languages xcelerate publishes bindings for.
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

    /// A plausible file name for the generated script, shown in the footer.
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
}

/// A normalized step, independent of any target language.
enum Op<'a> {
    NewPage(&'a str),
    Navigate(&'a str),
    Click(&'a str),
    Type(&'a str, &'a str),
    Press(&'a str, &'a str),
    Select(&'a str, &'a str),
    Hover(&'a str),
    Wait(&'a str),
    Text(&'a str),
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
            Action::Click { selector } => Op::Click(selector),
            Action::Fill { selector, text } => Op::Type(selector, text),
            Action::Press { selector, key } => Op::Press(selector, key),
            Action::Select { selector, value } => Op::Select(selector, value),
            Action::Hover { selector } => Op::Hover(selector),
            Action::WaitFor { selector } => Op::Wait(selector),
            Action::Text { selector } => Op::Text(selector),
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

/// A JSON array holding one value, as the `select_option` targets expect.
fn array(value: &str) -> String {
    format!("[{}]", q(value))
}

fn rust(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("let page = browser.new_page({}).await?;", q(u)),
            Op::Navigate(u) => format!("page.navigate({}).await?;", q(u)),
            Op::Click(s) => format!("page.find_element({}).await?.click().await?;", q(s)),
            Op::Type(s, t) => format!(
                "page.find_element({}).await?.type_text({}).await?;",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => {
                format!("page.find_element({}).await?.press({}).await?;", q(s), q(k))
            }
            Op::Select(s, v) => format!(
                "page.find_element({}).await?.select_option({}).await?;",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("page.find_element({}).await?.hover().await?;", q(s)),
            Op::Wait(s) => format!("page.wait_for_selector({}).await?;", q(s)),
            Op::Text(s) => format!(
                "println!(\"{{}}\", page.find_element({}).await?.text().await?);",
                q(s)
            ),
            Op::Screenshot(p) => {
                format!("std::fs::write({}, page.screenshot_full().await?)?;", q(p))
            }
        };
        body.l(1, &line);
    }
    format!(
        "use xcelerate::{{Browser, BrowserConfig}};\n\n\
         #[tokio::main]\n\
         async fn main() -> Result<(), xcelerate::XcelerateError> {{\n\
         \x20   let browser = Browser::launch(BrowserConfig::default()).await?;\n\
         {}    browser.close().await?;\n    Ok(())\n}}\n",
        body.out
    )
}

fn python(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page = await browser.new_page({})", q(u)),
            Op::Navigate(u) => format!("await page.navigate({})", q(u)),
            Op::Click(s) => format!("await (await page.find_element({})).click()", q(s)),
            Op::Type(s, t) => format!(
                "await (await page.find_element({})).type_text({})",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => format!("await (await page.find_element({})).press({})", q(s), q(k)),
            Op::Select(s, v) => format!(
                "await (await page.find_element({})).select_option({})",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("await (await page.find_element({})).hover()", q(s)),
            Op::Wait(s) => format!("await page.wait_for_selector({})", q(s)),
            Op::Text(s) => format!("print(await (await page.find_element({})).text())", q(s)),
            Op::Screenshot(p) => {
                format!("open({}, \"wb\").write(await page.screenshot_full())", q(p))
            }
        };
        body.l(1, &line);
    }
    format!(
        "import asyncio\nfrom xcelerate import Browser, BrowserConfig\n\n\n\
         async def main():\n\
         \x20   browser = await Browser.launch(BrowserConfig())\n\
         {}    await browser.close()\n\n\n\
         asyncio.run(main())\n",
        body.out
    )
}

fn javascript(ops: &[Op]) -> String {
    let mut body = Body::new("  ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("const page = await browser.new_page({});", q(u)),
            Op::Navigate(u) => format!("await page.navigate({});", q(u)),
            Op::Click(s) => format!("await (await page.find_element({})).click();", q(s)),
            Op::Type(s, t) => format!(
                "await (await page.find_element({})).type_text({});",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => {
                format!("await (await page.find_element({})).press({});", q(s), q(k))
            }
            Op::Select(s, v) => format!(
                "await (await page.find_element({})).select_option(JSON.stringify([{}]));",
                q(s),
                q(v)
            ),
            Op::Hover(s) => format!("await (await page.find_element({})).hover();", q(s)),
            Op::Wait(s) => format!("await page.wait_for_selector({});", q(s)),
            Op::Text(s) => format!(
                "console.log(await (await page.find_element({})).text());",
                q(s)
            ),
            Op::Screenshot(p) => {
                format!("fs.writeFileSync({}, await page.screenshot_full());", q(p))
            }
        };
        body.l(1, &line);
    }
    format!(
        "const {{ Browser }} = require('xcelerate');\nconst fs = require('fs');\n\n\
         async function main() {{\n\
         \x20 const browser = await Browser.launch({{ plugins: ['stealth'] }});\n\
         {}  await browser.close();\n}}\n\n\
         main().catch(console.error);\n",
        body.out
    )
}

fn csharp(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("using var page = await browser.NewPage({});", q(u)),
            Op::Navigate(u) => format!("await page.Navigate({});", q(u)),
            Op::Click(s) => format!("await (await page.FindElement({})).Click();", q(s)),
            Op::Type(s, t) => format!(
                "await (await page.FindElement({})).TypeText({});",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => format!("await (await page.FindElement({})).Press({});", q(s), q(k)),
            Op::Select(s, v) => format!(
                "await (await page.FindElement({})).SelectOption({});",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("await (await page.FindElement({})).Hover();", q(s)),
            Op::Wait(s) => format!("await page.WaitForSelector({});", q(s)),
            Op::Text(s) => format!(
                "Console.WriteLine(await (await page.FindElement({})).Text());",
                q(s)
            ),
            Op::Screenshot(p) => {
                format!("File.WriteAllBytes({}, await page.ScreenshotFull());", q(p))
            }
        };
        body.l(0, &line);
    }
    format!(
        "using System;\nusing System.IO;\nusing Xcelerate;\n\n\
         using var browser = await Browser.Launch(new BrowserConfig(Plugins: new[] {{ \"stealth\" }}));\n\
         {}\nawait browser.Close();\n",
        body.out
    )
}

fn kotlin(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("val page = browser.newPage({})", q(u)),
            Op::Navigate(u) => format!("page.navigate({})", q(u)),
            Op::Click(s) => format!("page.findElement({}).click()", q(s)),
            Op::Type(s, t) => format!("page.findElement({}).typeText({})", q(s), q(t)),
            Op::Press(s, k) => format!("page.findElement({}).press({})", q(s), q(k)),
            Op::Select(s, v) => {
                format!("page.findElement({}).selectOption({})", q(s), q(&array(v)))
            }
            Op::Hover(s) => format!("page.findElement({}).hover()", q(s)),
            Op::Wait(s) => format!("page.waitForSelector({})", q(s)),
            Op::Text(s) => format!("println(page.findElement({}).text())", q(s)),
            Op::Screenshot(p) => {
                format!("java.io.File({}).writeBytes(page.screenshotFull())", q(p))
            }
        };
        body.l(1, &line);
    }
    format!(
        "import uniffi.xcelerate.Browser\nimport uniffi.xcelerate.BrowserConfig\n\
         import kotlinx.coroutines.runBlocking\n\n\
         fun main() = runBlocking {{\n\
         \x20   val browser = Browser.launch(BrowserConfig(plugins = listOf(\"stealth\")))\n\
         {}    browser.closeBrowser()\n}}\n",
        body.out
    )
}

fn java(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("var page = browser.newPage({});", q(u)),
            Op::Navigate(u) => format!("page.navigate({});", q(u)),
            Op::Click(s) => format!("page.findElement({}).click();", q(s)),
            Op::Type(s, t) => format!("page.findElement({}).typeText({});", q(s), q(t)),
            Op::Press(s, k) => format!("page.findElement({}).press({});", q(s), q(k)),
            Op::Select(s, v) => {
                format!("page.findElement({}).selectOption({});", q(s), q(&array(v)))
            }
            Op::Hover(s) => format!("page.findElement({}).hover();", q(s)),
            Op::Wait(s) => format!("page.waitForSelector({});", q(s)),
            Op::Text(s) => format!("System.out.println(page.findElement({}).text());", q(s)),
            Op::Screenshot(p) => {
                format!("Files.write(Paths.get({}), page.screenshotFull());", q(p))
            }
        };
        body.l(2, &line);
    }
    format!(
        "import uniffi.xcelerate.Browser;\nimport uniffi.xcelerate.BrowserConfig;\n\
         import java.nio.file.Files;\nimport java.nio.file.Paths;\n\n\
         public class Main {{\n\
         \x20   public static void main(String[] args) throws Exception {{\n\
         \x20       var browser = Browser.launch(new BrowserConfig());\n\
         {}        browser.closeBrowser();\n    }}\n}}\n",
        body.out
    )
}

fn swift(ops: &[Op]) -> String {
    let mut body = Body::new("    ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("let page = try await browser.newPage(url: {})", q(u)),
            Op::Navigate(u) => format!("try await page.navigate(url: {})", q(u)),
            Op::Click(s) => format!("try await page.findElement(selector: {}).click()", q(s)),
            Op::Type(s, t) => format!(
                "try await page.findElement(selector: {}).typeText(text: {})",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => format!(
                "try await page.findElement(selector: {}).press(key: {})",
                q(s),
                q(k)
            ),
            Op::Select(s, v) => format!(
                "try await page.findElement(selector: {}).selectOption(valuesJson: {})",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("try await page.findElement(selector: {}).hover()", q(s)),
            Op::Wait(s) => format!("try await page.waitForSelector(selector: {})", q(s)),
            Op::Text(s) => format!(
                "print(try await page.findElement(selector: {}).text())",
                q(s)
            ),
            Op::Screenshot(p) => format!(
                "try await page.screenshotFull().write(to: URL(fileURLWithPath: {}))",
                q(p)
            ),
        };
        body.l(0, &line);
    }
    format!(
        "import Foundation\nimport Xcelerate\n\n\
         let browser = try await Browser.launch(config: BrowserConfig(\n\
         \x20   headless: true, detached: true, executablePath: nil, plugins: [\"stealth\"]))\n\
         {}\ntry await browser.close()\n",
        body.out
    )
}

fn ruby(ops: &[Op]) -> String {
    let mut body = Body::new("  ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page = browser.new_page({})", q(u)),
            Op::Navigate(u) => format!("page.navigate({})", q(u)),
            Op::Click(s) => format!("page.find_element({}).click", q(s)),
            Op::Type(s, t) => format!("page.find_element({}).type_text({})", q(s), q(t)),
            Op::Press(s, k) => format!("page.find_element({}).press({})", q(s), q(k)),
            Op::Select(s, v) => format!(
                "page.find_element({}).select_option({})",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("page.find_element({}).hover", q(s)),
            Op::Wait(s) => format!("page.wait_for_selector({})", q(s)),
            Op::Text(s) => format!("puts page.find_element({}).text", q(s)),
            Op::Screenshot(p) => format!("File.binwrite({}, page.screenshot_full)", q(p)),
        };
        body.l(1, &line);
    }
    format!(
        "require \"xcelerate\"\n\n\
         browser = Xcelerate::Browser.launch(Xcelerate::BrowserConfig.new(plugins: [\"stealth\"]))\n\
         {}browser.close\n",
        body.out
    )
}

fn dart(ops: &[Op]) -> String {
    let mut body = Body::new("  ");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("final page = await browser.newPage({});", q(u)),
            Op::Navigate(u) => format!("await page.navigate({});", q(u)),
            Op::Click(s) => format!("await (await page.findElement({})).click();", q(s)),
            Op::Type(s, t) => format!(
                "await (await page.findElement({})).typeText({});",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => format!("await (await page.findElement({})).press({});", q(s), q(k)),
            Op::Select(s, v) => format!(
                "await (await page.findElement({})).selectOption({});",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("await (await page.findElement({})).hover();", q(s)),
            Op::Wait(s) => format!("await page.waitForSelector({});", q(s)),
            Op::Text(s) => format!("print(await (await page.findElement({})).text());", q(s)),
            Op::Screenshot(p) => format!(
                "await File({}).writeAsBytes(await page.screenshotFull());",
                q(p)
            ),
        };
        body.l(1, &line);
    }
    format!(
        "import 'dart:io';\nimport 'package:xcelerate/xcelerate.dart';\n\n\
         Future<void> main() async {{\n\
         \x20 final browser = await Browser.launch(\n\
         \x20   BrowserConfig(headless: true, detached: true, plugins: ['stealth']));\n\
         {}  await browser.closeBrowser();\n}}\n",
        body.out
    )
}

fn go(ops: &[Op]) -> String {
    let mut body = Body::new("\t");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("page := must(browser.NewPage({}))", q(u)),
            Op::Navigate(u) => format!("must0(page.Navigate({}))", q(u)),
            Op::Click(s) => format!("must0(must(page.FindElement({})).Click())", q(s)),
            Op::Type(s, t) => format!("must0(must(page.FindElement({})).TypeText({}))", q(s), q(t)),
            Op::Press(s, k) => format!("must0(must(page.FindElement({})).Press({}))", q(s), q(k)),
            Op::Select(s, v) => format!(
                "must0(must(page.FindElement({})).SelectOption({}))",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!("must0(must(page.FindElement({})).Hover())", q(s)),
            Op::Wait(s) => format!("must0(page.WaitForSelector({}))", q(s)),
            Op::Text(s) => format!("fmt.Println(must(must(page.FindElement({})).Text()))", q(s)),
            Op::Screenshot(p) => {
                format!(
                    "must0(os.WriteFile({}, must(page.ScreenshotFull()), 0o644))",
                    q(p)
                )
            }
        };
        body.l(1, &line);
    }
    format!(
        "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n\n\t\
         xcelerate \"github.com/AzzoDude/xcelerate/bindings/go\"\n)\n\n\
         func must[T any](value T, err error) T {{\n\
         \tif err != nil {{\n\t\tpanic(err)\n\t}}\n\treturn value\n}}\n\n\
         func must0(err error) {{\n\tif err != nil {{\n\t\tpanic(err)\n\t}}\n}}\n\n\
         func main() {{\n\
         \tbrowser := must(xcelerate.BrowserLaunch(xcelerate.BrowserConfig{{}}))\n\
         \tdefer must0(browser.Close())\n\
         {}}}\n",
        body.out
    )
}

fn powershell(ops: &[Op]) -> String {
    let mut body = Body::new("");
    for op in ops {
        let line = match op {
            Op::NewPage(u) => format!("$page = New-XceleratePage -Browser $browser -Url {}", q(u)),
            Op::Navigate(u) => format!("Receive-XcelerateTask $page.Navigate({})", q(u)),
            Op::Click(s) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nReceive-XcelerateTask $el.Click()",
                q(s)
            ),
            Op::Type(s, t) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nReceive-XcelerateTask $el.TypeText({})",
                q(s),
                q(t)
            ),
            Op::Press(s, k) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nReceive-XcelerateTask $el.Press({})",
                q(s),
                q(k)
            ),
            Op::Select(s, v) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nReceive-XcelerateTask $el.SelectOption({})",
                q(s),
                q(&array(v))
            ),
            Op::Hover(s) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nReceive-XcelerateTask $el.Hover()",
                q(s)
            ),
            Op::Wait(s) => format!("Receive-XcelerateTask $page.WaitForSelector({})", q(s)),
            Op::Text(s) => format!(
                "$el = Receive-XcelerateTask $page.FindElement({})\nWrite-Host (Receive-XcelerateTask $el.Text())",
                q(s)
            ),
            Op::Screenshot(p) => format!(
                "$png = Receive-XcelerateTask $page.ScreenshotFull()\n[IO.File]::WriteAllBytes({}, $png)",
                q(p)
            ),
        };
        body.l(0, &line);
    }
    format!(
        "Import-Module Xcelerate\n\n\
         $browser = Start-XcelerateBrowser -Plugins stealth, human\n\
         {}\nStop-XcelerateBrowser $browser\n",
        body.out
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
                selector: "#go".into(),
            },
            Action::Fill {
                selector: "#q".into(),
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
        // The first navigate becomes the page open, so it appears once.
        assert_eq!(code.matches("https://example.com").count(), 1);
    }

    #[test]
    fn empty_recording_still_launches() {
        let code = Language::Rust.generate(&[]);
        assert!(code.contains("let browser = Browser::launch"));
    }
}
