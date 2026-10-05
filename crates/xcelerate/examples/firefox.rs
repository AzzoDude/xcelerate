//! Launch Firefox and drive it over WebDriver BiDi.
//!
//! ```sh
//! cargo run -p xcelerate --example firefox --no-default-features -- --headful
//! ```
//!
//! Without `--headful` the browser runs headless.

use xcelerate::firefox::{FirefoxBrowser, FirefoxConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let headful = std::env::args().any(|arg| arg == "--headful");
    let config = FirefoxConfig {
        headless: !headful,
        ..Default::default()
    };

    let browser = FirefoxBrowser::launch(config).await?;
    println!(
        "engine:  {} ({} mode)",
        browser.version(),
        if headful { "headful" } else { "headless" }
    );

    let page = browser.clone().new_page("about:blank".to_string()).await?;
    page.goto("https://example.com".to_string()).await?;
    println!("context: {}", page.context_id());
    println!("url:     {}", page.url().await?);
    println!("title:   {}", page.title().await?);

    browser.close().await?;
    println!("done: Xcelerate drove Firefox over WebDriver BiDi");
    Ok(())
}
