Set-StrictMode -Version Latest

if (-not (Get-Variable -Scope Script -Name BoardProfilesCache -ErrorAction SilentlyContinue)) {
    $script:BoardProfilesCache = $null
}

if (-not (Get-Variable -Scope Script -Name BoardProfilesRoot -ErrorAction SilentlyContinue)) {
    $script:BoardProfilesRoot = $PSScriptRoot
}

function Get-BoardProfilesPath {
    return (Join-Path (Split-Path -Parent $script:BoardProfilesRoot) 'config/board_profiles.json')
}

function Get-BoardProfiles {
    if ($script:BoardProfilesCache -ne $null) {
        return $script:BoardProfilesCache
    }

    $path = Get-BoardProfilesPath
    if (-not (Test-Path $path)) {
        throw "board profiles file not found: $path"
    }

    $json = Get-Content -Path $path -Raw -Encoding UTF8
    $parsed = $json | ConvertFrom-Json
    if ($null -eq $parsed -or $null -eq $parsed.boards) {
        throw "invalid board profiles format in $path"
    }

    $script:BoardProfilesCache = @($parsed.boards)
    return $script:BoardProfilesCache
}

function Resolve-BoardConfig {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $normalized = $Name.Trim().ToLowerInvariant()
    $boards = Get-BoardProfiles
    foreach ($board in $boards) {
        if ($board.name.ToLowerInvariant() -eq $normalized) {
            return $board
        }

        foreach ($alias in @($board.aliases)) {
            if ([string]::IsNullOrWhiteSpace($alias)) {
                continue
            }
            if ($alias.ToLowerInvariant() -eq $normalized) {
                return $board
            }
        }
    }

    $supported = @($boards | ForEach-Object { $_.name }) -join ', '
    throw "unsupported board '$Name'. supported boards: $supported"
}

function Get-BoardNames {
    return @(Get-BoardProfiles | ForEach-Object { $_.name })
}
