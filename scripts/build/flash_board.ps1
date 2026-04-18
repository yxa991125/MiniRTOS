param(
    [Parameter(Mandatory = $true)]
    [string]$Board,
    [Parameter(Mandatory = $true)]
    [string]$Image,
    [int]$Speed = 100,
    [string]$Probe,
    [switch]$ResetAfter,
    [switch]$SkipVerify
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
. (Join-Path (Join-Path (Split-Path -Parent $scriptDir) 'lib') 'board_profiles.ps1')

$boardConfig = Resolve-BoardConfig -Name $Board
$imagePath = (Resolve-Path $Image).Path
$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$outDir = Join-Path (Join-Path $repoRoot 'runs/flash') ("{0}_{1}" -f $timestamp, $boardConfig.name)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$metaPath = Join-Path $outDir 'flash_meta.json'
$logPath = Join-Path $outDir 'flash.log'
$gitSha = (& git -C $repoRoot rev-parse HEAD).Trim()

$downloadArgs = @(
    'download',
    '--chip', $boardConfig.chip,
    '--protocol', $boardConfig.probe_protocol,
    '--speed', $Speed,
    $imagePath
)
if ($Probe) {
    $downloadArgs += @('--probe', $Probe)
}
if (-not $SkipVerify) {
    $downloadArgs += '--verify'
}

$meta = [ordered]@{
    board = $boardConfig.name
    chip = $boardConfig.chip
    image = $imagePath
    speed = $Speed
    probe = $Probe
    reset_after = [bool]$ResetAfter
    verify = (-not $SkipVerify)
    git_sha = $gitSha
    download_command = ('probe-rs ' + ($downloadArgs -join ' '))
    output_dir = $outDir
}
$meta | ConvertTo-Json -Depth 4 | Set-Content -Path $metaPath -Encoding UTF8

Write-Host ("flashing board={0} image={1}" -f $boardConfig.name, $imagePath)
$stdoutPath = Join-Path $outDir 'flash.stdout.log'
$stderrPath = Join-Path $outDir 'flash.stderr.log'
$proc = Start-Process -FilePath 'probe-rs' -ArgumentList $downloadArgs -WorkingDirectory $repoRoot -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
if (Test-Path $stdoutPath) { Get-Content $stdoutPath | Tee-Object -FilePath $logPath -Append | Out-Null }
if (Test-Path $stderrPath) { Get-Content $stderrPath | Tee-Object -FilePath $logPath -Append | Out-Null }
if ($proc.ExitCode -ne 0) {
    exit $proc.ExitCode
}

if ($ResetAfter) {
    $resetArgs = @(
        'reset',
        '--chip', $boardConfig.chip,
        '--protocol', $boardConfig.probe_protocol,
        '--speed', $Speed
    )
    if ($Probe) {
        $resetArgs += @('--probe', $Probe)
    }
    Add-Content -Path $logPath -Value "`n[reset] probe-rs $($resetArgs -join ' ')"
    $resetOut = Join-Path $outDir 'reset.stdout.log'
    $resetErr = Join-Path $outDir 'reset.stderr.log'
    $resetProc = Start-Process -FilePath 'probe-rs' -ArgumentList $resetArgs -WorkingDirectory $repoRoot -NoNewWindow -Wait -PassThru -RedirectStandardOutput $resetOut -RedirectStandardError $resetErr
    if (Test-Path $resetOut) { Get-Content $resetOut | Tee-Object -FilePath $logPath -Append | Out-Null }
    if (Test-Path $resetErr) { Get-Content $resetErr | Tee-Object -FilePath $logPath -Append | Out-Null }
    if ($resetProc.ExitCode -ne 0) {
        exit $resetProc.ExitCode
    }
}

Write-Host ("meta: {0}" -f $metaPath)
