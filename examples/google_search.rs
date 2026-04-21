use xcelerate_core::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    // 1. Setup magic launching!
    let browser = Browser::launch(BrowserConfig {
        headless: false,
        ..Default::default()
    }).await?;

    // 3. User your clean API
    println!("Launching Browser and navigating to Google...");
    let page = browser.new_page("https://www.google.com".to_string()).await?;
    
    println!("Typing and Clicking...");
    page.find_element("input[name='q']".to_string())
        .await?
        .type_text("Xcelerate Rust Automation".to_string())
        .await?
        .click()
        .await?;

    println!("Process finished successfully!");
    Ok(())
}
