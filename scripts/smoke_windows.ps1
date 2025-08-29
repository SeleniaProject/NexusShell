param(
    [switch]$VerboseOutput
)

$ErrorActionPreference = 'Stop'

function Run-Test($name, $cmd, $expect) {
    Write-Host "[RUN] $name" -ForegroundColor Cyan
    $raw = & "$PSScriptRoot\..\target\release\nxsh.exe" -c $cmd 2>&1
    $out = if ($raw -is [System.Array]) { $raw -join "`n" } else { [string]$raw }
    if ($VerboseOutput) { Write-Host "--- OUTPUT ---`n$out`n--------------" }
    if ($out -notmatch $expect) {
        Write-Host "[FAIL] $name" -ForegroundColor Red
        Write-Host "Expect regex: $expect" -ForegroundColor DarkGray
        Write-Host "Actual:\n$out" -ForegroundColor DarkGray
        exit 1
    }
    Write-Host "[OK] $name" -ForegroundColor Green
}

# Basic pipeline success -> && RHS runs
Run-Test "pipe &&" "echo foo | findstr foo && echo OK" "(?ms)^foo\s*\r?\n?OK\s*$"

# Pipeline fail -> || RHS runs
Run-Test "pipe ||" "echo foo | findstr nope || echo OK2" "(?ms)^OK2\s*$"

# Multiple stages pipeline
Run-Test "pipe 2-stage" "echo alpha beta gamma | findstr beta && echo OK3" "(?ms)^alpha beta gamma\s*\r?\n?OK3\s*$"

Write-Host "All smoke tests passed." -ForegroundColor Green
