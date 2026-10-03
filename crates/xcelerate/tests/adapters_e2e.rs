//! End-to-end adapter tests against a real website.
//!
//! These launch a real Chromium/Edge via the core `Browser`, then exercise the
//! API-style adapters' element-query surface (`find_element(s)`,
//! `query_selector`, `query_selector_all`, `$`/`$$`).
//!
//! They need a local Chrome/Edge and network access; override the target with
//! `XCELERATE_TEST_URL`. Run with:
//!
//!     cargo test -p xcelerate --test adapters_e2e -- --nocapture

use xcelerate::BrowserConfig;
use xcelerate::adapters::{playwright, puppeteer, selenium};

fn test_url() -> String {
    std::env::var("XCELERATE_TEST_URL").unwrap_or_else(|_| "https://example.com".to_string())
}

fn config() -> BrowserConfig {
    BrowserConfig {
        headless: true,
        detached: false,
        executable_path: std::env::var("XCELERATE_CHROME").ok(),
        plugins: None,
    }
}

#[tokio::test]
async fn playwright_query_selector_all() {
    let browser = playwright::launch(Some(config())).await.expect("launch");
    let page = browser.new_page(test_url()).await.expect("new_page");

    let title = page.title().await.expect("title");
    assert!(
        !title.trim().is_empty(),
        "expected a non-empty page title, got {title:?}"
    );

    // querySelectorAll -> Vec<Locator>
    let paragraphs = page
        .query_selector_all("p".to_string())
        .await
        .expect("query_selector_all");
    assert!(
        !paragraphs.is_empty(),
        "expected at least one <p> on {}",
        test_url()
    );

    // querySelector -> Locator, and read it back
    let heading = page
        .query_selector("p".to_string())
        .await
        .expect("query_selector");
    let text = heading.inner_text().await.expect("inner_text");
    assert!(!text.trim().is_empty(), "expected non-empty <p> text");

    browser.close().await.ok();
}

#[tokio::test]
async fn puppeteer_query_selector_all() {
    let browser = puppeteer::launch(Some(config())).await.expect("launch");
    let page = browser.newPage(test_url()).await.expect("new_page");

    // $$ (querySelectorAll) -> Vec<ElementHandle>
    let handles = page.query_selector_all("p".to_string()).await.expect("$$");
    assert!(
        !handles.is_empty(),
        "expected at least one <p> via $$ on {}",
        test_url()
    );

    // $ (querySelector) -> ElementHandle
    let heading = page.query_selector("p".to_string()).await.expect("$");
    let html = heading
        .evaluate("el => el.textContent".to_string())
        .await
        .expect("evaluate");
    assert!(!html.trim().is_empty(), "expected non-empty <p> content");

    browser.close().await.ok();
}

#[tokio::test]
async fn selenium_find_elements() {
    let driver = selenium::launch(Some(config())).await.expect("launch");
    driver.get(test_url()).await.expect("get");

    // find_elements -> Vec<WebElement>
    let elements = driver
        .find_elements("css selector".to_string(), Some("p".to_string()))
        .await
        .expect("find_elements");
    assert!(
        !elements.is_empty(),
        "expected find_elements to return <p> nodes on {}",
        test_url()
    );

    // find_element -> WebElement
    let first = driver
        .find_element("css selector".to_string(), Some("p".to_string()))
        .await
        .expect("find_element");
    let text = first.text().await.expect("text");
    assert!(
        !text.trim().is_empty(),
        "expected non-empty WebElement text"
    );

    driver.quit().await.ok();
}
