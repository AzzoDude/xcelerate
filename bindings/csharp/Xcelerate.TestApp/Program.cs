using uniffi.xcelerate;

Console.WriteLine("--- Xcelerate C# UniFFI Test App ---");

try
{
    Console.WriteLine("[TEST 1] Launching Browser...");
    var config = new BrowserConfig(Headless: false, Detached: false, ExecutablePath: null, Plugins: new[] { "stealth", "human" });
    using var browser = await Browser.Launch(config);
    
    Console.WriteLine("[TEST 2] Creating New Page and Navigating...");
    // Let's navigate to wikipedia.org as a test page
    using var page = await browser.NewPage("https://www.wikipedia.org/");

    Console.WriteLine("Waiting for page load...");
    await page.WaitForNavigation();

    // Check the page title
    string title = await page.Title();
    Console.WriteLine($"Page Title: {title}");

    Console.WriteLine("[TEST 3] Locating Elements and Standard Actions...");
    // Find the language select or search input
    var searchInput = await page.FindElement("input#searchInput");
    
    // Test Focus and TypeText
    await searchInput.Focus();
    Console.WriteLine("Typing search query...");
    await searchInput.TypeText("C# Programming Language");

    // Retrieve and verify the value of the input using an attribute check
    string? typedValue = await searchInput.Attribute("value");
    Console.WriteLine($"Typed Input Value: {typedValue}");

    Console.WriteLine("[TEST 4] Stealth Pointer Interactions...");
    // Find the submit button
    var searchButton = await page.FindElement("button[type=\"submit\"]");
    
    // Test HoverStealth and ClickStealth
    Console.WriteLine("Performing Stealth Hover...");
    await searchButton.HoverStealth();
    
    Console.WriteLine("Performing Stealth Click...");
    await searchButton.ClickStealth();

    Console.WriteLine("Waiting for navigation after submit...");
    await page.WaitForNavigation();

    // Check the updated title
    string newTitle = await page.Title();
    Console.WriteLine($"New Page Title: {newTitle}");

    Console.WriteLine("[TEST 5] Screenshot Capture...");
    byte[] screenshotBytes = await page.Screenshot();
    string screenshotPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "wikipedia_search.png");
    await File.WriteAllBytesAsync(screenshotPath, screenshotBytes);
    Console.WriteLine($"Screenshot successfully saved to: {screenshotPath} (Size: {screenshotBytes.Length} bytes)");

    Console.WriteLine("[TEST 6] Closing Browser...");
    await browser.Close();

    Console.WriteLine("\n[SUCCESS] All standard browser automation tests executed successfully.");
}
catch (Exception ex)
{
    Console.WriteLine($"[ERROR] Test run failed: {ex.Message}");
    if (ex.InnerException != null)
    {
        Console.WriteLine($"[INNER ERROR] {ex.InnerException.Message}");
    }
    Console.WriteLine(ex.StackTrace);
}

Console.WriteLine("--- Test Session Finished ---");

