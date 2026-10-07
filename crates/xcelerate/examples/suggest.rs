//! Handle a live autocomplete dropdown (the "suggestions" list that appears
//! while you type, like YouTube's search recommendations).
//!
//! The dropdown is a race: it shows up *after* a network/debounce delay,
//! re-renders on every keystroke, and is often in a portal or shadow root. The
//! robust pattern is: type incrementally, wait for the list to *settle*, read
//! it, then select by keyboard (or by the stable indexed snapshot).
//!
//! ```sh
//! cargo run -p xcelerate --example suggest -- youtube
//! cargo run -p xcelerate --example suggest -- ddg
//! ```

use std::sync::Arc;
use std::time::Duration;

use xcelerate::{Browser, BrowserConfig, Element, LaunchOptions, Page, configure_launch_options};

const QUERY: &str = "rust programming";

/// Reads visible `[role="listbox"]` / `[role="option"]` nodes, plus a
/// class-hint fallback for engines that do not expose ARIA roles.
const READ_SUGGESTIONS: &str = r#"(() => {
  const shown = e => { const r = e.getBoundingClientRect();
                       return r.width > 0 && r.height > 0; };
  const clean = s => (s || "").replace(/\s+/g, " ").trim();

  const boxes = [...document.querySelectorAll('[role="listbox"]')].filter(shown);
  let options = [...document.querySelectorAll('[role="option"]')].filter(shown)
      .map(o => clean(o.innerText || o.textContent)).filter(Boolean);

  if (options.length === 0) {
    // Fallback: engines that render a plain <ul>/<li> popup.
    const hinted = [...document.querySelectorAll('ul,div')].filter(e =>
      shown(e) && /suggest|autocomplete|searchbox|sbqs|sbsb/i.test(e.className || ""));
    options = hinted.flatMap(e => [...e.querySelectorAll('li,[role="option"]')])
                    .filter(shown).map(o => clean(o.textContent)).filter(Boolean);
  }
  return { listboxes: boxes.length, options: [...new Set(options)].slice(0, 8) };
})()"#;

async fn pause(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

async fn read_suggestions(page: &Arc<Page>) -> String {
    page.evaluate_json(READ_SUGGESTIONS.to_string())
        .await
        .unwrap_or_else(|e| format!("<read failed: {e}>"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let site = std::env::args().nth(1).unwrap_or_else(|| "ddg".into());
    let (url, candidates): (&str, &[&str]) = match site.as_str() {
        "youtube" => (
            "https://www.youtube.com/",
            &[
                "input#search",
                "input[name='search_query']",
                "ytd-searchbox input",
            ],
        ),
        _ => (
            "https://duckduckgo.com/",
            &["textarea[name='q']", "input[name='q']", "#searchbox_input"],
        ),
    };

    configure_launch_options(LaunchOptions {
        extra_args: vec![
            "--window-size=1440,900".into(),
            "--window-position=60,40".into(),
        ],
        ..Default::default()
    });

    println!("=== {site}: {url} ===");
    let browser = Browser::launch(BrowserConfig {
        headless: false,
        detached: false,
        executable_path: None,
        plugins: None,
    })
    .await?;

    let page = Arc::clone(&browser)
        .new_page("about:blank".to_string())
        .await?;
    page.navigate(url.to_string()).await?;
    page.wait_for_network_idle(500, 20_000).await.ok();
    pause(1500).await;

    // Locate the search box across candidates (inputs, textareas, shadow DOM).
    let mut input: Option<Arc<Element>> = None;
    for selector in candidates {
        if let Ok(element) = page.clone().find_element(selector.to_string()).await {
            println!("  search box: {selector}");
            input = Some(element);
            break;
        }
    }
    let input = input.ok_or_else(|| "no search box found".to_string())?;

    // Type one character at a time and show the dropdown updating live.
    println!("  typing \"{QUERY}\" and watching the dropdown:");
    for ch in QUERY.chars() {
        Arc::clone(&input).type_text(ch.to_string()).await?;
        pause(160).await; // let the engine's debounce + XHR land
        let live = read_suggestions(&page).await;
        println!("    after '{ch}': {live}");
    }

    // Wait until the list *settles*: at least 3 suggestions, on screen.
    println!("  waiting for the list to settle...");
    page.wait_for_function(
        r#"document.querySelectorAll('[role="option"]').length >= 3
           || [...document.querySelectorAll('li')].filter(li => li.offsetParent).length >= 3"#
            .to_string(),
        10_000,
    )
    .await
    .ok();
    pause(400).await;

    let final_state = read_suggestions(&page).await;
    println!("  dropdown contents: {final_state}");

    // Select the first suggestion. Keyboard is the most robust: it neither
    // depends on pixel coordinates nor on the item staying in the DOM. (The
    // alternative is `page.snapshot_json()` + `page.click_index(i)`, which pins
    // the node by `backendNodeId` so a re-render cannot make the click drift.)
    println!("  selecting the highlighted suggestion with ArrowDown + Enter");
    Arc::clone(&input).press("ArrowDown".to_string()).await?;
    pause(300).await;
    Arc::clone(&input).press("Enter".to_string()).await?;
    page.wait_for_network_idle(500, 20_000).await.ok();
    pause(1200).await;

    println!("  landed on: {}", page.url().await?);
    let bytes = page.screenshot().await?;
    let path = format!(".research/demo/suggest-{site}.png");
    std::fs::create_dir_all(".research/demo")?;
    std::fs::write(&path, &bytes)?;
    println!("  screenshot -> {path}");

    browser.close().await.ok();
    Ok(())
}
