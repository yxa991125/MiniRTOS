param(
    [Parameter(Mandatory = $true)]
    [string]$Port,
    [string]$Board = "f411-nucleo",
    [string]$Chip,
    [int]$Baud = 0,
    [string]$Binary,
    [string]$OutputRoot = "",
    [int]$Speed = 100,
    [string]$Probe,
    [int]$DurationSec = 86400,
    [int]$ResetDelayMs = 200,
    [int]$ReadSliceMs = 0,
    [int]$PauseMs = 0,
    [switch]$NoFlash,
    [switch]$NoReset
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repoRoot "runs/soak"
}

$runId = Get-Date -Format "yyyyMMdd_HHmmss"
$outputDir = Join-Path $OutputRoot $runId
$jobMeta = Join-Path $outputDir "job.json"
$stdoutLog = Join-Path $outputDir "launcher.stdout.log"
$stderrLog = Join-Path $outputDir "launcher.stderr.log"
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$scriptPath = (Resolve-Path (Join-Path $repoRoot "scripts/test/soak_default_app.ps1")).Path
$argList = @(
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", $scriptPath,
    "-Board", $Board,
    "-Port", $Port,
    "-OutputRoot", $OutputRoot,
    "-Speed", $Speed,
    "-DurationSec", $DurationSec,
    "-ResetDelayMs", $ResetDelayMs,
    "-RunId", $runId
)
if ($ReadSliceMs -gt 0) {
    $argList += @("-ReadSliceMs", $ReadSliceMs)
}
if ($PauseMs -gt 0) {
    $argList += @("-PauseMs", $PauseMs)
}

if ($Baud -gt 0) {
    $argList += @("-Baud", $Baud)
}

if ($Chip) {
    $argList += @("-Chip", $Chip)
}
if ($Binary) {
    $argList += @("-Binary", $Binary)
}
if ($Probe) {
    $argList += @("-Probe", $Probe)
}
if ($NoFlash) {
    $argList += "-NoFlash"
}
if ($NoReset) {
    $argList += "-NoReset"
}

$process = Start-Process `
    -FilePath "powershell" `
    -ArgumentList $argList `
    -WorkingDirectory $repoRoot `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -WindowStyle Hidden `
    -PassThru

$meta = [pscustomobject]@{
    pid = $process.Id
    run_id = $runId
    board = $Board
    chip = if ($Chip) { $Chip } else { "" }
    probe = if ($Probe) { $Probe } else { "" }
    output_dir = $outputDir
    started_at = (Get-Date).ToString("s")
    port = $Port
    duration_sec = $DurationSec
    flashed = (-not $NoFlash)
    reset = (-not $NoReset)
    stdout_log = $stdoutLog
    stderr_log = $stderrLog
}

$meta | ConvertTo-Json | Set-Content -Path $jobMeta -Encoding UTF8

Write-Host "pid:      $($process.Id)"
Write-Host "run_id:   $runId"
Write-Host "output:   $outputDir"
Write-Host "job_meta: $jobMeta"
