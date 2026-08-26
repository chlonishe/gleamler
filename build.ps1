param(
    [switch]$Run
)

$ErrorActionPreference = "Stop"

Write-Host "==> Building Rust NIF..." -ForegroundColor Cyan
cargo build -p gleamler --release --features stress

New-Item -ItemType Directory -Force -Path priv | Out-Null

if ($IsWindows -or $env:OS -eq "Windows_NT") {
    $targetSubdir = if ($env:CARGO_BUILD_TARGET) { 
        Join-Path "target" $env:CARGO_BUILD_TARGET "release"
    } else { 
        "target/release"
    }

    $dllPaths = @(
        Join-Path $targetSubdir "gleamler.dll"
        "target/release/gleamler.dll"
    )
    $dllPath = $dllPaths | Where-Object { Test-Path $_ } | Select-Object -First 1

    if (-not $dllPath) {
        throw "Could not find gleamler.dll. Looked in: $($dllPaths -join ', ')"
    }

    Copy-Item $dllPath priv/gleamler.dll -Force
} else {
    $ext = if (& uname -s | Select-String -Pattern "Darwin") { "dylib" } else { "so" }
    Copy-Item target/release/libgleamler.$ext priv/gleamler.so -Force
}

Write-Host "==> Generating Erlang / Gleam stubs..." -ForegroundColor Cyan
cargo run -p gleamler_codegen -- gleamler src/gleamler_nif_ffi.erl src/gleamler_nif.gleam

Write-Host "==> Building Gleam..." -ForegroundColor Cyan
gleam build

if ($Run) {
    Write-Host "==> Running..." -ForegroundColor Green
    gleam run
}

Write-Host "==> Done!" -ForegroundColor Green
