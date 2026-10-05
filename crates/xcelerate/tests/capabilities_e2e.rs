//! End-to-end tests for the agent capabilities (markdown, search, challenge
//! detection, indexed snapshots/clicks, waits, HAR, per-call timeouts, policy).
//!
//! They launch a real Chromium/Edge and drive an in-memory page (`set_content`),
//! so no network is required. Override the browser with `XCELERATE_CHROME`. Run:
//!
//!     cargo test -p xcelerate --test capabilities_e2e -- --nocapture

use std::sync::Arc;

use xcelerate::{Browser, BrowserConfig, Page, configure_domain_policy};

const HTML: &str = r#"<!doctype html>
<html>
  <head><title>Capability Test</title></head>
  <body>
    <h1>Welcome</h1>
    <p>Hello <a href="https://example.com/x">link</a></p>
    <button id="b" onclick="window.__clicked = true">Sign in</button>
    <input placeholder="Email">
    <ul><li>Alpha</li><li>Beta</li></ul>
  </body>
</html>"#;

fn config() -> BrowserConfig {
    BrowserConfig {
        headless: true,
        detached: false,
        executable_path: std::env::var("XCELERATE_CHROME").ok(),
        plugins: None,
    }
}

async fn fixture(html: &str) -> (Arc<Browser>, Arc<Page>) {
    let browser = Browser::launch(config()).await.expect("launch");
    let page = Arc::clone(&browser)
        .new_page("about:blank".to_string())
        .await
        .expect("new_page");
    page.set_content(html.to_string())
        .await
        .expect("set_content");
    (browser, page)
}

#[tokio::test]
async fn markdown_find_and_challenge() {
    let (browser, page) = fixture(HTML).await;

    let markdown = page.markdown().await.expect("markdown");
    assert!(
        markdown.contains("# Welcome"),
        "missing heading:\n{markdown}"
    );
    assert!(
        markdown.contains("[link](https://example.com/x)"),
        "missing link:\n{markdown}"
    );
    assert!(markdown.contains("Sign in"), "missing button:\n{markdown}");

    assert_eq!(
        page.find_text("Sign in".to_string()).await.expect("find"),
        1
    );
    assert!(page.count_text("Alpha".to_string()).await.expect("count") >= 1);

    let report = page.detect_challenge().await.expect("challenge");
    assert!(
        !report.detected,
        "plain page flagged as a challenge: {:?}",
        report.signals
    );

    browser.close().await.ok();
}

#[tokio::test]
async fn snapshot_index_click_and_highlight() {
    let (browser, page) = fixture(HTML).await;

    let snapshot = page.agent_snapshot().await.expect("agent_snapshot");
    assert!(
        snapshot.contains("Sign in"),
        "snapshot missing button:\n{snapshot}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&page.snapshot_json().await.expect("snapshot_json")).expect("json");
    let index = json
        .as_array()
        .expect("array")
        .iter()
        .find(|entry| entry.get("name").and_then(|n| n.as_str()) == Some("Sign in"))
        .and_then(|entry| entry.get("index").and_then(|i| i.as_u64()))
        .expect("button index") as u32;

    page.highlight_index(index).await.expect("highlight_index");
    assert!(page.highlight_all().await.expect("highlight_all") >= 1);
    page.clear_highlights().await.expect("clear_highlights");

    Arc::clone(&page)
        .click_index(index)
        .await
        .expect("click_index");
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert!(
        page.evaluate_bool("window.__clicked === true".to_string())
            .await
            .expect("evaluate")
    );

    browser.close().await.ok();
}

#[tokio::test]
async fn waits_har_and_per_call_timeout() {
    let (browser, page) = fixture(HTML).await;

    page.wait_for_network_idle(200, 5_000)
        .await
        .expect("network idle");
    page.wait_for_dom_stable(200, 5_000)
        .await
        .expect("dom stable");

    let har_path = std::env::temp_dir().join("xcelerate_capabilities_test.har");
    let har_path = har_path.to_string_lossy().into_owned();
    page.start_har_recording().await.expect("start_har");
    let _ = page
        .evaluate_string("document.body.appendChild(document.createElement('div'))".to_string())
        .await;
    page.stop_har_recording().await.expect("stop_har");
    let saved = page.save_har(har_path.clone()).await.expect("save_har");
    let har: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&saved).expect("read har"))
            .expect("parse har");
    assert_eq!(
        har.pointer("/log/version").and_then(|v| v.as_str()),
        Some("1.2")
    );
    assert!(
        har.pointer("/log/entries")
            .and_then(|v| v.as_array())
            .is_some()
    );
    let _ = std::fs::remove_file(&saved);

    let result = page
        .execute_cdp_cmd_timeout(
            "Runtime.evaluate".to_string(),
            "{\"expression\":\"1 + 1\",\"returnByValue\":true}".to_string(),
            5_000,
        )
        .await
        .expect("cdp timeout call");
    assert!(result.contains('2'), "unexpected eval result: {result}");

    browser.close().await.ok();
}

#[tokio::test]
async fn domain_policy_blocks_navigation() {
    let (browser, page) = fixture(HTML).await;

    configure_domain_policy(vec![], vec!["blocked.example.invalid".to_string()]);
    let blocked = page
        .navigate("https://blocked.example.invalid/".to_string())
        .await;
    // Reset immediately so a parallel test is never affected.
    configure_domain_policy(vec![], vec![]);

    let error = blocked.expect_err("navigation should have been blocked");
    assert!(
        error.to_string().contains("blocked"),
        "unexpected error: {error}"
    );

    browser.close().await.ok();
}
