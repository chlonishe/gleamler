param(
    [switch]$Run,
    [switch]$Valgrind,
    [switch]$LeakTest,
    [switch]$DrMemory
)

$ErrorActionPreference = "Stop"

if ($DrMemory) {
    Write-Host "DrMemory is not supported via xtask. Use -LeakTest or -Valgrind." -ForegroundColor Red
    exit 1
}

$subcmd = if ($Valgrind) { "test --valgrind" }
          elseif ($LeakTest) { "test --leak" }
          elseif ($Run) { "test --gleam" }
          else { "build --release --stress" }

$cargo = (Get-Command cargo).Source
& $cargo xtask @($subcmd -split ' ')
