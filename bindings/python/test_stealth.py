
import asyncio
from xcelerate import Browser, BrowserConfig

async def main():
    print("--- Xcelerate Python Stealth Test ---")
    config = BrowserConfig(headless=True, stealth=True, detached=False, executable_path=None)
    browser = await Browser.launch(config)
    
    print("Opening Pixelscan...")
    page = await browser.new_page("https://pixelscan.net/bot-check/bot-check")
    
    # Wait for result to load
    print("Waiting 10 seconds for bot check...")
    await asyncio.sleep(10)
    title = await page.title()
    print(f"Page Title: {title}")
    
    screenshot = await page.screenshot_full()
    with open("pixelscan_python.png", "wb") as f:
        f.write(bytes(screenshot))
    print("Screenshot saved to pixelscan_python.png")
    
    await browser.close()
    print("Done.")

if __name__ == "__main__":
    asyncio.run(main())
