using uniffi.xcelerate_core;

Console.WriteLine("--- Xcelerate C# UniFFI Test App (Bot Check) ---");

try
{
    Console.WriteLine("[API] Browser.Launch (Headless: false, Stealth: true)");
    var config = new BrowserConfig(headless: false, stealth: true, detached: false, executablePath: null);
    using var browser = await Browser.Launch(config);
    
    Console.WriteLine("[API] Browser.NewPage");
    using var page = await browser.NewPage("https://pixelscan.net/bot-check");

    Console.WriteLine("Waiting 15 seconds for Pixelscan to run its bot checks...");
    await Task.Delay(15000);

    Console.WriteLine("[API] Page.Screenshot");
    byte[] ss = await page.Screenshot();
    System.IO.File.WriteAllBytes("pixelscan_result.png", ss);
    Console.WriteLine($"Screenshot saved to pixelscan_result.png (Size: {ss.Length})");

    Console.WriteLine("[API] Browser.Close");
    await browser.Close();

    Console.WriteLine("\n[SUCCESS] Bot check completed.");
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
