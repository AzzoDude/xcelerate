using Xcelerate;

Console.WriteLine("--- Xcelerate C# Idiomatic Test App ---");

try
{
    Console.WriteLine("Launching Browser...");
    using var browser = Browser.Launch(headless: false);
    
    Console.WriteLine("Creating New Page...");
    using var page = browser.NewPage("https://www.google.com");
    
    Console.WriteLine($"Page Title: {page.GetTitle()}");
    
    Console.WriteLine("Navigating to GitHub...");
    page.Navigate("https://github.com");
    Console.WriteLine($"New Page Title: {page.GetTitle()}");
    
    Console.WriteLine("Finding Search Input...");
    using var element = page.WaitForSelector("input[name='q']");
    
    Console.WriteLine("Typing into search...");
    element.TypeText("Xcelerate Rust");
    
    Console.WriteLine("Taking Screenshot...");
    byte[] screenshot = page.Screenshot();
    File.WriteAllBytes("github_test.png", screenshot);
    Console.WriteLine($"Screenshot saved! Size: {screenshot.Length} bytes");
    
    Console.WriteLine("Test Successful! Waiting 3 seconds...");
    Thread.Sleep(3000);
}
catch (Exception ex)
{
    Console.WriteLine($"[ERROR] {ex.Message}");
}

Console.WriteLine("--- Test Finished ---");
