@{
    RootModule           = 'Xcelerate.psm1'
    ModuleVersion        = '1.0.9'
    GUID                 = 'b7e4c1a2-6f3d-4e8b-9a2c-1d5f0e7b3c94'
    Author               = 'AzzoDude'
    CompanyName          = 'Chaosware'
    Copyright            = '(c) 2026 AzzoDude. MIT OR Apache-2.0.'
    Description          = 'PowerShell bindings for the xcelerate CDP browser-automation engine (wrapper over the .NET SDK).'
    PowerShellVersion    = '7.0'
    FunctionsToExport    = @(
        'Import-XcelerateAssembly'
        'Receive-XcelerateTask'
        'New-XcelerateConfig'
        'Start-XcelerateBrowser'
        'Stop-XcelerateBrowser'
        'New-XceleratePage'
        'Get-XceleratePlugin'
    )
    CmdletsToExport      = @()
    VariablesToExport    = @()
    AliasesToExport      = @()
    PrivateData          = @{
        PSData = @{
            Tags       = @('cdp', 'browser', 'automation', 'chrome', 'rust', 'dotnet', 'powershell')
            LicenseUri = 'https://github.com/AzzoDude/xcelerate/blob/master/LICENSE-MIT'
            ProjectUri = 'https://github.com/AzzoDude/xcelerate'
        }
    }
}
