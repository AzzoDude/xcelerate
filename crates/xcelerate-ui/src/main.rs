//! Standalone codegen overlay demo: `cargo run -p xcelerate-ui`.
//!
//! Seeds a short recording so the codegen window can be seen without a live
//! browser session. Use the language menu in the top navbar to render the same
//! steps in any of the eleven targets.

use xcelerate_ui::{Action, Language, OverlayHandle};

/// A small recording: open, search, submit, read the result, capture it.
fn recording() -> Vec<Action> {
    vec![
        Action::Navigate {
            url: "https://example.com".to_string(),
        },
        Action::Fill {
            selector: "#search".to_string(),
            text: "xcelerate".to_string(),
        },
        Action::Click {
            selector: "button[type=submit]".to_string(),
        },
        Action::WaitFor {
            selector: "main .results".to_string(),
        },
        Action::Text {
            selector: "main .results h1".to_string(),
        },
        Action::Screenshot {
            path: "results.png".to_string(),
        },
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = OverlayHandle::new();
    handle.enable_codegen(Language::Python);
    handle.set_recording(true);
    for action in recording() {
        handle.push_action(action);
    }
    xcelerate_ui::run(handle)
}
