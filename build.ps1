param(
    [switch]$Gen,
    [switch]$Run
)

$ErrorActionPreference = "Stop"

if ($Gen -or $Run) {
    Write-Host "==> Generating FFI stubs..." -ForegroundColor Cyan
    cargo run -p gleamler_codegen -- gleamler/src/lib.rs src/gleamler_nif_ffi.erl src/gleamler_nif.gleam
}

Write-Host "==> Building Rust NIF..." -ForegroundColor Cyan
cargo build -p gleamler --release

New-Item -ItemType Directory -Force -Path priv | Out-Null
Copy-Item target/release/gleamler.dll priv/gleamler.dll -Force

Write-Host "==> Building Gleam..." -ForegroundColor Cyan
gleam build

if ($Run) {
    Write-Host "==> Running..." -ForegroundColor Green
    gleam run
}

Write-Host "==> Done!" -ForegroundColor Green