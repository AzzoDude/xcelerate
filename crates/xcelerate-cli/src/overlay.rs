//! The native overlay, or an inert stand-in when the `ui` feature is off.
//!
//! The session talks only to this module, so the GUI stack (winit + egui) can be
//! dropped from the CLI build with `--no-default-features` without touching the
//! session: every call then becomes a cheap no-op.
//!
//! Unlike the old in-page HUD, this is a real OS window floating above the
//! browser. It is not affected by page CSS, survives every navigation, and its
//! Stop / Pause reach the session without a CDP round-trip.

use crate::cli::{Cli, Command};

#[cfg(feature = "ui")]
pub use xcelerate_ui::{Bounds, OverlayHandle};

/// Whether the run asked for the overlay. Every live run uses the codegen
/// overlay now that the old control bar is gone.
pub fn wanted(cli: &Cli) -> bool {
    cli.browser.ai
        || cli.browser.codegen.is_some()
        || matches!(cli.browser.gate, crate::cli::Gate::Os)
        || matches!(cli.command, Command::Live { .. })
}

/// Runs the CLI with the overlay owning the main thread.
///
/// winit insists the event loop live on the main thread, so the session gets its
/// own runtime on a background thread and this thread pumps the overlay. When
/// either side finishes the other is brought down.
#[cfg(feature = "ui")]
pub fn run_with_overlay(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let handle = xcelerate_ui::OverlayHandle::new();
    // The AI mode (`--ai` / `live`) and `--gate os` both use the codegen overlay
    // and the OS-level input gate; `--codegen <lang>` picks the language.
    let ai_mode = cli.browser.ai || matches!(cli.command, Command::Live { .. });
    let cover = ai_mode || matches!(cli.browser.gate, crate::cli::Gate::Os);
    let language = cli
        .browser
        .codegen
        .unwrap_or(crate::cli::CodegenLang::Python);
    handle.enable_codegen(into_language(language));
    if cover {
        handle.set_cover(true);
    }
    let done = Arc::new(AtomicBool::new(false));
    let session = {
        let handle = handle.clone();
        let done = Arc::clone(&done);
        // The thread returns a `String`, not the boxed error, because
        // `Box<dyn Error>` is not `Send` and so cannot cross the thread boundary.
        std::thread::spawn(move || -> Result<(), String> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let result = runtime
                .block_on(crate::commands::run(cli, handle.clone()))
                .map_err(|error| error.to_string());
            // Ask the window to close, then mark the session finished.
            handle.request_close();
            done.store(true, Ordering::SeqCst);
            result
        })
    };

    let ui = xcelerate_ui::run(handle);
    // The overlay window is gone. Either the session ended (and asked for the
    // close) or the human pressed X. In the latter case the session notices
    // within a moment and shuts the browser down, so give it a short grace
    // before exiting; nothing is left running.
    for _ in 0..50 {
        if done.load(Ordering::SeqCst) {
            if let Ok(Err(message)) = session.join() {
                return Err(message.into());
            }
            ui?;
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    ui?;
    Ok(())
}

/// Without the GUI compiled in, the overlay is skipped and the session runs here.
#[cfg(not(feature = "ui"))]
pub fn run_with_overlay(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(crate::commands::run(cli, OverlayHandle::new()))
}

#[cfg(not(feature = "ui"))]
pub use off::{Bounds, OverlayHandle};

#[cfg(not(feature = "ui"))]
#[allow(dead_code)]
mod off {
    //! No-op overlay used when the `ui` feature is disabled.

    /// The browser window's screen rectangle, in physical pixels.
    #[derive(Clone, Copy, Debug)]
    pub struct Bounds {
        pub x: f64,
        pub y: f64,
        pub width: f64,
        pub height: f64,
    }

    /// Inert handle: records nothing and never reports a command.
    #[derive(Clone, Default)]
    pub struct OverlayHandle;

    impl OverlayHandle {
        pub fn new() -> Self {
            Self
        }

        pub fn set_bounds(&self, _bounds: Option<Bounds>) {}

        pub fn request_close(&self) {}

        pub fn is_closed(&self) -> bool {
            false
        }
    }
}

/// A session step, mirrored into the codegen overlay when one is active.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "ui"), allow(dead_code))]
pub enum Step {
    Open(String),
    Click(String),
    Fill(String, String),
    Hover(String),
    Wait(String),
    Screenshot(String),
}

impl Step {
    /// Parses a session command into a step, when it maps cleanly.
    ///
    /// Only selector-based commands are recorded: index clicks (`click 3`) and
    /// text clicks (`click-text`) carry no CSS selector, so they are left out of
    /// the generated script rather than emitted as something that would not run.
    pub fn parse(verb: &str, rest: &str) -> Option<Step> {
        let rest = rest.trim();
        match verb {
            "open" | "goto" => (!rest.is_empty()).then(|| Step::Open(rest.to_string())),
            "click" if !rest.is_empty() && rest.parse::<u32>().is_err() => {
                Some(Step::Click(rest.to_string()))
            }
            "fill" => rest
                .split_once(char::is_whitespace)
                .map(|(selector, text)| Step::Fill(selector.to_string(), text.trim().to_string())),
            "hover" if !rest.is_empty() => Some(Step::Hover(rest.to_string())),
            "wait" if !rest.is_empty() && rest.parse::<u64>().is_err() => {
                Some(Step::Wait(rest.to_string()))
            }
            "shot" | "screenshot" => Some(Step::Screenshot(if rest.is_empty() {
                "screenshot.png".to_string()
            } else {
                rest.to_string()
            })),
            _ => None,
        }
    }
}

/// Mirrors a successful step into the codegen overlay, if one is showing.
#[cfg(feature = "ui")]
pub fn record_step(overlay: &OverlayHandle, step: Step) {
    if overlay.codegen_language().is_none() {
        return;
    }
    overlay.push_action(into_action(step));
}

/// Mirrors a successful step into the codegen overlay, if one is showing.
#[cfg(not(feature = "ui"))]
pub fn record_step(_overlay: &OverlayHandle, _step: Step) {}

#[cfg(feature = "ui")]
fn into_action(step: Step) -> xcelerate_ui::Action {
    use xcelerate_ui::Action;
    match step {
        Step::Open(url) => Action::Navigate { url },
        Step::Click(selector) => Action::Click { selector },
        Step::Fill(selector, text) => Action::Fill { selector, text },
        Step::Hover(selector) => Action::Hover { selector },
        Step::Wait(selector) => Action::WaitFor { selector },
        Step::Screenshot(path) => Action::Screenshot { path },
    }
}

#[cfg(feature = "ui")]
fn into_language(lang: crate::cli::CodegenLang) -> xcelerate_ui::Language {
    use crate::cli::CodegenLang;
    use xcelerate_ui::Language;
    match lang {
        CodegenLang::Rust => Language::Rust,
        CodegenLang::Python => Language::Python,
        CodegenLang::Javascript => Language::JavaScript,
        CodegenLang::Csharp => Language::CSharp,
        CodegenLang::Kotlin => Language::Kotlin,
        CodegenLang::Java => Language::Java,
        CodegenLang::Swift => Language::Swift,
        CodegenLang::Ruby => Language::Ruby,
        CodegenLang::Dart => Language::Dart,
        CodegenLang::Go => Language::Go,
        CodegenLang::Powershell => Language::PowerShell,
    }
}
