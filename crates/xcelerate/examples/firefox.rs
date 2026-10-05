//! Launch Firefox and drive it over WebDriver BiDi.
//!
//! ```sh
//! cargo run -p xcelerate --example firefox --no-default-features
//! ```

use xcelerate::firefox::{FirefoxBrowser, FirefoxConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = FirefoxBrowser::launch(FirefoxConfig::default()).await?;
    println!("engine:  {}", browser.version());

    let page = browser.clone().new_page("about:blank".to_string()).await?;
    page.goto("https://example.com".to_string()).await?;
    println!("context: {}", page.context_id());
    println!("url:     {}", page.url().await?);
    println!("title:   {}", page.title().await?);

    browser.close().await?;
    println!("done: Xcelerate drove Firefox over WebDriver BiDi");
    Ok(())
}
