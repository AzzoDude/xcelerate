using uniffi.xcelerate_core;

Console.WriteLine("--- Xcelerate C# UniFFI Test App (Async + Stealth) ---");

try
{
    Console.WriteLine("Launching Browser (Headless=New + Stealth)...");
    var config = new BrowserConfig(headless: true, stealth: true, detached: false, executablePath: null);
    using var browser = await Browser.Launch(config);
    
    Console.WriteLine("Creating New Page (Testing Pixelscan Stealth)...");
    // UniFFI generated NewPage returns Task<Page>
    using var page = await browser.NewPage("https://pixelscan.net/bot-check/");
    
    Console.WriteLine("Waiting for Pixelscan analysis (approx 10s)...");
    await Task.Delay(10000); 
    
    Console.WriteLine($"Current Page Title: {await page.Title()}");
    
    Console.WriteLine("Taking Full-Page Screenshot of Pixescan results...");
    byte[] screenshot = await page.ScreenshotFull();
    await File.WriteAllBytesAsync("pixelscan_full_page.png", screenshot);
    Console.WriteLine($"Screenshot saved! Size: {screenshot.Length} bytes");
    
    Console.WriteLine("\n[SUCCESS] Test completed.");
    Console.WriteLine("The browser should have closed automatically.");
}
catch (Exception ex)
{
    Console.WriteLine($"[ERROR] {ex.Message}");
    if (ex.InnerException != null)
    {
        Console.WriteLine($"[INNER ERROR] {ex.InnerException.Message}");
    }
    Console.WriteLine(ex.StackTrace);
}

Console.WriteLine("--- Test Finished ---");
