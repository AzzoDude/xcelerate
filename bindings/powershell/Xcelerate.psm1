#Requires -Version 7.0

<#
.SYNOPSIS
    PowerShell bindings for the xcelerate CDP browser-automation engine.

.DESCRIPTION
    PowerShell has no UniFFI generator, so this module drives the generated .NET
    SDK (`uniffi.xcelerate`) directly. The managed assembly and the native cdylib
    are staged side by side by `scripts/generate_bindings/powershell.py`.

    Every engine method is `async` and returns a .NET `Task`. The wrapper
    functions resolve tasks synchronously so they compose naturally in a script;
    `Receive-XcelerateTask` unwraps any `Task` you obtain from the raw .NET
    objects, e.g. `Receive-XcelerateTask $page.Title()`.
#>

Set-StrictMode -Version Latest

$script:XcelerateModuleRoot = $PSScriptRoot
$script:XcelerateLoaded = $false
$script:XcelerateLoadError = $null

function Import-XcelerateAssembly {
    <#
    .SYNOPSIS
        Loads the xcelerate .NET assembly and makes its native library resolvable.

    .DESCRIPTION
        Picks the managed assembly whose target framework matches the runtime
        hosting PowerShell (net8.0 / net9.0 / net10.0) and preloads the native
        cdylib next to it, so the `[LibraryImport("xcelerate")]` calls resolve.
        Idempotent; safe to call before any other function.
    #>
    [CmdletBinding()]
    param()

    if ($script:XcelerateLoaded) { return }
    if ($script:XcelerateLoadError) { throw $script:XcelerateLoadError }

    $nativeName = if ($IsWindows) { 'xcelerate.dll' }
    elseif ($IsMacOS) { 'libxcelerate.dylib' }
    else { 'libxcelerate.so' }

    $runtimeMajor = [System.Environment]::Version.Major
    for ($major = $runtimeMajor; $major -ge 8; $major--) {
        $tfm = "net$major.0"
        $dir = Join-Path $script:XcelerateModuleRoot "lib/$tfm"
        $managed = Join-Path $dir 'Xcelerate.Net.dll'
        if (-not (Test-Path -LiteralPath $managed)) { continue }

        $native = Join-Path $dir $nativeName
        if (Test-Path -LiteralPath $native) {
            # Preloading pins the exact file so later name-based P/Invoke lookups
            # reuse it; PATH covers hosts that search the environment instead.
            [System.Runtime.InteropServices.NativeLibrary]::Load($native) | Out-Null
        }
        $env:PATH = "$dir$([IO.Path]::PathSeparator)$env:PATH"

        [System.Reflection.Assembly]::LoadFrom($managed) | Out-Null
        $script:XcelerateLoaded = $true
        Write-Verbose "Loaded xcelerate .NET binding from $managed"
        return
    }

    $message = "xcelerate binding not found under '$script:XcelerateModuleRoot/lib'. " +
    "Run 'python scripts/generate_bindings/powershell.py' to stage it."
    $script:XcelerateLoadError = $message
    throw $message
}

function Receive-XcelerateTask {
    <#
    .SYNOPSIS
        Blocks on a .NET Task and returns its result.

    .DESCRIPTION
        Unwraps the async methods exposed by the raw .NET objects, e.g.
        `Receive-XcelerateTask $page.Title()`. Non-Task input is passed through.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory, ValueFromPipeline)]
        [object] $Task
    )
    process {
        if ($Task -is [System.Threading.Tasks.Task]) {
            return $Task.GetAwaiter().GetResult()
        }
        return $Task
    }
}

function New-XcelerateConfig {
    <#
    .SYNOPSIS
        Builds a BrowserConfig with xcelerate's defaults (headless, detached).

    .EXAMPLE
        New-XcelerateConfig -Plugins plugins/my-plugin -NoHeadless
    #>
    [CmdletBinding()]
    [OutputType([object])]
    param(
        [switch] $NoHeadless,
        [switch] $NoDetached,
        [string] $ExecutablePath,
        [string[]] $Plugins
    )
    Import-XcelerateAssembly
    return [uniffi.xcelerate.BrowserConfig]::new(
        [bool](-not $NoHeadless),
        [bool](-not $NoDetached),
        $ExecutablePath,
        $Plugins
    )
}

function Start-XcelerateBrowser {
    <#
    .SYNOPSIS
        Launches a browser and returns the Browser object.

    .EXAMPLE
        $browser = Start-XcelerateBrowser -Plugins plugins/my-plugin -NoHeadless
    #>
    [CmdletBinding()]
    [OutputType([object])]
    param(
        [object] $Config,
        [switch] $NoHeadless,
        [switch] $NoDetached,
        [string] $ExecutablePath,
        [string[]] $Plugins
    )
    Import-XcelerateAssembly
    if ($null -eq $Config) {
        $Config = New-XcelerateConfig -NoHeadless:$NoHeadless -NoDetached:$NoDetached `
            -ExecutablePath $ExecutablePath -Plugins $Plugins
    }
    return [uniffi.xcelerate.Browser]::Launch($Config).GetAwaiter().GetResult()
}

function Stop-XcelerateBrowser {
    <#
    .SYNOPSIS
        Closes a browser launched by Start-XcelerateBrowser.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory, ValueFromPipeline)]
        [object] $Browser
    )
    process {
        Import-XcelerateAssembly
        $null = $Browser.Close().GetAwaiter().GetResult()
    }
}

function New-XceleratePage {
    <#
    .SYNOPSIS
        Opens a new page in the browser and returns the Page object.

    .EXAMPLE
        $page = New-XceleratePage -Browser $browser -Url 'https://example.com'
    #>
    [CmdletBinding()]
    [OutputType([object])]
    param(
        [Parameter(Mandatory)]
        [object] $Browser,
        [Parameter(Mandatory, Position = 0)]
        [string] $Url
    )
    Import-XcelerateAssembly
    return $Browser.NewPage($Url).GetAwaiter().GetResult()
}

function Get-XceleratePlugin {
    <#
    .SYNOPSIS
        Returns a handle to an enabled plugin.

    .EXAMPLE
        Receive-XcelerateTask (Get-XceleratePlugin -Browser $browser -Name my-plugin).Invoke('info', '{}')
    #>
    [CmdletBinding()]
    [OutputType([object])]
    param(
        [Parameter(Mandatory)]
        [object] $Browser,
        [Parameter(Mandatory, Position = 0)]
        [string] $Name
    )
    Import-XcelerateAssembly
    return $Browser.Plugin($Name)
}

# Best-effort: make the .NET types available as soon as the module is imported,
# so callers can use `[uniffi.xcelerate.*]` directly without an explicit load.
try {
    Import-XcelerateAssembly
}
catch {
    Write-Warning $_.Exception.Message
}

Export-ModuleMember -Function @(
    'Import-XcelerateAssembly'
    'Receive-XcelerateTask'
    'New-XcelerateConfig'
    'Start-XcelerateBrowser'
    'Stop-XcelerateBrowser'
    'New-XceleratePage'
    'Get-XceleratePlugin'
)
