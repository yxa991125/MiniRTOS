param(
    [string]$Port,
    [string]$Chip = "STM32F411RETx",
    [int]$Baud = 115200,
    [string]$Binary = "target/thumbv7em-none-eabihf/release/CortexOS",
    [string]$OutputRoot = "app_soak_runs",
    [int]$Speed = 100,
    [int]$DurationSec = 86400,
    [int]$ResetDelayMs = 200,
    [int]$ReadSliceMs = 1200,
    [int]$PauseMs = 100,
    [switch]$NoFlash
)

$ErrorActionPreference = "Stop"

if (-not $Port) {
    throw "Use -Port, for example: .\\scripts\\start_24h_soak.ps1 -Port COM6"
}

$runId = Get-Date -Format "yyyyMMdd_HHmmss"
$outputDir = Join-Path $OutputRoot $runId
$jobMeta = Join-Path $outputDir "job.json"
$stdoutLog = Join-Path $outputDir "launcher.stdout.log"
$stderrLog = Join-Path $outputDir "launcher.stderr.log"
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$scriptPath = (Resolve-Path "scripts/soak_default_app.ps1").Path
$argList = @(
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", $scriptPath,
    "-Chip", $Chip,
    "-Port", $Port,
    "-Baud", $Baud,
    "-Binary", $Binary,
    "-OutputRoot", $OutputRoot,
    "-Speed", $Speed,
    "-DurationSec", $DurationSec,
    "-ResetDelayMs", $ResetDelayMs,
    "-ReadSliceMs", $ReadSliceMs,
    "-PauseMs", $PauseMs,
    "-RunId", $runId
)

if ($NoFlash) {
    $argList += "-NoFlash"
}

$process = Start-Process `
    -FilePath "powershell" `
    -ArgumentList $argList `
    -WorkingDirectory (Get-Location).Path `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -WindowStyle Hidden `
    -PassThru

$meta = [pscustomobject]@{
    pid = $process.Id
    run_id = $runId
    output_dir = $outputDir
    started_at = (Get-Date).ToString("s")
    port = $Port
    duration_sec = $DurationSec
    flashed = (-not $NoFlash)
    stdout_log = $stdoutLog
    stderr_log = $stderrLog
}

$meta | ConvertTo-Json | Set-Content -Path $jobMeta -Encoding UTF8

Write-Host "pid:      $($process.Id)"
Write-Host "run_id:   $runId"
Write-Host "output:   $outputDir"
Write-Host "job_meta: $jobMeta"

