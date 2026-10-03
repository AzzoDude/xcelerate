# Xcelerate PowerShell Module

PowerShell bindings for the xcelerate CDP browser-automation engine.

PowerShell has no UniFFI generator, so this module wraps the generated **.NET
SDK** (`uniffi.xcelerate`). The module code is hand-written; the managed
assembly and the native `xcelerate` cdylib are staged next to each other by
`scripts/generate_powershell_bindings.py`.

## Requirements

- **PowerShell 7.0+** (`pwsh`). Windows PowerShell 5.1 cannot load the .NET 8+
  assembly.
- The staged payload (`lib/`), produced by the generator. This is not committed.

## Building

```bash
python scripts/generate_powershell_bindings.py
```

This builds the Rust core and the .NET SDK if needed, then copies
`Xcelerate.Net.dll` plus the native library into `bindings/powershell/lib/<tfm>/`
for `net8.0`, `net9.0`, and `net10.0`. The module picks the framework that
matches the runtime hosting PowerShell.

## Quick start

```powershell
Import-Module ./bindings/powershell/Xcelerate.psd1

$browser = Start-XcelerateBrowser -Plugins stealth, human -NoHeadless
$page    = New-XceleratePage -Browser $browser -Url 'https://example.com'

Write-Host "Title: $(Receive-XcelerateTask $page.Title())"

$heading = Receive-XcelerateTask $page.WaitForSelector('h1')
Write-Host "Heading: $(Receive-XcelerateTask $heading.Text())"

$png = Receive-XcelerateTask $page.ScreenshotFull()
[IO.File]::WriteAllBytes("$PWD/capture.png", $png)

Stop-XcelerateBrowser $browser
```

## Functions

| Function | Purpose |
| --- | --- |
| `Import-XcelerateAssembly` | Load the .NET SDK (called automatically by the others) |
| `Receive-XcelerateTask` | Block on any `Task[T]` returned by the raw .NET objects |
| `New-XcelerateConfig` | Build a `BrowserConfig` (defaults: headless, detached) |
| `Start-XcelerateBrowser` | Launch a browser and return the `Browser` object |
| `Stop-XcelerateBrowser` | Close the browser |
| `New-XceleratePage` | Open a page and return the `Page` object |
| `Get-XceleratePlugin` | A handle to an enabled plugin |

Every engine method is `async`; the wrapper functions resolve tasks for you, and
`Receive-XcelerateTask` does the same for the raw .NET API.

## Plugins

Plugins are opt-in (default-deny). Enable them at launch or at runtime:

```powershell
$browser = Start-XcelerateBrowser -Plugins stealth

$browser.PluginNames()            # enabled on this browser
$browser.AvailablePlugins()       # compiled-in catalog

$stealth = Get-XceleratePlugin -Browser $browser -Name stealth
$stealth.Ops()                    # operations it exposes
$stealth.Invoke('info', '{}')     # JSON in, JSON out
```

Third-party plugins are not supported yet: `LoadPlugin` refuses rather than
executing unknown code.

## Raw .NET access

Because the module is a thin wrapper, any method on the generated types is
available directly:

```powershell
$config = [uniffi.xcelerate.BrowserConfig]::new($false, $true, $null, @('stealth'))
$browser = [uniffi.xcelerate.Browser]::Launch($config).GetAwaiter().GetResult()
```

## License

MIT OR Apache-2.0
