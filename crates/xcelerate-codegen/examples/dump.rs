//! Prints a recorded session rendered in all eleven languages: `cargo run -p
//! xcelerate-codegen --example dump`. Handy for eyeballing selector handling
//! (CSS, id, XPath, absolute "full" XPath, text) without a live browser.

use xcelerate_codegen::{Action, Language, Selector};

fn main() {
    let actions = vec![
        Action::Navigate {
            url: "https://example.com".into(),
        },
        Action::Click {
            selector: Selector::css("#go"),
        },
        Action::Fill {
            selector: Selector::id("q"),
            text: "hello".into(),
        },
        Action::Click {
            selector: Selector::xpath("/html/body/div[2]/button"),
        },
        Action::Hover {
            selector: Selector::xpath("//a[normalize-space(text())='Docs']"),
        },
        Action::Click {
            selector: Selector::text("Sign in"),
        },
        Action::Screenshot {
            path: "shot.png".into(),
        },
    ];

    for language in Language::ALL {
        println!(
            "// ===== {} ({}) =====",
            language.label(),
            language.file_name()
        );
        println!("{}", language.generate(&actions));
    }
}
