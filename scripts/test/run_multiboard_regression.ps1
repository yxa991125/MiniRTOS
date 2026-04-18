param(
    [switch]$IncludeBench,
    [switch]$IncludeF103Debug,
    [switch]$SkipSmoke,
    [string]$F103Port,
    [string]$F411Port,
    [string]$F103Probe,
    [string]$F411Probe,
    [string[]]$SmokeBoardPorts = @(),
    [string[]]$SmokeBoardProbes = @(),
    [string[]]$BuildMatrix = @(
        'f411-nucleo:debug:app:required',
        'f411-nucleo:release:app:required',
        'f103c8-bluepill:release:app:required'
    ),
    [string]$FlashOnSmoke = 'true',
    [switch]$AutoDisableFlashWhenProbeMissing,
    [int]$SmokeReadTimeoutMs = 0,
    [int]$SmokeStartupWindowMs = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
. (Join-Path (Join-Path (Split-Path -Parent $scriptDir) 'lib') 'board_profiles.ps1')

$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$outDir = Join-Path (Join-Path $repoRoot 'runs/regression') $timestamp
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$summaryCsv = Join-Path $outDir 'summary.csv'
$summaryJson = Join-Path $outDir 'summary.json'
$metaJson = Join-Path $outDir 'meta.json'

$results = New-Object System.Collections.Generic.List[object]
$stepIndex = 0
$requiredFailed = $false

function Parse-BoolArg {
    param(
        [string]$Value,
        [bool]$Default = $true
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return $Default
    }

    switch ($Value.Trim().ToLowerInvariant()) {
        '1' { return $true }
        'true' { return $true }
        '$true' { return $true }
        'yes' { return $true }
        'on' { return $true }
        '0' { return $false }
        'false' { return $false }
        '$false' { return $false }
        'no' { return $false }
        'off' { return $false }
        default {
            throw "invalid -FlashOnSmoke value '$Value'. use true/false or 1/0."
        }
    }
}

function Probe-Available {
    if (-not (Get-Command 'probe-rs' -ErrorAction SilentlyContinue)) {
        return $false
    }

    $listOutput = & probe-rs list 2>&1
    if ($LASTEXITCODE -ne 0) {
        return $false
    }

    $joined = ($listOutput | Out-String).Trim()
    if ([string]::IsNullOrWhiteSpace($joined)) {
        return $false
    }

    if ($joined -match 'No debug probes were found') {
        return $false
    }

    return $true
}

function Add-StepResult {
    param(
        [string]$Step,
        [string]$Status,
        [bool]$Required,
        [int]$ExitCode,
        [double]$Seconds,
        [string]$Command,
        [string]$StdoutPath,
        [string]$StderrPath,
        [string]$Note
    )

    $script:results.Add([pscustomobject]@{
        step = $Step
        status = $Status
        required = $Required
        exit_code = $ExitCode
        duration_s = [math]::Round($Seconds, 2)
        command = $Command
        stdout = $StdoutPath
        stderr = $StderrPath
        note = $Note
    })

    if ($Required -and $Status -ne 'PASS') {
        $script:requiredFailed = $true
    }
}

function Invoke-ExternalStep {
    param(
        [string]$Name,
        [bool]$Required,
        [string]$FilePath,
        [string[]]$Arguments
    )

    $script:stepIndex += 1
    $tag = "{0:D2}_{1}" -f $script:stepIndex, $Name
    $stdoutPath = Join-Path $script:outDir ($tag + '.stdout.log')
    $stderrPath = Join-Path $script:outDir ($tag + '.stderr.log')
    $command = $FilePath + ' ' + ($Arguments -join ' ')

    Write-Host ("[{0}] {1}" -f $tag, $command)
    $started = Get-Date
    $exitCode = -1
    $note = ''
    $status = 'FAIL'

    try {
        $proc = Start-Process -FilePath $FilePath -ArgumentList $Arguments -WorkingDirectory $repoRoot -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
        $exitCode = $proc.ExitCode
        if ($exitCode -eq 0) {
            $status = 'PASS'
        } else {
            $note = "exit_code=$exitCode"
        }
    } catch {
        $note = $_.Exception.Message
    }

    $seconds = ((Get-Date) - $started).TotalSeconds
    Add-StepResult -Step $Name -Status $status -Required $Required -ExitCode $exitCode -Seconds $seconds -Command $command -StdoutPath $stdoutPath -StderrPath $stderrPath -Note $note
}

function Add-SkippedStep {
    param(
        [string]$Name,
        [string]$Reason
    )

    $script:stepIndex += 1
    $tag = "{0:D2}_{1}" -f $script:stepIndex, $Name
    Write-Host ("[{0}] SKIP ({1})" -f $tag, $Reason)
    Add-StepResult -Step $Name -Status 'SKIP' -Required $false -ExitCode 0 -Seconds 0 -Command '' -StdoutPath '' -StderrPath '' -Note $Reason
}

function Parse-BuildEntry {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Entry
    )

    if ($Entry -notmatch '^\s*([^:]+)\s*:\s*([^:]+)\s*:\s*([^:]+)\s*(?::\s*([^:]+)\s*)?$') {
        throw "invalid build matrix entry '$Entry'. expected 'board:profile:mode[:required|optional]'"
    }

    $boardName = $matches[1].Trim()
    $profile = $matches[2].Trim().ToLowerInvariant()
    $mode = $matches[3].Trim().ToLowerInvariant()
    $requiredToken = $matches[4]
    $required = $true
    if (-not [string]::IsNullOrWhiteSpace($requiredToken)) {
        switch ($requiredToken.Trim().ToLowerInvariant()) {
            'required' { $required = $true }
            'optional' { $required = $false }
            default { throw "invalid build matrix requirement '$requiredToken' in entry '$Entry'" }
        }
    }

    if ($profile -notin @('debug', 'release')) {
        throw "invalid profile '$profile' in build matrix entry '$Entry'"
    }
    if ($mode -notin @('app', 'bench', 'uart-probe')) {
        throw "invalid mode '$mode' in build matrix entry '$Entry'"
    }

    $board = Resolve-BoardConfig -Name $boardName
    return [pscustomobject]@{
        Board = $board.name
        Profile = $profile
        Mode = $mode
        Required = $required
    }
}

function Parse-BoardKVList {
    param(
        [string[]]$Entries,
        [string]$Label
    )

    $map = @{}
    foreach ($entry in @($Entries)) {
        foreach ($part in @($entry -split '[,;]')) {
            if ([string]::IsNullOrWhiteSpace($part)) {
                continue
            }

            if ($part -notmatch '^\s*([^:=]+)\s*[:=]\s*(.+)\s*$') {
                throw "invalid $Label entry '$part'. expected 'board:VALUE' or 'board=VALUE'"
            }

            $board = Resolve-BoardConfig -Name $matches[1].Trim()
            $value = $matches[2].Trim()
            if ([string]::IsNullOrWhiteSpace($value)) {
                throw "invalid $Label entry '$part': value is empty"
            }

            if ($map.ContainsKey($board.name)) {
                throw "duplicate board '$($board.name)' in $Label entries"
            }

            $map[$board.name] = $value
        }
    }

    return $map
}

function Build-StepName {
    param(
        [string]$Prefix,
        [string]$Board,
        [string]$Profile,
        [string]$Mode
    )

    $boardToken = ($Board -replace '[^a-zA-Z0-9]+', '_').Trim('_').ToLowerInvariant()
    return "{0}_{1}_{2}_{3}" -f $Prefix, $boardToken, $Profile, $Mode
}

$flashOnSmokeEnabled = Parse-BoolArg -Value $FlashOnSmoke -Default $true

$meta = [ordered]@{
    timestamp = $timestamp
    include_bench = [bool]$IncludeBench
    include_f103_debug = [bool]$IncludeF103Debug
    skip_smoke = [bool]$SkipSmoke
    f103_port = $F103Port
    f411_port = $F411Port
    f103_probe = $F103Probe
    f411_probe = $F411Probe
    smoke_board_ports = @($SmokeBoardPorts)
    smoke_board_probes = @($SmokeBoardProbes)
    build_matrix = @($BuildMatrix)
    flash_on_smoke = $flashOnSmokeEnabled
    auto_disable_flash_when_probe_missing = [bool]$AutoDisableFlashWhenProbeMissing
    smoke_read_timeout_ms = $SmokeReadTimeoutMs
    smoke_startup_window_ms = $SmokeStartupWindowMs
    git_sha = (& git -C $repoRoot rev-parse HEAD).Trim()
    output_dir = $outDir
}
$meta | ConvertTo-Json -Depth 6 | Set-Content -Path $metaJson -Encoding UTF8

if ($flashOnSmokeEnabled -and $AutoDisableFlashWhenProbeMissing) {
    $probeDetected = Probe-Available
    if ($probeDetected) {
        Add-StepResult -Step 'probe_precheck_for_flash' -Status 'PASS' -Required $false -ExitCode 0 -Seconds 0 -Command 'probe-rs list' -StdoutPath '' -StderrPath '' -Note 'probe detected'
    } else {
        Add-SkippedStep -Name 'probe_precheck_for_flash' -Reason 'no probe detected; force FlashOnSmoke=false for this run'
        $flashOnSmokeEnabled = $false
    }
}

$buildEntries = New-Object System.Collections.Generic.List[object]
foreach ($entry in $BuildMatrix) {
    $buildEntries.Add((Parse-BuildEntry -Entry $entry))
}

if ($IncludeF103Debug) {
    $hasF103Debug = $false
    foreach ($entry in $buildEntries) {
        if ($entry.Board -eq 'f103c8-bluepill' -and $entry.Profile -eq 'debug' -and $entry.Mode -eq 'app') {
            $hasF103Debug = $true
            break
        }
    }
    if (-not $hasF103Debug) {
        $buildEntries.Add([pscustomobject]@{
                Board = 'f103c8-bluepill'
                Profile = 'debug'
                Mode = 'app'
                Required = $false
            })
    }
} else {
    Add-SkippedStep -Name 'build_f103_debug_app' -Reason 'disabled by default (64K conservative FLASH often overflows in debug profile)'
}

if ($IncludeBench) {
    $hasBench = $false
    foreach ($entry in $buildEntries) {
        if ($entry.Board -eq 'f411-nucleo' -and $entry.Profile -eq 'release' -and $entry.Mode -eq 'bench') {
            $hasBench = $true
            break
        }
    }
    if (-not $hasBench) {
        $buildEntries.Add([pscustomobject]@{
                Board = 'f411-nucleo'
                Profile = 'release'
                Mode = 'bench'
                Required = $true
            })
    }
}

$psh = 'powershell'
$prefix = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File')

foreach ($entry in $buildEntries) {
    $stepName = Build-StepName -Prefix 'build' -Board $entry.Board -Profile $entry.Profile -Mode $entry.Mode
    Invoke-ExternalStep -Name $stepName -Required $entry.Required -FilePath $psh -Arguments ($prefix + @(
            (Join-Path $repoRoot 'scripts/build/build_board.ps1'),
            '-Board', $entry.Board,
            '-Profile', $entry.Profile,
            '-Mode', $entry.Mode
        ))
}

$smokeTargets = New-Object System.Collections.Generic.List[object]
if (@($SmokeBoardPorts).Count -gt 0) {
    $portMap = Parse-BoardKVList -Entries $SmokeBoardPorts -Label 'SmokeBoardPorts'
    $probeMap = Parse-BoardKVList -Entries $SmokeBoardProbes -Label 'SmokeBoardProbes'
    foreach ($boardName in ($portMap.Keys | Sort-Object)) {
        $probe = ''
        if ($probeMap.ContainsKey($boardName)) {
            $probe = $probeMap[$boardName]
        }
        $smokeTargets.Add([pscustomobject]@{
                Board = $boardName
                Port = $portMap[$boardName]
                Probe = $probe
            })
    }
} else {
    if ($F103Port) {
        $smokeTargets.Add([pscustomobject]@{
                Board = 'f103rct6-generic'
                Port = $F103Port
                Probe = $F103Probe
            })
    }
    if ($F411Port) {
        $smokeTargets.Add([pscustomobject]@{
                Board = 'f411-nucleo'
                Port = $F411Port
                Probe = $F411Probe
            })
    }
}

if (-not $SkipSmoke) {
    if ($smokeTargets.Count -eq 0) {
        Add-SkippedStep -Name 'smoke_targets' -Reason 'no smoke board/port provided'
    } else {
        foreach ($target in $smokeTargets) {
            $stepName = Build-StepName -Prefix 'smoke' -Board $target.Board -Profile 'release' -Mode 'app'
            $args = $prefix + @(
                (Join-Path $repoRoot 'scripts/test/run_app_smoke.ps1'),
                '-Board', $target.Board,
                '-Port', $target.Port
            )
            if ($SmokeReadTimeoutMs -gt 0) {
                $args += @('-ReadTimeoutMs', $SmokeReadTimeoutMs)
            }
            if ($SmokeStartupWindowMs -gt 0) {
                $args += @('-StartupWindowMs', $SmokeStartupWindowMs)
            }
            if ($target.Probe) {
                $args += @('-Probe', $target.Probe)
            }
            if ($flashOnSmokeEnabled) {
                $args += '-Flash'
            }
            Invoke-ExternalStep -Name $stepName -Required $true -FilePath $psh -Arguments $args
        }
    }
} else {
    Add-SkippedStep -Name 'smoke_targets' -Reason 'SkipSmoke enabled'
}

$results | Export-Csv -Path $summaryCsv -NoTypeInformation -Encoding UTF8
$results | ConvertTo-Json -Depth 5 | Set-Content -Path $summaryJson -Encoding UTF8

$passCount = @($results | Where-Object { $_.status -eq 'PASS' }).Count
$failCount = @($results | Where-Object { $_.status -eq 'FAIL' }).Count
$skipCount = @($results | Where-Object { $_.status -eq 'SKIP' }).Count
Write-Host ("summary: pass={0} fail={1} skip={2}" -f $passCount, $failCount, $skipCount)
Write-Host ("output:  {0}" -f $outDir)

if ($requiredFailed) {
    exit 1
}
