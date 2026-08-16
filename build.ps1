param(
    [switch]$Gen,
    [switch]$Run
)

$ErrorActionPreference = "Stop"

$rustSrc = Get-ChildItem -Path "gleamler/src" -Filter "*.rs" -Recurse | Select-Object -ExpandProperty FullName
$erlOut = "src/gleamler_nif_ffi.erl"
$gleamOut = "src/gleamler_nif.gleam"

$needsGen = $Gen
if (-not $needsGen) {
    if (-not (Test-Path $erlOut) -or -not (Test-Path $gleamOut)) {
        $needsGen = $true
    } else {
        $outTime = [datetime]::MinValue
        if (Test-Path $erlOut) { $outTime = [math]::Max($outTime, (Get-Item $erlOut).LastWriteTime) }
        if (Test-Path $gleamOut) { $outTime = [math]::Max($outTime, (Get-Item $gleamOut).LastWriteTime) }
        
        foreach ($src in $rustSrc) {
            if ((Get-Item $src).LastWriteTime -gt $outTime) {
                $needsGen = $true
                break
            }
        }
    }
}

if ($needsGen) {
    Write-Host "==> Generating FFI stubs..." -ForegroundColor Cyan
    cargo run -p gleamler_codegen -- gleamler/src/lib.rs $erlOut $gleamOut
}

Write-Host "==> Building Rust NIF..." -ForegroundColor Cyan
cargo build -p gleamler --release

New-Item -ItemType Directory -Force -Path priv | Out-Null

if ($IsWindows -or $env:OS -eq "Windows_NT") {
    Copy-Item target/release/gleamler.dll priv/gleamler.dll -Force
} else {
    $ext = if (& uname -s | Select-String -Pattern "Darwin") { "dylib" } else { "so" }
    Copy-Item target/release/libgleamler.$ext priv/gleamler.$ext -Force
}

Write-Host "==> Building Gleam..." -ForegroundColor Cyan
gleam build

if ($Run) {
    Write-Host "==> Running..." -ForegroundColor Green
    gleam run
}

Write-Host "==> Done!" -ForegroundColor Green