param(
    [Parameter(Mandatory = $true)]
    [string]$Port,
    [string]$Board = "f411-nucleo",
    [string]$Chip,
    [int]$Baud = 115200,
    [string]$Binary,
    [string]$OutputRoot = "app_soak_runs",
    [int]$Speed = 100,
    [string]$Probe,
    [int]$DurationSec = 60,
    [int]$ResetDelayMs = 200,
    [int]$ReadSliceMs = 1800,
    [int]$PauseMs = 100,
    [string]$RunId,
    [switch]$NoFlash,
    [switch]$NoReset
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptDir "lib/board_profiles.ps1")

$boardConfig = Resolve-BoardConfig -Name $Board
if ([string]::IsNullOrWhiteSpace($Chip)) {
    $Chip = $boardConfig.chip
}
if ([string]::IsNullOrWhiteSpace($Binary)) {
    $Binary = Join-Path (Join-Path (Join-Path "target" $boardConfig.target) "release") "CortexOS"
}

$timestamp = if ($RunId) { $RunId } else { Get-Date -Format "yyyyMMdd_HHmmss" }
$outputDir = Join-Path $OutputRoot $timestamp
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$logPath = Join-Path $outputDir "session.log"
$summaryCsv = Join-Path $outputDir "summary.csv"
$summaryJson = Join-Path $outputDir "summary.json"

$logLines = New-Object System.Collections.Generic.List[string]
$commandFailures = New-Object System.Collections.Generic.List[string]
$logWriter = $null

function Add-LogLine {
    param([string]$Line)
    $script:logLines.Add($Line) | Out-Null
    if ($null -ne $script:logWriter) {
        $script:logWriter.WriteLine($Line)
        $script:logWriter.Flush()
    }
}

function Read-Lines {
    param(
        [System.IO.Ports.SerialPort]$Serial,
        [int]$WindowMs
    )

    $end = (Get-Date).AddMilliseconds($WindowMs)
    $items = New-Object System.Collections.Generic.List[string]
    while ((Get-Date) -lt $end) {
        try {
            $line = $Serial.ReadLine().TrimEnd("`r", "`n")
            $items.Add($line) | Out-Null
            Add-LogLine $line
        }
        catch [System.TimeoutException] {
        }
    }
    return $items
}

$commandPlan = @(
    @{ name = "PING"; cmd = "PING"; expect = '^PONG$' },
    @{ name = "ECHO"; cmd = "ECHO soak"; expect = '^soak$' },
    @{ name = "LED"; cmd = "LED TOGGLE"; expect = '^OK$|^ERR led_unavailable$' },
    @{ name = "PWM"; cmd = "PWM 50"; expect = '^OK$|^ERR pwm_unavailable$' },
    @{ name = "STAT"; cmd = "STAT"; expect = '^STAT ' }
)

$serial = $null
try {
    $logWriter = New-Object System.IO.StreamWriter($logPath, $false, [System.Text.UTF8Encoding]::new($false))

    if (-not $NoFlash) {
        Write-Host "flashing $Binary"
        $downloadArgs = @(
            'download',
            '--chip', $Chip,
            '--protocol', $boardConfig.probe_protocol,
            '--speed', $Speed,
            '--verify',
            $Binary
        )
        if ($Probe) {
            $downloadArgs += @('--probe', $Probe)
        }
        & probe-rs $downloadArgs
        if ($LASTEXITCODE -ne 0) {
            throw "probe-rs download failed"
        }
    }

    $serial = New-Object System.IO.Ports.SerialPort $Port, $Baud, "None", 8, "One"
    $serial.NewLine = "`n"
    $serial.ReadTimeout = 200
    $serial.Open()
    $serial.DiscardInBuffer()
    $serial.DiscardOutBuffer()

    if (-not $NoReset) {
        Write-Host "resetting target"
        $resetArgs = @(
            'reset',
            '--chip', $Chip,
            '--protocol', $boardConfig.probe_protocol,
            '--speed', $Speed
        )
        if ($Probe) {
            $resetArgs += @('--probe', $Probe)
        }
        & probe-rs $resetArgs
        if ($LASTEXITCODE -ne 0) {
            throw "probe-rs reset failed"
        }

        Start-Sleep -Milliseconds $ResetDelayMs
    }

    $bootLines = Read-Lines -Serial $serial -WindowMs 2500

    $bootSeen = @($bootLines | Where-Object { $_ -match '^boot ok \(' }).Count -gt 0
    $taskBannerSeen = @($bootLines | Where-Object { $_ -match '^app tasks created:' }).Count -gt 0

    $commandsSent = 0
    $commandsPassed = 0
    $commandsFailed = 0
    $deadline = (Get-Date).AddSeconds($DurationSec)
    $planIndex = 0

    while ((Get-Date) -lt $deadline) {
        $entry = $commandPlan[$planIndex % $commandPlan.Count]
        $planIndex++
        $commandsSent++
        $serial.Write($entry.cmd + "`r`n")
        $lines = Read-Lines -Serial $serial -WindowMs $ReadSliceMs
        if (@($lines | Where-Object { $_ -match $entry.expect }).Count -gt 0) {
            $commandsPassed++
        }
        else {
            $commandsFailed++
            $commandFailures.Add($entry.name) | Out-Null
        }
        Start-Sleep -Milliseconds $PauseMs
    }

    $null = Read-Lines -Serial $serial -WindowMs 1500
}
finally {
    if ($null -ne $serial) {
        if ($serial.IsOpen) {
            $serial.Close()
        }
        $serial.Dispose()
    }
    if ($null -ne $logWriter) {
        $logWriter.Dispose()
    }
}

$faultLines = @($logLines | Where-Object { $_ -match '^fault:' })
$errorLines = @($logLines | Where-Object { $_ -match '^ERR' })
$healthLines = @($logLines | Where-Object { $_ -match '^(health:|STAT )' })

$maxFeeds = 0
$maxStale = 0
$maxRxOv = 0
$maxTxOv = 0
$maxCmdDrop = 0
foreach ($line in $healthLines) {
    if ($line -match 'feeds=(\d+)') { $maxFeeds = [Math]::Max($maxFeeds, [int]$Matches[1]) }
    if ($line -match 'stale=(\d+)') { $maxStale = [Math]::Max($maxStale, [int]$Matches[1]) }
    if ($line -match 'rxov=(\d+)') { $maxRxOv = [Math]::Max($maxRxOv, [int]$Matches[1]) }
    if ($line -match 'txov=(\d+)') { $maxTxOv = [Math]::Max($maxTxOv, [int]$Matches[1]) }
    if ($line -match 'cmd_drop=(\d+)') { $maxCmdDrop = [Math]::Max($maxCmdDrop, [int]$Matches[1]) }
}

$summary = [pscustomobject]@{
    timestamp = $timestamp
    board = $boardConfig.name
    chip = $Chip
    probe = if ($Probe) { $Probe } else { '' }
    port = $Port
    duration_sec = $DurationSec
    flashed = (-not $NoFlash)
    reset = (-not $NoReset)
    boot_seen = $bootSeen
    task_banner_seen = $taskBannerSeen
    commands_sent = $commandsSent
    commands_passed = $commandsPassed
    commands_failed = $commandsFailed
    health_lines = $healthLines.Count
    max_feeds = $maxFeeds
    max_stale = $maxStale
    max_rx_overflow = $maxRxOv
    max_tx_overflow = $maxTxOv
    max_cmd_drop = $maxCmdDrop
    fault_lines = $faultLines.Count
    error_lines = $errorLines.Count
    command_failures = if ($commandFailures.Count -gt 0) { ($commandFailures -join ';') } else { '' }
    log = $logPath
}

$summary | Export-Csv -Path $summaryCsv -NoTypeInformation -Encoding UTF8
$summary | ConvertTo-Json | Set-Content -Path $summaryJson -Encoding UTF8

Write-Host "log:      $logPath"
Write-Host "summary:  $summaryCsv"
Write-Host "summaryj: $summaryJson"
$summary
