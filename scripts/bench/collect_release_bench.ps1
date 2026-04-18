param(
    [string]$Chip = "STM32F411RETx",
    [string]$Port,
    [int]$Baud = 115200,
    [int]$Runs = 10,
    [int]$Speed = 100,
    [string]$Binary = "target/thumbv7em-none-eabihf/release/CortexOS",
    [string]$OutputRoot = "",
    [int]$ReadTimeoutMs = 120000,
    [int]$ResetDelayMs = 200,
    [switch]$NoFlash
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

if (-not $Port) {
    throw "请使用 -Port 指定串口，例如: .\\scripts\\bench\\collect_release_bench.ps1 -Port COM6"
}
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repoRoot "runs/bench"
}

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$outputDir = Join-Path $OutputRoot $timestamp
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$summaryRows = New-Object System.Collections.Generic.List[object]
$validationRows = New-Object System.Collections.Generic.List[object]
$scaleRows = New-Object System.Collections.Generic.List[object]
$o1Rows = New-Object System.Collections.Generic.List[object]
$latencyAttrRows = New-Object System.Collections.Generic.List[object]
$cleanBreakdownRows = New-Object System.Collections.Generic.List[object]

function Get-NearestRankPercentile {
    param(
        [long[]]$Values,
        [int]$Percent
    )

    if (-not $Values -or $Values.Count -eq 0) {
        return $null
    }

    $sorted = $Values | Sort-Object
    $rank = [Math]::Ceiling($sorted.Count * ($Percent / 100.0))
    if ($rank -lt 1) {
        $rank = 1
    }
    if ($rank -gt $sorted.Count) {
        $rank = $sorted.Count
    }

    return [long]$sorted[$rank - 1]
}

try {
    for ($run = 1; $run -le $Runs; $run++) {
        if (-not $NoFlash) {
            Write-Host "[$run/$Runs] flashing $Binary"
            & probe-rs download --chip $Chip --protocol swd --speed $Speed --verify $Binary
            if ($LASTEXITCODE -ne 0) {
                throw "probe-rs download failed on run $run"
            }
        }
        else {
            Write-Host "[$run/$Runs] reset-only run (NoFlash)"
        }

        $deadline = [DateTime]::UtcNow.AddMilliseconds($ReadTimeoutMs)
        $builder = New-Object System.Text.StringBuilder
        $completed = $false
        $serial = $null

        try {
            $serial = New-Object System.IO.Ports.SerialPort $Port, $Baud, "None", 8, "One"
            $serial.NewLine = "`n"
            $serial.ReadTimeout = 200
            $serial.Open()
            $serial.DiscardInBuffer()
            $serial.DiscardOutBuffer()

            Write-Host "[$run/$Runs] resetting target"
            & probe-rs reset --chip $Chip --protocol swd --speed $Speed
            if ($LASTEXITCODE -ne 0) {
                throw "probe-rs reset failed on run $run"
            }

            Start-Sleep -Milliseconds $ResetDelayMs

            while ([DateTime]::UtcNow -lt $deadline) {
                try {
                    $line = $serial.ReadLine()
                    [void]$builder.AppendLine($line)
                    if ($line -match "bench complete") {
                        $completed = $true
                        break
                    }
                }
                catch [TimeoutException] {
                }
            }
        }
        finally {
            if ($null -ne $serial -and $serial.IsOpen) {
                $serial.Close()
            }
            if ($null -ne $serial) {
                $serial.Dispose()
            }
            Start-Sleep -Milliseconds 100
        }

        $text = $builder.ToString()
        $runFile = Join-Path $outputDir ("run_{0:D2}.log" -f $run)
        Set-Content -Path $runFile -Value $text -Encoding UTF8

        if ([string]::IsNullOrWhiteSpace($text)) {
            Write-Warning "run $run serial log is empty; check COM port selection and target reset behavior"
        }
        elseif (-not $completed) {
            Write-Warning "run $run did not observe 'bench complete' before timeout; consider increasing -ReadTimeoutMs"
        }

        $metricRegex = [regex]"bench:(?<name>[a-zA-Z0-9_]+)\s+count=(?<count>\d+)(?:\s+skipped=(?<skipped>\d+))?\s+min=(?<min>\d+)cy/\d+us\s+avg=(?<avg>\d+)cy/\d+us(?:\s+p50=(?<p50>\d+)cy/\d+us\s+p95=(?<p95>\d+)cy/\d+us)?\s+max=(?<max>\d+)cy/\d+us"
        foreach ($match in $metricRegex.Matches($text)) {
            $skipped = if ($match.Groups["skipped"].Success) { [int]$match.Groups["skipped"].Value } else { 0 }
            $p50 = if ($match.Groups["p50"].Success) { [long]$match.Groups["p50"].Value } else { $null }
            $p95 = if ($match.Groups["p95"].Success) { [long]$match.Groups["p95"].Value } else { $null }
            $summaryRows.Add([pscustomobject]@{
                run = $run
                metric = $match.Groups["name"].Value
                count = [long]$match.Groups["count"].Value
                skipped = $skipped
                min_cy = [long]$match.Groups["min"].Value
                avg_cy = [long]$match.Groups["avg"].Value
                p50_cy = $p50
                p95_cy = $p95
                max_cy = [long]$match.Groups["max"].Value
            }) | Out-Null
        }

        $validationRegex = [regex]"bench:(?<name>timeout_wheel_[a-zA-Z0-9_]+)\s+pass=(?<pass>\d+)\s+fail=(?<fail>\d+)(?:\s+expected=(?<expected_min>\d+)\.\.(?<expected_max>\d+)ticks\s+observed_min=(?<observed_min>\d+)\s+observed_max=(?<observed_max>\d+))?"
        foreach ($match in $validationRegex.Matches($text)) {
            $validationRows.Add([pscustomobject]@{
                run = $run
                metric = $match.Groups["name"].Value
                pass = [long]$match.Groups["pass"].Value
                fail = [long]$match.Groups["fail"].Value
                expected_min_ticks = if ($match.Groups["expected_min"].Success) { [long]$match.Groups["expected_min"].Value } else { $null }
                expected_max_ticks = if ($match.Groups["expected_max"].Success) { [long]$match.Groups["expected_max"].Value } else { $null }
                observed_min_ticks = if ($match.Groups["observed_min"].Success) { [long]$match.Groups["observed_min"].Value } else { $null }
                observed_max_ticks = if ($match.Groups["observed_max"].Success) { [long]$match.Groups["observed_max"].Value } else { $null }
            }) | Out-Null
        }

        $scaleRegex = [regex]"bench:scheduler_scale\s+tasks=(?<tasks>\d+)\s+rounds=(?<rounds>\d+)\s+round_min=(?<round_min>\d+)cy/\d+us\s+round_avg=(?<round_avg>\d+)cy/\d+us\s+round_max=(?<round_max>\d+)cy/\d+us\s+per_switch_min=(?<per_switch_min>\d+)cy/\d+us\s+per_switch_avg=(?<per_switch_avg>\d+)cy/\d+us\s+per_switch_max=(?<per_switch_max>\d+)cy/\d+us"
        foreach ($match in $scaleRegex.Matches($text)) {
            $scaleRows.Add([pscustomobject]@{
                run = $run
                tasks = [int]$match.Groups["tasks"].Value
                rounds = [int]$match.Groups["rounds"].Value
                round_min_cy = [long]$match.Groups["round_min"].Value
                round_avg_cy = [long]$match.Groups["round_avg"].Value
                round_max_cy = [long]$match.Groups["round_max"].Value
                per_switch_min_cy = [long]$match.Groups["per_switch_min"].Value
                per_switch_avg_cy = [long]$match.Groups["per_switch_avg"].Value
                per_switch_max_cy = [long]$match.Groups["per_switch_max"].Value
            }) | Out-Null
        }

        $o1Regex = [regex]"bench:scheduler_o1_check\s+steady_per_switch_avg_8_32=(?<steady_8>\d+)\/(?<steady_32>\d+)cy\s+ratio=(?<ratio>\d+)permille\s+baseline_2task=(?<baseline>\d+)cy\s+verdict=(?<verdict>[a-zA-Z0-9_]+)"
        foreach ($match in $o1Regex.Matches($text)) {
            $o1Rows.Add([pscustomobject]@{
                run = $run
                steady_8_cy = [long]$match.Groups["steady_8"].Value
                steady_32_cy = [long]$match.Groups["steady_32"].Value
                ratio_permille = [long]$match.Groups["ratio"].Value
                baseline_2task_cy = [long]$match.Groups["baseline"].Value
                verdict = $match.Groups["verdict"].Value
            }) | Out-Null
        }

        $latencyAttrRegex = [regex]"bench:(?<metric>[a-zA-Z0-9_]+_attribution)\s+threshold=(?<threshold>\d+)cy\s+overlap_samples=(?<overlap>\d+)\s+spikes=(?<spikes>\d+)\s+irq_spikes=(?<irq_spikes>\d+)\s+clean_spikes=(?<clean_spikes>\d+)\s+systick_spikes=(?<systick_spikes>\d+)\s+tim2_spikes=(?<tim2_spikes>\d+)\s+max_irq_spike=(?<max_irq>\d+)cy\s+max_clean_spike=(?<max_clean>\d+)cy"
        foreach ($match in $latencyAttrRegex.Matches($text)) {
            $latencyAttrRows.Add([pscustomobject]@{
                run = $run
                metric = $match.Groups["metric"].Value
                threshold_cy = [long]$match.Groups["threshold"].Value
                overlap_samples = [long]$match.Groups["overlap"].Value
                spikes = [long]$match.Groups["spikes"].Value
                irq_spikes = [long]$match.Groups["irq_spikes"].Value
                clean_spikes = [long]$match.Groups["clean_spikes"].Value
                systick_spikes = [long]$match.Groups["systick_spikes"].Value
                tim2_spikes = [long]$match.Groups["tim2_spikes"].Value
                max_irq_spike_cy = [long]$match.Groups["max_irq"].Value
                max_clean_spike_cy = [long]$match.Groups["max_clean"].Value
            }) | Out-Null
        }

        $twoPhaseRegex = [regex]"bench:(?<metric>[a-zA-Z0-9_]+_clean_breakdown)\s+clean_spikes=(?<clean_spikes>\d+)\s+(?<label1>[a-z0-9_]+)_dominant=(?<dominant1>\d+)\s+(?<label2>[a-z0-9_]+)_dominant=(?<dominant2>\d+)\s+max_(?<max_label1>[a-z0-9_]+)=(?<max1>\d+)cy\s+max_(?<max_label2>[a-z0-9_]+)=(?<max2>\d+)cy"
        foreach ($match in $twoPhaseRegex.Matches($text)) {
            $cleanBreakdownRows.Add([pscustomobject]@{
                run = $run
                metric = $match.Groups["metric"].Value
                phase_count = 2
                clean_spikes = [long]$match.Groups["clean_spikes"].Value
                label1 = $match.Groups["label1"].Value
                dominant1 = [long]$match.Groups["dominant1"].Value
                label2 = $match.Groups["label2"].Value
                dominant2 = [long]$match.Groups["dominant2"].Value
                label3 = $null
                dominant3 = $null
                label4 = $null
                dominant4 = $null
                max_label1 = $match.Groups["max_label1"].Value
                max1_cy = [long]$match.Groups["max1"].Value
                max_label2 = $match.Groups["max_label2"].Value
                max2_cy = [long]$match.Groups["max2"].Value
                max_label3 = $null
                max3_cy = $null
                max_label4 = $null
                max4_cy = $null
            }) | Out-Null
        }

        $fourPhaseRegex = [regex]"bench:(?<metric>[a-zA-Z0-9_]+_clean_breakdown)\s+clean_spikes=(?<clean_spikes>\d+)\s+(?<label1>[a-z0-9_]+)_dominant=(?<dominant1>\d+)\s+(?<label2>[a-z0-9_]+)_dominant=(?<dominant2>\d+)\s+(?<label3>[a-z0-9_]+)_dominant=(?<dominant3>\d+)\s+(?<label4>[a-z0-9_]+)_dominant=(?<dominant4>\d+)\s+max_(?<max_label1>[a-z0-9_]+)=(?<max1>\d+)cy\s+max_(?<max_label2>[a-z0-9_]+)=(?<max2>\d+)cy\s+max_(?<max_label3>[a-z0-9_]+)=(?<max3>\d+)cy\s+max_(?<max_label4>[a-z0-9_]+)=(?<max4>\d+)cy"
        foreach ($match in $fourPhaseRegex.Matches($text)) {
            $cleanBreakdownRows.Add([pscustomobject]@{
                run = $run
                metric = $match.Groups["metric"].Value
                phase_count = 4
                clean_spikes = [long]$match.Groups["clean_spikes"].Value
                label1 = $match.Groups["label1"].Value
                dominant1 = [long]$match.Groups["dominant1"].Value
                label2 = $match.Groups["label2"].Value
                dominant2 = [long]$match.Groups["dominant2"].Value
                label3 = $match.Groups["label3"].Value
                dominant3 = [long]$match.Groups["dominant3"].Value
                label4 = $match.Groups["label4"].Value
                dominant4 = [long]$match.Groups["dominant4"].Value
                max_label1 = $match.Groups["max_label1"].Value
                max1_cy = [long]$match.Groups["max1"].Value
                max_label2 = $match.Groups["max_label2"].Value
                max2_cy = [long]$match.Groups["max2"].Value
                max_label3 = $match.Groups["max_label3"].Value
                max3_cy = [long]$match.Groups["max3"].Value
                max_label4 = $match.Groups["max_label4"].Value
                max4_cy = [long]$match.Groups["max4"].Value
            }) | Out-Null
        }
    }
}
finally {
}

$summaryCsv = Join-Path $outputDir "summary.csv"
$summaryRows | Export-Csv -Path $summaryCsv -NoTypeInformation -Encoding UTF8

$baselineRows = foreach ($group in ($summaryRows | Group-Object metric | Sort-Object Name)) {
    $metricRuns = @($group.Group)
    $avgValues = [long[]]@($metricRuns | ForEach-Object { $_.avg_cy })
    $maxValues = [long[]]@($metricRuns | ForEach-Object { $_.max_cy })
    $p50Values = [long[]]@($metricRuns | Where-Object { $null -ne $_.p50_cy } | ForEach-Object { $_.p50_cy })
    $p95Values = [long[]]@($metricRuns | Where-Object { $null -ne $_.p95_cy } | ForEach-Object { $_.p95_cy })
    $skippedValues = [long[]]@($metricRuns | ForEach-Object { $_.skipped })

    [pscustomobject]@{
        metric = $group.Name
        runs = $metricRuns.Count
        skipped = if ($skippedValues.Count -gt 0) { ($skippedValues | Measure-Object -Maximum).Maximum } else { 0 }
        avg_min_cy = ($avgValues | Measure-Object -Minimum).Minimum
        avg_p50_cy = Get-NearestRankPercentile -Values $avgValues -Percent 50
        avg_p95_cy = Get-NearestRankPercentile -Values $avgValues -Percent 95
        avg_max_cy = ($avgValues | Measure-Object -Maximum).Maximum
        max_observed_cy = ($maxValues | Measure-Object -Maximum).Maximum
        sample_p50_cy = Get-NearestRankPercentile -Values $p50Values -Percent 50
        sample_p95_cy = Get-NearestRankPercentile -Values $p95Values -Percent 95
    }
}

$baselineCsv = Join-Path $outputDir "baseline_summary.csv"
$baselineRows | Export-Csv -Path $baselineCsv -NoTypeInformation -Encoding UTF8

$validationCsv = Join-Path $outputDir "timeout_validation.csv"
$validationRows | Export-Csv -Path $validationCsv -NoTypeInformation -Encoding UTF8

$validationSummaryRows = foreach ($group in ($validationRows | Group-Object metric | Sort-Object Name)) {
    $metricRuns = @($group.Group)
    $expectedMin = ($metricRuns | Where-Object { $null -ne $_.expected_min_ticks } | Select-Object -First 1).expected_min_ticks
    $expectedMax = ($metricRuns | Where-Object { $null -ne $_.expected_max_ticks } | Select-Object -First 1).expected_max_ticks
    $observedMinValues = [long[]]@($metricRuns | Where-Object { $null -ne $_.observed_min_ticks } | ForEach-Object { $_.observed_min_ticks })
    $observedMaxValues = [long[]]@($metricRuns | Where-Object { $null -ne $_.observed_max_ticks } | ForEach-Object { $_.observed_max_ticks })

    [pscustomobject]@{
        metric = $group.Name
        runs = $metricRuns.Count
        pass_total = ($metricRuns | Measure-Object pass -Sum).Sum
        fail_total = ($metricRuns | Measure-Object fail -Sum).Sum
        expected_min_ticks = $expectedMin
        expected_max_ticks = $expectedMax
        observed_min_ticks = if ($observedMinValues.Count -gt 0) { ($observedMinValues | Measure-Object -Minimum).Minimum } else { $null }
        observed_max_ticks = if ($observedMaxValues.Count -gt 0) { ($observedMaxValues | Measure-Object -Maximum).Maximum } else { $null }
    }
}

$validationSummaryCsv = Join-Path $outputDir "timeout_validation_summary.csv"
$validationSummaryRows | Export-Csv -Path $validationSummaryCsv -NoTypeInformation -Encoding UTF8

$scaleCsv = Join-Path $outputDir "scheduler_scale.csv"
$scaleRows | Export-Csv -Path $scaleCsv -NoTypeInformation -Encoding UTF8

$scaleSummaryRows = foreach ($group in ($scaleRows | Group-Object tasks | Sort-Object Name)) {
    $taskRuns = @($group.Group)
    $roundAvg = [long[]]@($taskRuns | ForEach-Object { $_.round_avg_cy })
    $perSwitchAvg = [long[]]@($taskRuns | ForEach-Object { $_.per_switch_avg_cy })
    $perSwitchMax = [long[]]@($taskRuns | ForEach-Object { $_.per_switch_max_cy })

    [pscustomobject]@{
        tasks = [int]$group.Name
        runs = $taskRuns.Count
        rounds = ($taskRuns | Measure-Object rounds -Maximum).Maximum
        round_avg_min_cy = ($roundAvg | Measure-Object -Minimum).Minimum
        round_avg_p50_cy = Get-NearestRankPercentile -Values $roundAvg -Percent 50
        round_avg_p95_cy = Get-NearestRankPercentile -Values $roundAvg -Percent 95
        round_avg_max_cy = ($roundAvg | Measure-Object -Maximum).Maximum
        per_switch_avg_min_cy = ($perSwitchAvg | Measure-Object -Minimum).Minimum
        per_switch_avg_p50_cy = Get-NearestRankPercentile -Values $perSwitchAvg -Percent 50
        per_switch_avg_p95_cy = Get-NearestRankPercentile -Values $perSwitchAvg -Percent 95
        per_switch_avg_max_cy = ($perSwitchAvg | Measure-Object -Maximum).Maximum
        per_switch_max_observed_cy = ($perSwitchMax | Measure-Object -Maximum).Maximum
    }
}

$scaleSummaryCsv = Join-Path $outputDir "scheduler_scale_summary.csv"
$scaleSummaryRows | Export-Csv -Path $scaleSummaryCsv -NoTypeInformation -Encoding UTF8

$o1Csv = Join-Path $outputDir "scheduler_o1.csv"
$o1Rows | Export-Csv -Path $o1Csv -NoTypeInformation -Encoding UTF8

$o1SummaryRows = if ($o1Rows.Count -gt 0) {
    $ratioValues = [long[]]@($o1Rows | ForEach-Object { $_.ratio_permille })
    $baselineValues = [long[]]@($o1Rows | ForEach-Object { $_.baseline_2task_cy })
    $steady8Values = [long[]]@($o1Rows | ForEach-Object { $_.steady_8_cy })
    $steady32Values = [long[]]@($o1Rows | ForEach-Object { $_.steady_32_cy })

    @([pscustomobject]@{
        runs = $o1Rows.Count
        verdict_likely_o1 = (@($o1Rows | Where-Object { $_.verdict -eq "likely_o1" })).Count
        verdict_not_o1 = (@($o1Rows | Where-Object { $_.verdict -eq "not_o1" })).Count
        ratio_min_permille = ($ratioValues | Measure-Object -Minimum).Minimum
        ratio_p50_permille = Get-NearestRankPercentile -Values $ratioValues -Percent 50
        ratio_p95_permille = Get-NearestRankPercentile -Values $ratioValues -Percent 95
        ratio_max_permille = ($ratioValues | Measure-Object -Maximum).Maximum
        baseline_2task_p50_cy = Get-NearestRankPercentile -Values $baselineValues -Percent 50
        steady_8_p50_cy = Get-NearestRankPercentile -Values $steady8Values -Percent 50
        steady_32_p50_cy = Get-NearestRankPercentile -Values $steady32Values -Percent 50
    })
}
else {
    @()
}

$o1SummaryCsv = Join-Path $outputDir "scheduler_o1_summary.csv"
$o1SummaryRows | Export-Csv -Path $o1SummaryCsv -NoTypeInformation -Encoding UTF8

$latencyAttrCsv = Join-Path $outputDir "latency_attribution.csv"
$latencyAttrRows | Export-Csv -Path $latencyAttrCsv -NoTypeInformation -Encoding UTF8

$latencyAttrSummaryRows = foreach ($group in ($latencyAttrRows | Group-Object metric | Sort-Object Name)) {
    $metricRuns = @($group.Group)
    [pscustomobject]@{
        metric = $group.Name
        runs = $metricRuns.Count
        threshold_cy = ($metricRuns | Measure-Object threshold_cy -Maximum).Maximum
        overlap_samples_total = ($metricRuns | Measure-Object overlap_samples -Sum).Sum
        spikes_total = ($metricRuns | Measure-Object spikes -Sum).Sum
        irq_spikes_total = ($metricRuns | Measure-Object irq_spikes -Sum).Sum
        clean_spikes_total = ($metricRuns | Measure-Object clean_spikes -Sum).Sum
        systick_spikes_total = ($metricRuns | Measure-Object systick_spikes -Sum).Sum
        tim2_spikes_total = ($metricRuns | Measure-Object tim2_spikes -Sum).Sum
        max_irq_spike_cy = ($metricRuns | Measure-Object max_irq_spike_cy -Maximum).Maximum
        max_clean_spike_cy = ($metricRuns | Measure-Object max_clean_spike_cy -Maximum).Maximum
    }
}

$latencyAttrSummaryCsv = Join-Path $outputDir "latency_attribution_summary.csv"
$latencyAttrSummaryRows | Export-Csv -Path $latencyAttrSummaryCsv -NoTypeInformation -Encoding UTF8

$cleanBreakdownCsv = Join-Path $outputDir "clean_breakdown.csv"
$cleanBreakdownRows | Export-Csv -Path $cleanBreakdownCsv -NoTypeInformation -Encoding UTF8

$cleanBreakdownSummaryRows = foreach ($group in ($cleanBreakdownRows | Group-Object metric | Sort-Object Name)) {
    $metricRuns = @($group.Group)
    [pscustomobject]@{
        metric = $group.Name
        runs = $metricRuns.Count
        clean_spikes_total = ($metricRuns | Measure-Object clean_spikes -Sum).Sum
        label1 = ($metricRuns | Select-Object -First 1).label1
        dominant1_total = ($metricRuns | Measure-Object dominant1 -Sum).Sum
        label2 = ($metricRuns | Select-Object -First 1).label2
        dominant2_total = ($metricRuns | Measure-Object dominant2 -Sum).Sum
        label3 = ($metricRuns | Select-Object -First 1).label3
        dominant3_total = ($metricRuns | Measure-Object dominant3 -Sum).Sum
        label4 = ($metricRuns | Select-Object -First 1).label4
        dominant4_total = ($metricRuns | Measure-Object dominant4 -Sum).Sum
        max1_cy = ($metricRuns | Measure-Object max1_cy -Maximum).Maximum
        max2_cy = ($metricRuns | Measure-Object max2_cy -Maximum).Maximum
        max3_cy = ($metricRuns | Measure-Object max3_cy -Maximum).Maximum
        max4_cy = ($metricRuns | Measure-Object max4_cy -Maximum).Maximum
    }
}

$cleanBreakdownSummaryCsv = Join-Path $outputDir "clean_breakdown_summary.csv"
$cleanBreakdownSummaryRows | Export-Csv -Path $cleanBreakdownSummaryCsv -NoTypeInformation -Encoding UTF8

if ($summaryRows.Count -eq 0) {
    Write-Warning "no bench metrics were parsed; check run_*.log for empty serial output or bench startup failures"
}

Write-Host "raw logs: $outputDir"
Write-Host "summary:  $summaryCsv"
Write-Host "baseline: $baselineCsv"
Write-Host "timeout:  $validationCsv"
Write-Host "scale:    $scaleCsv"
Write-Host "o1:       $o1Csv"
Write-Host "attr:     $latencyAttrCsv"
Write-Host "clean:    $cleanBreakdownCsv"
