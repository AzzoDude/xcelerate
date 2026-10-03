# Xcelerate SDK for .NET

[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Xcelerate is a high-performance, lightweight Chrome DevTools Protocol (CDP) client tailored for the .NET ecosystem. Built on a performance-optimized Rust core, it provides an idiomatic C# wrapper that combines native speed with the safety and ease of use expected by .NET developers.

## Features

- **Managed Lifecycle**: Fully supports `IDisposable` patterns to ensure clean browser and process termination.
- **Async/Await First**: Standard `Task`-based asynchronous API for modern C# applications.
- **Security-first Plugins**: Default-deny plugin system; `stealth` (binary patching + Chrome API masking) is a first-party plugin you opt into.
- **NativeAOT Compatible**: Designed for high performance and low memory footprints.
- **Simplified Deployment**: Bundles the required native binaries for Windows (x64), removing the need for external C++ or Rust installations on the target machine.

## Installation

Install the package via the .NET CLI:

```powershell
dotnet add package Xcelerate
```

## Basic Usage

Xcelerate emphasizes a clean, readable API. The library handles port polling, browser initialization, and handshake recovery automatically.

```csharp
using Xcelerate;

// Launch a browser instance with intelligent defaults
// (Optional: headless=true, stealth=false, detached=true, executablePath=null)
// Plugins are opt-in: pass Plugins to enable the stealth plugin.
using var browser = await Browser.Launch(new BrowserConfig(Plugins: new[] { "stealth" }));

// Initialize a new page and perform navigation
using var page = await browser.NewPage("https://www.example.com");

// Capture page metadata
string title = await page.Title();
Console.WriteLine($"Current title: {title}");

// Perform interactions
using var element = await page.WaitForSelector("button.primary");
await element.Click();

// Generate high-resolution full-page screenshots
byte[] screenshot = await page.ScreenshotFull();
File.WriteAllBytes("capture.png", screenshot);
```

## Advanced Configuration

The SDK supports specialized launch options for complex automation scenarios:

- **Plugins**: Default-deny list of first-party plugins to enable. `new BrowserConfig(Plugins: new[] { "stealth", "human" })` opts into stealth and human-like input; nothing runs unless listed. `load_plugin` refuses third-party plugins until the sandboxed runner ships.
- **Detached Mode**: Allows the browser process to persist independently of the parent .NET application.
- **Headless=New**: Utilizes the modern Chromium headless engine for improved rendering and compatibility.

## Compatibility

- **Frameworks**: .NET 8.0, .NET 9.0, .NET 10.0+
- **Platform**: Windows x64 (Automatic binary deployment via NuGet)

## License

This project is distributed under the MIT License.
