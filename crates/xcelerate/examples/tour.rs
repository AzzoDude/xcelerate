//! A slow, *headful* walkthrough: open a real Chromium window and watch
//! Xcelerate drive it through (1) a page that fingerprints automation and
//! (2) a live search where it types, submits, and reads the results.
//!
//! ```sh
//! cargo run -p xcelerate --example tour
//! ```
//!
//! Screenshots land in `.research/demo/`.

use std::sync::Arc;
use std::time::Duration;

use xcelerate::{Browser, BrowserConfig, LaunchOptions, Page, configure_launch_options};

const OUT: &str = ".research/demo";
const DETECT_PAGE: &str = "https://bot.sannysoft.com/";
const SEARCH_PAGE: &str = "https://duckduckgo.com/";
const QUERY: &str = "xcelerate fast lightweight browser automation";

fn say(msg: &str) {
    println!("\n=== {msg} ===");
}

async fn pause(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

async fn shot(page: &Arc<Page>, name: &str) {
    match page.screenshot().await {
        Ok(bytes) => {
            let path = format!("{OUT}/{name}.png");
            match std::fs::write(&path, &bytes) {
                Ok(()) => println!("  screenshot -> {path} ({} KiB)", bytes.len() / 1024),
                Err(e) => println!("  could not write {path}: {e}"),
            }
        }
        Err(e) => println!("  screenshot failed: {e}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(OUT)?;

    configure_launch_options(LaunchOptions {
        // A visible, sensibly-sized window so the walkthrough is watchable.
        extra_args: vec![
            "--window-size=1440,900".to_string(),
            "--window-position=60,40".to_string(),
        ],
        ..Default::default()
    });

    say("launch Chromium (headful)");
    let browser = Browser::launch(BrowserConfig {
        headless: false, // the point: you watch it work
        detached: false,
        executable_path: None, // let Xcelerate locate Chrome
        plugins: None,
    })
    .await?;
    println!("  up: {}", browser.version().await?);
    pause(800).await;

    let page = Arc::clone(&browser)
        .new_page("about:blank".to_string())
        .await?;

    // ---- 1. The hard exam: a page built to catch automation ----------------
    say("navigation 1/2: bot.sannysoft.com (automation-detection exam)");
    page.navigate(DETECT_PAGE.to_string()).await?;
    page.wait_for_network_idle(500, 15_000).await.ok();
    pause(1200).await;
    println!("  title: {}", page.title().await?);
    shot(&page, "01-sannysoft").await;

    let rows = page
        .evaluate_json(
            r#"(() => Array.from(document.querySelectorAll("table tr"))
                 .map(tr => tr.innerText.replace(/\s+/g, " ").trim())
                 .filter(t => t.length > 0)
                 .slice(0, 12))()"#
                .to_string(),
        )
        .await?;
    println!("  verdict table (first rows): {rows}");

    let report = page.detect_challenge().await?;
    println!(
        "  anti-bot report: detected={} vendors={:?} signals={:?}",
        report.detected, report.vendors, report.signals
    );
    pause(1500).await;

    // ---- 2. Live interaction: slow typing, submit, read results ------------
    say("navigation 2/2: duckduckgo.com (type slowly, submit, read results)");
    page.navigate(SEARCH_PAGE.to_string()).await?;
    page.wait_for_network_idle(500, 15_000).await.ok();
    pause(1000).await;
    shot(&page, "02-ddg-home").await;

    // DuckDuckGo's search box is a <textarea>; other engines use <input>. Try
    // a few candidates so the walkthrough is not brittle to a redesign.
    let candidates = [
        "textarea[name='q']",
        "input[name='q']",
        "#searchbox_input",
        "input[type='search']",
        "input[type='text']",
    ];
    let mut found = None;
    for selector in candidates {
        if let Ok(element) = page.clone().find_element(selector.to_string()).await {
            println!("  search box: {selector}");
            found = Some(element);
            break;
        }
    }
    let input = found.ok_or_else(|| "no search box found".to_string())?;
    println!("  typing \"{QUERY}\" slowly (50 ms/char)...");
    Arc::clone(&input).type_text(QUERY.to_string()).await?;
    pause(700).await;
    shot(&page, "03-typed").await;

    println!("  pressing Enter");
    Arc::clone(&input).press("Enter".to_string()).await?;
    page.wait_for_network_idle(500, 20_000).await.ok();
    pause(1200).await;
    println!("  landed on: {}", page.url().await?);
    shot(&page, "04-results").await;

    let titles = page
        .evaluate_json(
            r#"(() => Array.from(document.querySelectorAll(
                   "[data-testid='result-title-a'], a.result__a"))
                 .slice(0, 5)
                 .map(a => a.innerText.trim()))()"#
                .to_string(),
        )
        .await?;
    println!("  top results: {titles}");

    // An agent-friendly view of the results page, plus the index-click API.
    let snap = page.agent_snapshot().await?;
    let preview: String = snap.lines().take(12).collect::<Vec<_>>().join("\n");
    println!("  agent snapshot (first lines):\n{preview}");

    say("done - closing the window");
    pause(1500).await;
    browser.close().await.ok();
    Ok(())
}
