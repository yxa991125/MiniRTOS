param(
    [Parameter(Mandatory = $true)]
    [string]$Port,
    [int]$BaudRate = 115200,
    [int]$ReadWindowMs = 800
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Read-Available {
    param(
        [System.IO.Ports.SerialPort]$Serial,
        [int]$WindowMs
    )

    $deadline = [datetime]::UtcNow.AddMilliseconds($WindowMs)
    $buffer = ''

    while ([datetime]::UtcNow -lt $deadline) {
        if ($Serial.BytesToRead -gt 0) {
            $chunk = $Serial.ReadExisting()
            if ($chunk) {
                $buffer += $chunk
            }
            Start-Sleep -Milliseconds 10
            continue
        }

        Start-Sleep -Milliseconds 10
    }

    if ([string]::IsNullOrEmpty($buffer)) {
        return
    }

    $buffer = $buffer -replace "`r", ''
    $lines = $buffer -split "`n"
    foreach ($line in $lines) {
        if (-not [string]::IsNullOrWhiteSpace($line)) {
            Write-Host ("rx< {0}" -f $line)
        }
    }
}

try {
    [void][System.IO.Ports.SerialPort]
} catch {
    Add-Type -AssemblyName System.IO.Ports
}

$serial = [System.IO.Ports.SerialPort]::new(
    $Port,
    $BaudRate,
    [System.IO.Ports.Parity]::None,
    8,
    [System.IO.Ports.StopBits]::One
)
$serial.ReadTimeout = 100
$serial.WriteTimeout = 1000
$serial.DtrEnable = $false
$serial.RtsEnable = $false

try {
    $serial.Open()
    Write-Host ("opened {0} @ {1} 8N1" -f $Port, $BaudRate)
    Write-Host "commands:"
    Write-Host "  :read    only read incoming data once"
    Write-Host "  :quit    exit"
    Write-Host "  other text will be sent with CRLF"

    Read-Available -Serial $serial -WindowMs 1200

    while ($true) {
        $line = Read-Host 'tx>'
        if ($line -eq ':quit') {
            break
        }
        if ($line -eq ':read') {
            Read-Available -Serial $serial -WindowMs $ReadWindowMs
            continue
        }

        $serial.Write($line + "`r`n")
        Read-Available -Serial $serial -WindowMs $ReadWindowMs
    }
}
finally {
    if ($serial.IsOpen) {
        $serial.Close()
    }
    $serial.Dispose()
}
