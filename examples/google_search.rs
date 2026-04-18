use xcelerate::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    // 1. Setup magic launching!
    let (browser, handler) = Browser::launch(
        BrowserConfig::builder()
            .headless(false) // Set to true for background running
            .build()?
    ).await?;
    
    // 2. Start the engine
    tokio::spawn(handler.run());

    // 3. User your clean API
    println!("Launching Browser and navigating to Google...");
    let page = browser.new_page("https://www.google.com").await?;
    
    println!("Typing and Clicking...");
    page.find_element("input[name='q']")
        .await?
        .type_text("Xcelerate Rust Automation")
        .await?
        .click()
        .await?;

    println!("Process finished successfully!");
    Ok(())
}
