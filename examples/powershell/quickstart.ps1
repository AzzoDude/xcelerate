#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Minimal end-to-end xcelerate demo, written in PowerShell.

.DESCRIPTION
    Launches headless Chrome, opens a page, reports the compiled-in plugin catalog,
    reads the page title, the first heading, and a list, captures a full-page
    screenshot, and closes the browser.

    It uses the published `Xcelerate` module when it is installed, and otherwise
    falls back to the repo-local build.

.PARAMETER Url
    Page to open. Defaults to a self-contained data: URL so the demo runs with no
    network. Pass a real URL instead, e.g. -Url https://example.com.

.PARAMETER ExecutablePath
    Browser binary to use. Auto-discovers Chrome or Edge when omitted.

.PARAMETER Plugins
    First-party plugins to enable (default-deny), e.g. -Plugins stealth, human.

.PARAMETER Screenshot
    Where to write the full-page screenshot. Defaults to xcelerate-demo.png.

.PARAMETER NoHeadless
    Show the browser window instead of running headless.

.EXAMPLE
    pwsh ./examples/powershell/quickstart.ps1

.EXAMPLE
    pwsh ./examples/powershell/quickstart.ps1 -Url https://example.com -Plugins stealth -NoHeadless
#>
[CmdletBinding()]
param(
    [string] $Url = 'data:text/html,<title>Xcelerate Demo</title><h1>Hello from PowerShell</h1><ul><li>one</li><li>two</li><li>three</li></ul>',
    [string] $ExecutablePath,
    [string[]] $Plugins = @(),
    [string] $Screenshot = 'xcelerate-demo.png',
    [switch] $NoHeadless
)

$ErrorActionPreference = 'Stop'

# Prefer the published module; fall back to the repo-local manifest.
if (Get-Module -ListAvailable -Name Xcelerate) {
    Import-Module Xcelerate
}
else {
    $local = Join-Path (Split-Path -Parent $PSCommandPath) '../../bindings/powershell/Xcelerate.psd1'
    Import-Module (Resolve-Path $local) -Force
}

# Auto-discover a Chromium browser when one was not supplied.
if (-not $ExecutablePath) {
    $candidates = @(
        "$env:ProgramFiles\Google\Chrome\Application\chrome.exe"
        "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe"
        "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe"
        "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"
    )
    $ExecutablePath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}

$started = Get-Date
Write-Host "browser : $ExecutablePath"

$browser = Start-XcelerateBrowser -Plugins $Plugins -ExecutablePath $ExecutablePath -NoHeadless:$NoHeadless
try {
    Write-Host "catalog : $($browser.AvailablePlugins() -join ', ')"
    Write-Host "enabled : [$($browser.PluginNames() -join ', ')]"

    $page = New-XceleratePage -Browser $browser -Url $Url

    # FindElement returns immediately; WaitForSelector keeps polling until the
    # element appears (useful for content rendered after load). Not every page has
    # an <h1>, so treat it as optional.
    $heading = try { Receive-XcelerateTask $page.FindElement('h1') } catch { $null }
    $items = Receive-XcelerateTask $page.QuerySelectorAll('li')

    Write-Host "title   : $(Receive-XcelerateTask $page.Title())"
    if ($heading) {
        Write-Host "h1      : $(Receive-XcelerateTask $heading.Text())"
    }
    Write-Host "items   : $(($items | ForEach-Object { Receive-XcelerateTask $_.Text() }) -join ', ')"

    $png = Receive-XcelerateTask $page.ScreenshotFull()
    [IO.File]::WriteAllBytes((Join-Path (Get-Location) $Screenshot), $png)
    Write-Host "shot    : $Screenshot ($($png.Length) bytes)"
}
finally {
    Stop-XcelerateBrowser $browser
    Write-Host ("done    : {0} ms" -f [int]((Get-Date) - $started).TotalMilliseconds)
}
