Param(
    [switch]$IncludeIgnored,
    [string]$TargetDir = "target_matrix"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Invoke-CI {
    param(
        [string]$Name,
        [string]$Cmd
    )
    Write-Host "=== RUN: $Name ===" -ForegroundColor Cyan
    Write-Host "    $Cmd" -ForegroundColor DarkGray
    # Run in a child PowerShell but force-propagate the native exit code to the parent
    $inner = "$Cmd; exit $LASTEXITCODE"
    & powershell -NoProfile -ExecutionPolicy Bypass -Command $inner
    if ($LASTEXITCODE -ne 0) {
        throw "FAILED: $Name (exit $LASTEXITCODE)"
    }
}

$env:CARGO_TERM_COLOR = "always"
$env:RUST_BACKTRACE = "1"
$env:CARGO_TARGET_DIR = $TargetDir

# Build common test args
$testArgs = "-q -- --test-threads=1"
if ($IncludeIgnored) {
    $testArgs = "-q -- --test-threads=1 --ignored"
}

$steps = @(
    @{ name = "workspace (default features)"; cmd = "cargo test $testArgs" },
    @{ name = "nxsh_core (full features)";   cmd = "cargo test -p nxsh_core --features full $testArgs" },
    @{ name = "nxsh_core (busybox_min)";     cmd = "cargo test -p nxsh_core --features busybox_min $testArgs" },
    @{ name = "nxsh_plugin (full)";          cmd = "cargo test -p nxsh_plugin --features full $testArgs" },
    @{ name = "nxsh_plugin (minimal)";       cmd = "cargo test -p nxsh_plugin --no-default-features --features minimal $testArgs" },
    @{ name = "nxsh_plugin (secure)";        cmd = "cargo test -p nxsh_plugin --no-default-features --features secure $testArgs" },
    @{ name = "nxsh_builtins";               cmd = "cargo test -p nxsh_builtins $testArgs" }
)

$failures = @()
$start = Get-Date
try {
    foreach ($s in $steps) {
        try {
            Invoke-CI -Name $s.name -Cmd $s.cmd
        }
        catch {
            $failures += $s.name
            Write-Host $_ -ForegroundColor Red
        }
    }
}
finally {
    $elapsed = (Get-Date) - $start
    Write-Host "=== SUMMARY ===" -ForegroundColor Yellow
    if ($failures.Count -eq 0) {
        Write-Host "All test steps passed." -ForegroundColor Green
    } else {
        Write-Host ("Failures: {0}" -f ($failures -join ", ")) -ForegroundColor Red
        exit 1
    }
    Write-Host ("Elapsed: {0:N1}s" -f $elapsed.TotalSeconds)
}
