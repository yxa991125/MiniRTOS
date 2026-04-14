param(
    [Parameter(Mandatory = $true)]
    [string]$Board,
    [Parameter(Mandatory = $true)]
    [string]$Port,
    [int]$BaudRate = 115200,
    [int]$ReadTimeoutMs = 3000,
    [int]$StartupWindowMs = 2000,
    [int]$ProbeSpeed = 100,
    [string]$Probe,
    [switch]$RequireBootBanner,
    [switch]$ResetBeforeCapture,
    [switch]$Flash,
    [string]$Image
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptDir 'lib/board_profiles.ps1')

function Read-LineWithDeadline {
    param(
        [System.IO.Ports.SerialPort]$Serial,
        [datetime]$Deadline
    )

    while ([datetime]::UtcNow -lt $Deadline) {
        try {
            return $Serial.ReadLine()
        } catch [System.TimeoutException] {
            continue
        }
    }

    return $null
}

function Normalize-Line {
    param([string]$Line)
    if ($null -eq $Line) {
        return $null
    }
    return $Line.TrimEnd("`r", "`n")
}

function Send-ExpectLine {
    param(
        [System.IO.Ports.SerialPort]$Serial,
        [string]$Command,
        [scriptblock]$Match,
        [datetime]$Deadline,
        [string]$Label
    )

    $Serial.Write("$Command`r`n")
    while ([datetime]::UtcNow -lt $Deadline) {
        $rawLine = Read-LineWithDeadline -Serial $Serial -Deadline $Deadline
        if ($null -eq $rawLine) {
            break
        }
        $line = Normalize-Line -Line $rawLine
        Add-Content -Path $script:SessionLog -Value $line
        if (& $Match $line) {
            return [ordered]@{ ok = $true; line = $line; label = $Label }
        }
    }

    return [ordered]@{ ok = $false; line = ''; label = $Label }
}

$boardConfig = Resolve-BoardConfig -Name $Board
$defaultImage = Join-Path (Join-Path (Join-Path 'target' $boardConfig.target) 'release') 'CortexOS'
if ($Flash) {
    $imageToFlash = if ($Image) { $Image } else { $defaultImage }
    # Keep flash and reset decoupled: open serial first, then reset, so startup banners are observable.
    $flashArgs = @(
        '-NoProfile',
        '-ExecutionPolicy', 'Bypass',
        '-File', 'scripts/flash_board.ps1',
        '-Board', $boardConfig.name,
        '-Image', $imageToFlash
    )
    if ($Probe) {
        $flashArgs += @('-Probe', $Probe)
    }
    & powershell $flashArgs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$outDir = Join-Path 'app_smoke_runs' ("{0}_{1}" -f $timestamp, $boardConfig.name)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$script:SessionLog = Join-Path $outDir 'session.log'
$summaryJson = Join-Path $outDir 'summary.json'
$summaryCsv = Join-Path $outDir 'summary.csv'

$gitSha = (& git rev-parse HEAD).Trim()
$meta = [ordered]@{
    board = $boardConfig.name
    port = $Port
    baud_rate = $BaudRate
    read_timeout_ms = $ReadTimeoutMs
    startup_window_ms = $StartupWindowMs
    probe_speed = $ProbeSpeed
    probe = $Probe
    require_boot_banner = [bool]$RequireBootBanner
    reset_before_capture = [bool]($ResetBeforeCapture -or $Flash)
    flash = [bool]$Flash
    image = if ($Image) { $Image } else { $defaultImage }
    git_sha = $gitSha
}
$meta | ConvertTo-Json -Depth 4 | Set-Content -Path (Join-Path $outDir 'smoke_meta.json') -Encoding UTF8

try {
    [void][System.IO.Ports.SerialPort]
} catch {
    Add-Type -AssemblyName System.IO.Ports
}
$serial = [System.IO.Ports.SerialPort]::new($Port, $BaudRate, [System.IO.Ports.Parity]::None, 8, [System.IO.Ports.StopBits]::One)
$serial.NewLine = "`n"
$serial.ReadTimeout = 200
$serial.WriteTimeout = $ReadTimeoutMs
$serial.DtrEnable = $false
$serial.RtsEnable = $false
$serial.Open()

$bootSeen = $false
$taskBannerSeen = $false
$results = New-Object System.Collections.Generic.List[object]

try {
    if ($ResetBeforeCapture -or $Flash) {
        $resetArgs = @(
            'reset',
            '--chip', $boardConfig.chip,
            '--protocol', $boardConfig.probe_protocol,
            '--speed', $ProbeSpeed
        )
        if ($Probe) {
            $resetArgs += @('--probe', $Probe)
        }
        Add-Content -Path $script:SessionLog -Value ("[reset] probe-rs " + ($resetArgs -join ' '))
        & probe-rs $resetArgs
        if ($LASTEXITCODE -ne 0) {
            throw 'probe reset failed before serial capture'
        }
        Start-Sleep -Milliseconds 120
    }

    $startupDeadline = [datetime]::UtcNow.AddMilliseconds($StartupWindowMs)
    while ([datetime]::UtcNow -lt $startupDeadline) {
        $rawLine = Read-LineWithDeadline -Serial $serial -Deadline $startupDeadline
        if ($null -eq $rawLine) {
            break
        }
        $line = Normalize-Line -Line $rawLine
        Add-Content -Path $script:SessionLog -Value $line
        if ($line -like '*boot ok*') {
            $bootSeen = $true
        }
        if ($line -like '*app tasks created*') {
            $taskBannerSeen = $true
        }
    }

    if ($RequireBootBanner -and -not $bootSeen) {
        throw 'required boot banner not observed'
    }

    $commandDeadline = [datetime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
    $results.Add((Send-ExpectLine -Serial $serial -Command 'PING' -Match { param($line) $line -eq 'PONG' } -Deadline $commandDeadline -Label 'PING'))
    $commandDeadline = [datetime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
    $results.Add((Send-ExpectLine -Serial $serial -Command 'ECHO smoke' -Match { param($line) $line -eq 'smoke' } -Deadline $commandDeadline -Label 'ECHO'))
    $commandDeadline = [datetime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
    $results.Add((Send-ExpectLine -Serial $serial -Command 'STAT' -Match { param($line) $line -like 'STAT *' } -Deadline $commandDeadline -Label 'STAT'))
    $commandDeadline = [datetime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
    $results.Add((Send-ExpectLine -Serial $serial -Command 'LED TOGGLE' -Match { param($line) $line -eq 'OK' -or $line -eq 'ERR led_unavailable' } -Deadline $commandDeadline -Label 'LED'))
    $commandDeadline = [datetime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
    $results.Add((Send-ExpectLine -Serial $serial -Command 'PWM 50' -Match { param($line) $line -eq 'OK' -or $line -eq 'ERR pwm_unavailable' } -Deadline $commandDeadline -Label 'PWM'))
}
finally {
    if ($serial.IsOpen) {
        $serial.Close()
    }
    $serial.Dispose()
}

$passed = @($results | Where-Object { $_.ok }).Count
$summary = [ordered]@{
    board = $boardConfig.name
    boot_seen = $bootSeen
    task_banner_seen = $taskBannerSeen
    commands_sent = $results.Count
    commands_passed = $passed
    commands_failed = $results.Count - $passed
}

$summary | ConvertTo-Json -Depth 4 | Set-Content -Path $summaryJson -Encoding UTF8
$summary.GetEnumerator() | ForEach-Object {
    [pscustomobject]@{ key = $_.Key; value = $_.Value }
} | Export-Csv -Path $summaryCsv -NoTypeInformation -Encoding UTF8

if ($summary.commands_failed -ne 0) {
    Write-Error "app smoke failed; see $outDir"
    exit 1
}

Write-Host ("smoke ok: {0}" -f $outDir)
