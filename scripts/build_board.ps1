param(
    [Parameter(Mandatory = $true)]
    [string]$Board,
    [ValidateSet('debug', 'release')]
    [string]$Profile = 'debug',
    [ValidateSet('app', 'bench', 'uart-probe')]
    [string]$Mode = 'app'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptDir 'lib/board_profiles.ps1')

$boardConfig = Resolve-BoardConfig -Name $Board
$features = @($boardConfig.feature)
if ($Mode -eq 'bench') {
    if (-not [bool]$boardConfig.supports.bench) {
        throw "board '$($boardConfig.name)' does not support mode 'bench'"
    }
    $features += 'bench'
} elseif ($Mode -eq 'uart-probe') {
    if (-not [bool]$boardConfig.supports.uart_probe) {
        throw "board '$($boardConfig.name)' does not support mode 'uart-probe'"
    }
    $features += 'uart-probe'
}

$cargoArgs = @(
    'build',
    '--target', $boardConfig.target,
    '--no-default-features',
    '--features', ($features -join ',')
)
if ($Profile -eq 'release') {
    $cargoArgs += '--release'
}

$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$outDir = Join-Path 'board_builds' ("{0}_{1}_{2}_{3}" -f $timestamp, $boardConfig.name, $Profile, $Mode)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$logPath = Join-Path $outDir 'build.log'
$metaPath = Join-Path $outDir 'build_meta.json'
$gitSha = (& git rev-parse HEAD).Trim()
$imagePath = if ($Profile -eq 'release') {
    Join-Path (Join-Path (Join-Path 'target' $boardConfig.target) 'release') 'CortexOS'
} else {
    Join-Path (Join-Path (Join-Path 'target' $boardConfig.target) 'debug') 'CortexOS'
}

$meta = [ordered]@{
    board = $boardConfig.name
    feature = $boardConfig.feature
    target = $boardConfig.target
    chip = $boardConfig.chip
    profile = $Profile
    mode = $Mode
    git_sha = $gitSha
    command = ('cargo ' + ($cargoArgs -join ' '))
    image = $imagePath
    output_dir = $outDir
}
$meta | ConvertTo-Json -Depth 4 | Set-Content -Path $metaPath -Encoding UTF8

Write-Host ("building board={0} profile={1} mode={2}" -f $boardConfig.name, $Profile, $Mode)
$stdoutPath = Join-Path $outDir 'build.stdout.log'
$stderrPath = Join-Path $outDir 'build.stderr.log'
$proc = Start-Process -FilePath 'cargo' -ArgumentList $cargoArgs -WorkingDirectory (Get-Location) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
if (Test-Path $stdoutPath) { Get-Content $stdoutPath | Tee-Object -FilePath $logPath -Append | Out-Null }
if (Test-Path $stderrPath) { Get-Content $stderrPath | Tee-Object -FilePath $logPath -Append | Out-Null }
if ($proc.ExitCode -ne 0) {
    exit $proc.ExitCode
}

Write-Host ("image: {0}" -f $imagePath)
Write-Host ("meta:  {0}" -f $metaPath)
