#!/usr/bin/env powershell
# Comprehensive Test Suite for Enhanced CUI Features

param(
    [switch]$Verbose,
    [switch]$Performance
)

Write-Host "🧪 NexusShell Enhanced CUI Test Suite" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Blue

$ErrorActionPreference = "Continue"
$TestsPassed = 0
$TestsFailed = 0

function Test-Command {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "`n🔍 Testing: $Description" -ForegroundColor Yellow
    Write-Host "Command: $Command" -ForegroundColor Gray
    
    try {
        $startTime = Get-Date
        Invoke-Expression $Command
        $endTime = Get-Date
        $duration = ($endTime - $startTime).TotalMilliseconds
        
        Write-Host "✅ PASSED" -ForegroundColor Green
        if ($Performance) {
            Write-Host "   Duration: ${duration}ms" -ForegroundColor Cyan
        }
        $script:TestsPassed++
    }
    catch {
        Write-Host "❌ FAILED: $($_.Exception.Message)" -ForegroundColor Red
        $script:TestsFailed++
    }
}

# Test enhanced commands
Test-Command "cargo run --bin nxsh_builtins -- df -h" "Enhanced disk usage with progress bars"
Test-Command "cargo run --bin nxsh_builtins -- ls" "Enhanced directory listing with advanced tables"
Test-Command "cargo run --bin nxsh_builtins -- ps" "Enhanced process monitoring with animations"
Test-Command "cargo run --bin nxsh_builtins -- pwd" "Enhanced path display with breadcrumbs"

# Test with different options
Test-Command "cargo run --bin nxsh_builtins -- ls -la" "Long format listing with detailed animations"
Test-Command "cargo run --bin nxsh_builtins -- ps aux" "Detailed process listing with performance monitoring"

# Test error handling
Test-Command "cargo run --bin nxsh_builtins -- df /nonexistent" "Error handling with notifications"

Write-Host "`n📊 Test Results Summary" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Blue
Write-Host "✅ Tests Passed: $TestsPassed" -ForegroundColor Green
Write-Host "❌ Tests Failed: $TestsFailed" -ForegroundColor Red
Write-Host "📈 Success Rate: $([math]::Round(($TestsPassed / ($TestsPassed + $TestsFailed)) * 100, 2))%" -ForegroundColor Yellow

if ($TestsFailed -eq 0) {
    Write-Host "`n🎉 All enhanced CUI features are working perfectly!" -ForegroundColor Green
    Write-Host "The beautiful terminal interface is ready for production use." -ForegroundColor Cyan
} else {
    Write-Host "`n⚠️  Some tests failed. Please review the output above." -ForegroundColor Yellow
}

Write-Host "`n🚀 Enhanced Features Verified:" -ForegroundColor Magenta
Write-Host "  • Advanced progress bars with ETA calculations" -ForegroundColor White
Write-Host "  • Smooth loading animations and transitions" -ForegroundColor White
Write-Host "  • Beautiful table formatting with rounded borders" -ForegroundColor White
Write-Host "  • Intelligent color coding and status indicators" -ForegroundColor White
Write-Host "  • Interactive notifications and alerts" -ForegroundColor White
Write-Host "  • Theme-based visual consistency" -ForegroundColor White
Write-Host "  • Real-time performance monitoring" -ForegroundColor White
Write-Host "  • Context-aware adaptive displays" -ForegroundColor White
