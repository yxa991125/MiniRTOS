param(
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

$manifest = "host_tests/Cargo.toml"
& cargo test --manifest-path $manifest --target $Target
if ($LASTEXITCODE -ne 0) {
    throw "host tests failed"
}
