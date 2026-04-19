# Xcelerate

[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, lightweight **Chrome DevTools Protocol (CDP)** client for .NET. Built for speed and developer experience, `Xcelerate` provides a clean, managed API for browser automation with a minimalist, "Zero-Config" core.

## 🚀 Features

- **Safe & Managed**: 100% safe C# API (no pointers or `unsafe` blocks required).
- **NativeAOT Compatible**: Optimized for startup speed and low memory footprint.
- **Zero-Config**: Automatic discovery and launching of Chrome/Edge on Windows.
- **Fluent API**: Intuitive methods for automation (Navigate, Type, Click, Screenshots).
- **Zero Dependencies**: Bundles everything you need in a single package.

## 📦 Installation

```powershell
dotnet add package Xcelerate
```

## 🛠 Usage Examples

`Xcelerate` supports both synchronous and asynchronous programming models.

### ⚡ Synchronous (Simple Scripts)
```csharp
using Xcelerate;

// 1. Launch & Automatic Cleanup
using var browser = Browser.Launch(headless: false);

// 2. High-level Managed API
using var page = browser.NewPage("https://www.google.com");
Console.WriteLine($"Title: {page.GetTitle()}");

// 3. Navigation & Interaction
page.Navigate("https://github.com");
using var search = page.WaitForSelector("input[name='q']");
search.TypeText("Xcelerate");
```

### 🌐 Asynchronous (Modern Applications)
```csharp
using Xcelerate;

// 1. Launch Async
using var browser = await Browser.LaunchAsync(headless: true);

// 2. Multi-step Async Logic
using var page = await browser.NewPageAsync("https://www.google.com");
await page.NavigateAsync("https://github.com");

var title = await page.GetTitleAsync();
Console.WriteLine($"Title: {title}");

// 3. Element Interactions
using var element = await page.WaitForSelectorAsync("input[name='q']");
await element.TypeTextAsync("Xcelerate SDK");
await element.ClickAsync();
```

## 🏗 Why Xcelerate?

Unlike other CDP wrappers that can be heavy or complex to set up, `Xcelerate` focuses on the "First 5 Minutes" experience. It handles the messy process of launching, port polling, and PID management so you can focus on your automation logic.

## ⚖ License
Distributed under the MIT License. See `LICENSE` for more information.

---
*Developed by AzzoDude*
