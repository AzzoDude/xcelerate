//! Turning a recorded session run into a codegen script.

use xcelerate_codegen::{Action, Language, Selector};
use xcelerate_interpreter::interact::looks_like_a_selector;

use crate::cli::CodegenLang;

use super::input::parse_coordinates;

/// Maps a recorded session command into a codegen [`Action`], when it maps
/// cleanly, tracking the last selector so focus-scoped steps can be recorded.
///
/// Index clicks (`click 3`) and bare `type` carry no portable locator, so they
/// are left out. `press` / `submit` act on the focused element, which after a
/// `fill` is the field just typed into, so they are recorded against `last`.
pub(crate) fn record_action(tokens: &[String], last: &mut Option<Selector>) -> Option<Action> {
    let verb = tokens
        .first()
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let rest = tokens.get(1..).map_or(String::new(), |rest| rest.join(" "));
    let rest = rest.trim();
    match verb.as_str() {
        "open" | "goto" if !rest.is_empty() => Some(Action::Navigate {
            url: rest.to_string(),
        }),
        // `click` / `tap` take an index, a selector, or visible text. An index
        // carries no portable locator, so it is left out of the recording.
        "click" | "tap" if !rest.is_empty() && rest.parse::<u32>().is_err() => {
            let selector = if looks_like_a_selector(rest) {
                Selector::parse(rest)
            } else {
                Selector::text(rest)
            };
            *last = Some(selector.clone());
            Some(Action::Click { selector })
        }
        "hover" if !rest.is_empty() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::Hover { selector })
        }
        // `mouse [click] <target>`: `mouse click X` records a click, `mouse X` a
        // hover. Coordinates and snapshot indices carry no portable locator, so
        // they are left out.
        "mouse" => {
            let action = tokens.get(1).map(|t| t.to_ascii_lowercase());
            let is_click = action.as_deref() == Some("click");
            let offset = if matches!(action.as_deref(), Some("click" | "move")) {
                2
            } else {
                1
            };
            let target = tokens.get(offset..).map(|rest| rest.join(" "))?;
            let target = target.trim();
            if target.is_empty()
                || parse_coordinates(target).is_some()
                || target.parse::<u32>().is_ok()
            {
                return None;
            }
            let selector = if looks_like_a_selector(target) {
                Selector::parse(target)
            } else {
                Selector::text(target)
            };
            *last = Some(selector.clone());
            Some(if is_click {
                Action::Click { selector }
            } else {
                Action::Hover { selector }
            })
        }
        "wait" | "sleep" if !rest.is_empty() && rest.parse::<u64>().is_err() => {
            let selector = Selector::parse(rest);
            *last = Some(selector.clone());
            Some(Action::WaitFor { selector })
        }
        "fill" => {
            // Tokens, not `rest`: a quoted selector may contain spaces.
            let selector = tokens.get(1)?;
            let text = tokens.get(2..).filter(|rest| !rest.is_empty())?.join(" ");
            let selector = Selector::parse(selector);
            *last = Some(selector.clone());
            Some(Action::Fill {
                selector,
                text: text.trim().to_string(),
            })
        }
        // `press` / `submit` act on the focused element. After a `fill` that is
        // the field just typed into, so record it against the last selector -
        // otherwise the step would be lost from the generated script.
        "press" | "submit" | "send" => {
            let key = if verb == "press" {
                rest.to_string()
            } else {
                "Enter".to_string()
            };
            last.clone().map(|selector| Action::Press { selector, key })
        }
        "shot" | "screenshot" | "shot-full" | "screenshot-full" => Some(Action::Screenshot {
            path: if rest.is_empty() {
                "screenshot.png".to_string()
            } else {
                rest.to_string()
            },
        }),
        _ => None,
    }
}

/// Maps the CLI `--codegen` language onto the codegen target.
pub(crate) fn codegen_language(lang: CodegenLang) -> Language {
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

/// Renders the recorded run and writes it to `output`, or prints it when no
/// path was given.
pub(crate) fn emit_codegen(
    lang: CodegenLang,
    actions: &[Action],
    output: Option<&std::path::Path>,
) {
    let language = codegen_language(lang);
    let code = language.generate(actions);
    match output {
        Some(path) => match std::fs::write(path, code.as_bytes()) {
            Ok(()) => println!(
                "codegen: wrote {} ({} actions, {})",
                path.display(),
                actions.len(),
                language.label()
            ),
            Err(error) => eprintln!("codegen: could not write {}: {error}", path.display()),
        },
        None => println!(
            "\n# ---- generated {} ({} actions) ----\n{code}",
            language.label(),
            actions.len()
        ),
    }
}
