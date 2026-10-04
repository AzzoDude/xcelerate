# Client for the human plugin (generated).

$script:Plugin = "human"

function Invoke-HumanInfo {
    param(
        [Parameter(Mandatory)] $Browser
    )
    $args = @{}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("info", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}

function Invoke-HumanMove {
    param(
        [Parameter(Mandatory)] $Browser,
        $X,
        $Y
    )
    $args = @{"x" = $X, "y" = $Y}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("move", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}

function Invoke-HumanClick {
    param(
        [Parameter(Mandatory)] $Browser,
        $X,
        $Y
    )
    $args = @{"x" = $X, "y" = $Y}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("click", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}

function Invoke-HumanType {
    param(
        [Parameter(Mandatory)] $Browser,
        $Text
    )
    $args = @{"text" = $Text}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("type", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}

function Invoke-HumanScroll {
    param(
        [Parameter(Mandatory)] $Browser,
        $DeltaY
    )
    $args = @{"deltaY" = $DeltaY}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("scroll", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}

function Invoke-HumanDelay {
    param(
        [Parameter(Mandatory)] $Browser,
        $MinMs,
        $MaxMs
    )
    $args = @{"minMs" = $MinMs, "maxMs" = $MaxMs}
    $raw = (Get-XceleratePlugin -Browser $Browser -Name $script:Plugin).Invoke("delay", ($args | ConvertTo-Json -Compress))
    if ([string]::IsNullOrWhiteSpace($raw) -or $raw -eq 'null') { return $null }
    $raw | ConvertFrom-Json
}
