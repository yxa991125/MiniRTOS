param(
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
$manifest = Join-Path $repoRoot "host_tests/Cargo.toml"
& cargo test --manifest-path $manifest --target $Target
if ($LASTEXITCODE -ne 0) {
    throw "host tests failed"
}
