param(
    [switch]$Run,
    [switch]$Valgrind,
    [switch]$LeakTest,
    [switch]$DrMemory
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
cargo run -p gleamler_codegen -- gleamler src/gleamler_nif_ffi.erl src/gleamler_nif.gleam --with-stress

Write-Host "==> Building Gleam..." -ForegroundColor Cyan
gleam build

if ($LeakTest) {
    Write-Host "==> Running resource leak test (cross-platform)..." -ForegroundColor Cyan
    
    $gleamEbin = "build/dev/erlang/gleamler/ebin"
    if (-not (Test-Path $gleamEbin)) {
        $gleamEbin = (Get-ChildItem -Path "build/dev/erlang" -Recurse -Filter "ebin" -Directory | Select-Object -First 1).FullName
    }
    if (-not $gleamEbin) {
        throw "Could not find ebin directory in build/dev/erlang"
    }

    $gleamPriv = Join-Path (Split-Path $gleamEbin -Parent) "priv"
    if (Test-Path "priv") {
        if (Test-Path $gleamPriv) {
            Remove-Item -Recurse -Force $gleamPriv
        }
        Copy-Item -Recurse -Path "priv" -Destination $gleamPriv -Force | Out-Null
        Write-Host "    Copied priv -> $gleamPriv" -ForegroundColor DarkGray
    }

    $erlTestFile = "src/resource_leak_test.erl"
    if (Test-Path $erlTestFile) {
        Write-Host "    Compiling $erlTestFile -> $gleamEbin" -ForegroundColor DarkGray
        erlc -o $gleamEbin $erlTestFile
    }

    & erl +S 1:1 -noshell -pa $gleamEbin -s resource_leak_test run -s init stop
    if ($LASTEXITCODE -ne 0) {
        throw "Leak test failed with exit code $LASTEXITCODE"
    }
    Write-Host "==> Leak test passed!" -ForegroundColor Green
    exit 0
}

if ($DrMemory) {
    if (-not ($IsWindows -or $env:OS -eq "Windows_NT")) {
        Write-Host "ERROR: -DrMemory is intended for Windows only." -ForegroundColor Red
        Write-Host "On Linux/macOS use -Valgrind or -LeakTest." -ForegroundColor Yellow
        exit 1
    }
    $drmemory = Get-Command drmemory -ErrorAction SilentlyContinue
    if (-not $drmemory) {
        Write-Host "ERROR: Dr. Memory not found in PATH." -ForegroundColor Red
        Write-Host "Download from https://drmemory.org/ and add to PATH." -ForegroundColor Yellow
        Write-Host "Alternatively, use -LeakTest for a pure Erlang memory check." -ForegroundColor Yellow
        exit 1
    }
    Write-Host "==> Running Dr. Memory (Windows Valgrind alternative)..." -ForegroundColor Cyan
    $erlPath = (Get-Command erl).Source
    & drmemory -leaks_only -count_leaks -- $erlPath +S 1:1 -noshell -pa build/dev/erlang/*/ebin -s resource_leak_test run -s init stop
    if ($LASTEXITCODE -ne 0) {
        throw "Dr. Memory detected issues or exited with code $LASTEXITCODE"
    }
    Write-Host "==> Dr. Memory check passed!" -ForegroundColor Green
    exit 0
}

if ($Valgrind) {
    if ($IsWindows -or $env:OS -eq "Windows_NT") {
        Write-Host "==> ERROR: Valgrind is not available on Windows." -ForegroundColor Red
        Write-Host "    Use -LeakTest (recommended) or install Dr. Memory and use -DrMemory." -ForegroundColor Yellow
        exit 1
    }
    Write-Host "==> Running Valgrind resource leak test..." -ForegroundColor Cyan
    $make = Get-Command make -ErrorAction SilentlyContinue
    if ($make) {
        & make valgrind-test
    } else {
        $valgrind = Get-Command valgrind -ErrorAction SilentlyContinue
        if (-not $valgrind) {
            Write-Host "ERROR: neither 'make' nor 'valgrind' found in PATH." -ForegroundColor Red
            exit 1
        }
        & sh -c 'valgrind --leak-check=full --show-leak-kinds=definite,indirect --errors-for-leak-kinds=definite,indirect --error-exitcode=1 --suppressions=valgrind.supp erl +S 1:1 -noshell -pa build/dev/erlang/*/ebin -s resource_leak_test run -s init stop'
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Valgrind detected memory leaks or exited with code $LASTEXITCODE"
    }
    Write-Host "==> Valgrind test passed!" -ForegroundColor Green
    exit 0
}

if ($Run) {
    Write-Host "==> Running..." -ForegroundColor Green
    gleam run
}

Write-Host "==> Done!" -ForegroundColor Green
