param(
    [Parameter(Mandatory = $true)]
    [string]$Board,
    [Parameter(Mandatory = $true)]
    [string]$Chip,
    [Parameter(Mandatory = $true)]
    [string]$Target,
    [string]$Feature,
    [string[]]$Aliases = @(),
    [int]$FlashKB = 64,
    [int]$RamKB = 20,
    [string]$ProbeProtocol = 'swd',
    [switch]$SupportsBench,
    [switch]$SupportsUartProbe,
    [switch]$RegisterInBoardProfiles,
    [switch]$Force,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($Board -notmatch '^[a-z0-9][a-z0-9-]*$') {
    throw "invalid -Board '$Board'. expected lowercase kebab-case, e.g. 'stm32g0b1-devkit'."
}

if ([string]::IsNullOrWhiteSpace($Feature)) {
    $Feature = "board-$Board"
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

$memoryPath = Join-Path $repoRoot ("memory/{0}.x" -f $Board)
$bspModule = $Board -replace '-', '_'
$bspPath = Join-Path $repoRoot ("src/bsp/{0}.rs" -f $bspModule)
$profilesPath = Join-Path (Join-Path (Split-Path -Parent $scriptDir) 'config') 'board_profiles.json'

function Write-FileSafely {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Content
    )

    if (Test-Path $Path) {
        if (-not $Force) {
            throw "file already exists: $Path (use -Force to overwrite)"
        }
    }

    if ($DryRun) {
        Write-Host ("[dry-run] write {0}" -f $Path)
        return
    }

    $dir = Split-Path -Parent $Path
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    Set-Content -Path $Path -Value $Content -Encoding UTF8
}

$memoryContent = @"
/* Auto-generated board memory template for '$Board' */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = ${FlashKB}K
  RAM   : ORIGIN = 0x20000000, LENGTH = ${RamKB}K
}

/* Keep initial stack at RAM top unless board needs a custom layout. */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
"@

$bspContent = @"
use core::fmt::Write;

use crate::{kernel, platform::uart::UartStats};

pub struct BoardContext {
    reset_reason: kernel::ResetReason,
    sysclk_hz: u32,
}

impl BoardContext {
    pub fn take() -> Option<Self> {
        // TODO($Board): initialize clocks/peripherals and claim PAC resources.
        Some(Self {
            reset_reason: kernel::ResetReason::Unknown,
            sysclk_hz: 0,
        })
    }

    pub fn reset_reason(&self) -> kernel::ResetReason {
        self.reset_reason
    }

    pub fn sysclk_hz(&self) -> u32 {
        self.sysclk_hz
    }

    pub fn emit_boot_banner(&self) {
        let mut tx = BootWriter;
        let _ = writeln!(tx, "boot ok ($Board)");
        let _ = writeln!(tx, "reset={}", self.reset_reason.as_str());
        let _ = writeln!(tx, "cpu={}Hz", self.sysclk_hz);
    }

    #[cfg(feature = "bench")]
    pub fn init_bench(&mut self, _dcb: &mut cortex_m::peripheral::DCB, _dwt: &mut cortex_m::peripheral::DWT) {
        panic!("bench is not implemented for board '$Board'");
    }
}

struct BootWriter;

impl Write for BootWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart::boot_write_bytes(s.as_bytes());
        Ok(())
    }
}

pub mod controls {
    pub fn led_available() -> bool {
        false
    }

    pub fn set_led(_on: bool) -> bool {
        false
    }

    pub fn toggle_led() -> bool {
        false
    }

    pub fn pwm_available() -> bool {
        false
    }

    pub fn set_pwm_percent(_percent: u8) -> bool {
        false
    }
}

pub mod uart {
    use crate::platform::uart::UartStats;
    use crate::sync::event::EventError;

    pub fn init_hardware() {}

    pub fn init_app_uart() {}

    pub fn boot_write_bytes(_bytes: &[u8]) {}

    pub fn app_is_ready() -> bool {
        false
    }

    pub fn app_wait_for_rx(_timeout_ms: Option<u32>) -> Result<(), EventError> {
        Err(EventError::Timeout)
    }

    pub fn app_clear_rx_event() {}

    pub fn app_read_byte() -> Option<u8> {
        None
    }

    pub fn app_wait_for_tx(_timeout_ms: Option<u32>) -> Result<(), EventError> {
        Err(EventError::Timeout)
    }

    pub fn app_clear_tx_event() {}

    pub fn app_enqueue_tx_bytes(_bytes: &[u8]) -> usize {
        0
    }

    pub fn app_drain_tx() -> usize {
        0
    }

    pub fn app_stats() -> UartStats {
        UartStats::default()
    }
}

pub mod watchdog {
    pub fn start(_timeout_ms: u32) -> bool {
        false
    }

    pub fn feed() -> bool {
        false
    }
}
"@

Write-FileSafely -Path $memoryPath -Content $memoryContent
Write-FileSafely -Path $bspPath -Content $bspContent

if ($RegisterInBoardProfiles) {
    if (-not (Test-Path $profilesPath)) {
        throw "board profiles file not found: $profilesPath"
    }

    $profiles = Get-Content -Path $profilesPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($null -eq $profiles.boards) {
        throw "invalid board profiles format: missing 'boards'"
    }

    foreach ($item in @($profiles.boards)) {
        if ($item.name -eq $Board) {
            throw "board '$Board' already exists in board_profiles.json"
        }
    }

    $entry = [ordered]@{
        name = $Board
        aliases = @($Aliases)
        feature = $Feature
        target = $Target
        chip = $Chip
        probe_protocol = $ProbeProtocol
        supports = [ordered]@{
            bench = [bool]$SupportsBench
            uart_probe = [bool]$SupportsUartProbe
        }
    }

    $newBoards = New-Object System.Collections.ArrayList
    foreach ($item in @($profiles.boards)) {
        [void]$newBoards.Add($item)
    }
    [void]$newBoards.Add([pscustomobject]$entry)
    $profiles.boards = $newBoards

    if ($DryRun) {
        Write-Host ("[dry-run] update {0} with board '{1}'" -f $profilesPath, $Board)
    } else {
        $profiles | ConvertTo-Json -Depth 8 | Set-Content -Path $profilesPath -Encoding UTF8
    }
}

Write-Host ""
Write-Host "Scaffold complete."
Write-Host ("- memory template: {0}" -f $memoryPath)
Write-Host ("- bsp template:    {0}" -f $bspPath)
if ($RegisterInBoardProfiles) {
    Write-Host ("- board profile:   {0}" -f $profilesPath)
}
Write-Host ""
Write-Host "Next steps:"
Write-Host ("1) Add feature/dependency in Cargo.toml (feature: {0})" -f $Feature)
Write-Host ("2) Register module in src/bsp/mod.rs (mod {0})" -f $bspModule)
Write-Host ("3) Implement UART/LED/PWM/watchdog in src/bsp/{0}.rs" -f $bspModule)
Write-Host ("4) Build test: scripts/build/build_board.ps1 -Board {0} -Profile release -Mode app" -f $Board)
