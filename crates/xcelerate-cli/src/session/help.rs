//! The session's help text and its snapshot renderer.

use super::input::quote_selector;

/// Prints the session command list shown by `help` / `?`.
pub(crate) fn print_session_help() {
    println!(
        "commands:\n\
         \x20 open <url>                     navigate the live page\n\
         \x20 back | reload                  history / refresh\n\
         \x20 title | url | text | markdown  read the page\n\
         \x20 content                        raw HTML\n\
         \x20 snapshot                       indexed interactive elements\n\
         \x20 click <index|selector|text>   click by snapshot index, CSS selector, or visible text\n\
         \x20 tap <selector|text>           same pick, but a DOM click that never moves the mouse\n\
         \x20 click-xy <x> <y>              raw coordinate click (canvas / embedded)\n\
         \x20 upload <selector> <path>      set a file input to a local file\n\
         \x20 cookie [get] [name]           print cookies as JSON (all, or one by name)\n\
         \x20 cookie set <name> <value> [domain] [path]  store a cookie\n\
         \x20 cookie add <json>             add a cookie (or array) with full attributes\n\
         \x20 cookie delete <name> | clear  remove one cookie / all cookies\n\
         \x20 fill <selector|index> <text>  focus + type slowly (50 ms/char)\n\
         \x20 select <selector> <value>     choose an option in a native <select>\n\
         \x20 type <text>                    type into the focused element\n\
         \x20 press <key>                    press a key on the focused element\n\
         \x20 submit                         press Enter on the focused element (send)\n\
         \x20 hover <selector>               move the mouse over an element\n\
         \x20 mouse [click] <index|selector|text>  move the cursor there (or `mouse <x> <y>`); `click` also clicks\n\
         \x20 media                          list the page's images/video/audio as JSON\n\
         \x20 download <url> <path>          fetch a URL (file or HLS) through the browser and save it\n\
         \x20 scroll <px|up|down|top|bottom> scroll the page\n\
         \x20 find <text>                    how many elements contain the text\n\
         \x20 wait <ms|selector>             sleep, or wait for an element\n\
         \x20 wait-stable [ms]               wait until the DOM stops changing\n\
         \x20 wait-idle [ms]                 wait until the network goes quiet\n\
         \x20 wait-random <min> <max>        sleep a uniform random hold (ms)\n\
         \x20 challenge                       detect an anti-bot / CAPTCHA challenge\n\
         \x20 await-human [s]                 wait until a person clears the challenge\n\
         \x20 eval <js>                      evaluate JavaScript, print the result\n\
         \x20 guard <path.js>               block popups/ads on this page and every new one\n\
         \x20 tabs                          list targets (id, type, url)\n\
         \x20 new-tab [url]                 open a new tab and make it active\n\
         \x20 switch <n|targetId>            make a tab active (index from `tabs`; no arg cycles)\n\
         \x20 close-tab <index|targetId>     close a tab (index or id from `tabs`)\n\
         \x20 shot [path]                    viewport screenshot (default screenshot.png)\n\
         \x20 shot-full [path]               full-page screenshot\n\
         \x20 record [path]                   start a video (only with --codegen)\n\
         \x20 stop-record                     stop the video (only with --codegen)\n\
         \x20 codegen [preview|out|undo|remove <n>|clear|status]\n\
         \x20                                 preview / edit the recorded script\n\
         \x20 done [summary]                  mark the task complete\n\
         \x20 help | quit                    (an AI run must `done` or `quit!` to exit)"
    );
}

/// Renders `media_json` output as one line per item, e.g.
/// `[1] <image> 720x404 https://…` — the readable counterpart to `snapshot`.
pub(crate) fn print_media(json: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => {
            println!("{json}");
            return;
        }
    };
    let Some(items) = parsed.get("media").and_then(serde_json::Value::as_array) else {
        println!("{json}");
        return;
    };
    if items.is_empty() {
        println!("(no media)");
        return;
    }
    for (index, item) in items.iter().enumerate() {
        let kind = item
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("media");
        let width = item
            .get("width")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let height = item
            .get("height")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let url = item
            .get("url")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let dims = if width > 0 && height > 0 {
            format!(" {width}x{height}")
        } else {
            String::new()
        };
        // A compact flag list beats a wall of JSON keys.
        let mut flags = Vec::new();
        if item.get("background").and_then(serde_json::Value::as_bool) == Some(true) {
            flags.push("background");
        }
        if item.get("streaming").and_then(serde_json::Value::as_bool) == Some(true) {
            flags.push("streaming (blob, no URL)");
        }
        let suffix = if flags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", flags.join(", "))
        };
        if url.is_empty() {
            println!("[{index}] <{kind}>{dims}{suffix}");
        } else {
            println!("[{index}] <{kind}>{dims}{suffix}  {url}");
        }
    }
}

/// Renders `snapshot_json` output as one line per element, e.g.
/// `[3] <link> "Sign in"`.
pub(crate) fn print_snapshot(json: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => {
            println!("{json}");
            return;
        }
    };
    let Some(entries) = parsed.as_array() else {
        println!("{json}");
        return;
    };
    if entries.is_empty() {
        println!("(no interactive elements)");
        return;
    }
    for entry in entries {
        let index = entry
            .get("index")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let role = entry
            .get("role")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let name = entry
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let value = entry
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        // A derived CSS selector, when one exists, so the element can be driven by
        // `fill "#email" …` / `click "#submit"` instead of only by index.
        let selector = entry
            .get("selector")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let suffix = if selector.is_empty() {
            String::new()
        } else {
            format!("  {}", quote_selector(selector))
        };
        if value.is_empty() {
            println!("[{index}] <{role}> {name:?}{suffix}");
        } else {
            println!("[{index}] <{role}> {name:?} = {value:?}{suffix}");
        }
    }
}
