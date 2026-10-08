//! Input parsing and the small helpers shared by the verb groups.

use std::sync::Arc;

use xcelerate::Page;

/// Splits a session statement into the `(selector, text)` pair used by `fill`
/// and `upload`: the token right after the verb is the selector and the rest are
/// rejoined with single spaces, so a quoted selector that itself contains spaces
/// (`fill '[aria-label="Email address"]' hi`) stays intact. `None` when either
/// half is missing.
pub(crate) fn split_selector_text(tokens: &[String]) -> Option<(String, String)> {
    let selector = tokens.get(1)?;
    let rest = tokens.get(2..)?;
    if rest.is_empty() {
        return None;
    }
    Some((selector.clone(), rest.join(" ")))
}

/// Renders a selector as a quoted string literal so it can be copied verbatim into
/// a command (`fill '[name="email"]' …`). Single quotes unless the selector
/// itself contains one, in which case double quotes with `\"` escapes are used.
pub(crate) fn quote_selector(selector: &str) -> String {
    if selector.contains('\'') {
        format!("\"{}\"", selector.replace('"', "\\\""))
    } else {
        format!("'{selector}'")
    }
}

/// Reports that a text-based click matched nothing.
pub(crate) fn report_no_text_match(text: &str) {
    println!("no visible element contains {text:?}");
}

/// Parses a bare `"x y"` pair into cursor coordinates, when both parts are
/// numbers and there is nothing else.
pub(crate) fn parse_coordinates(value: &str) -> Option<(f64, f64)> {
    let mut parts = value.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((x, y))
}

/// Splits a `mouse` argument into `(click, target)`.
///
/// A leading `click` or `move` token picks the action: `mouse click "Sign in"`
/// moves the mouse to the target and clicks it, while `mouse "Sign in"` (or
/// `mouse move "Sign in"`) only moves. The action word is read from the lexed
/// `tokens`, so a quoted target that merely starts with "click" stays a target.
pub(crate) fn split_mouse_action(tokens: &[String], rest: &str) -> (bool, String) {
    match tokens
        .get(1)
        .map(|token| token.to_ascii_lowercase())
        .as_deref()
    {
        Some("click") => (true, tokens[2..].join(" ")),
        Some("move") => (false, tokens[2..].join(" ")),
        _ => (false, rest.to_string()),
    }
}

/// Brings a tab and its window to the front. `Target.activateTarget` (browser
/// domain) raises the window; `Page.bringToFront` is a page-scoped fallback for
/// browsers or modes that reject it.
pub(crate) async fn focus_page(page: &Arc<Page>) {
    if page.activate().await.is_err() {
        let _ = page.bring_to_front().await;
    }
}

/// Whether an error means the CDP session was detached, so re-attaching the
/// page and retrying the step can recover it.
pub(crate) fn is_session_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    text.contains("session with given id")
        || text.contains("-32001")
        || text.contains("session detached")
}

#[cfg(test)]
mod tests {
    use super::{parse_coordinates, split_selector_text};
    use crate::session::record::record_action;
    use xcelerate_interpreter::interact::looks_like_a_selector;
    use xcelerate_interpreter::lex::{Line, lex_line};

    fn tokens(line: &str) -> Vec<String> {
        match lex_line(line).expect("lexes") {
            Line::Statement(tokens) => tokens,
            other => panic!("expected a statement, got {other:?}"),
        }
    }

    #[test]
    fn splits_selector_and_text() {
        assert_eq!(
            split_selector_text(&tokens("fill #email hello")),
            Some(("#email".into(), "hello".into()))
        );
        // Text keeps its internal spaces.
        assert_eq!(
            split_selector_text(&tokens("fill #name Ada Lovelace")),
            Some(("#name".into(), "Ada Lovelace".into()))
        );
    }

    #[test]
    fn keeps_spaces_inside_a_quoted_selector() {
        assert_eq!(
            split_selector_text(&tokens("fill '[aria-label=\"Email address\"]' hi")),
            Some(("[aria-label=\"Email address\"]".into(), "hi".into()))
        );
    }

    #[test]
    fn requires_both_selector_and_text() {
        assert_eq!(split_selector_text(&tokens("fill")), None);
        assert_eq!(split_selector_text(&tokens("fill #email")), None);
    }

    #[test]
    fn parses_a_bare_coordinate_pair() {
        assert_eq!(parse_coordinates("120 90"), Some((120.0, 90.0)));
        assert_eq!(parse_coordinates("-4.5 3"), Some((-4.5, 3.0)));
        // A single number (a snapshot index) is not a coordinate pair.
        assert_eq!(parse_coordinates("3"), None);
        assert_eq!(parse_coordinates("#email"), None);
        assert_eq!(parse_coordinates("1 2 3"), None);
    }

    #[test]
    fn spots_selectors_passed_to_text_clicks() {
        assert!(looks_like_a_selector("[name=\"pass\"]"));
        assert!(looks_like_a_selector("#email"));
        assert!(looks_like_a_selector(".btn"));
        assert!(looks_like_a_selector("//a[@id='x']"));
        assert!(!looks_like_a_selector("Log in"));
        assert!(!looks_like_a_selector("Sign up"));
    }

    #[test]
    fn click_records_index_selector_and_text() {
        use xcelerate_codegen::{Action, Selector};

        // A bare index carries no portable locator, so it is left out.
        let mut last = None;
        assert_eq!(record_action(&tokens("click 3"), &mut last), None);

        // A selector-looking argument records a CSS selector.
        let mut last = None;
        assert_eq!(
            record_action(&tokens("click '#email'"), &mut last),
            Some(Action::Click {
                selector: Selector::parse("#email")
            })
        );

        // Anything else records a visible-text match, for `click` and `tap` alike.
        let mut last = None;
        assert_eq!(
            record_action(&tokens("click 'Sign in'"), &mut last),
            Some(Action::Click {
                selector: Selector::text("Sign in")
            })
        );
        let mut last = None;
        assert_eq!(
            record_action(&tokens("tap 'Accept all'"), &mut last),
            Some(Action::Click {
                selector: Selector::text("Accept all")
            })
        );
    }
}
