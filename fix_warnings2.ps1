# さらなる警告修正

# monitor.rsの未使用変数を修正
$monitorFile = "crates\nxsh_builtins\src\monitor.rs"
if (Test-Path $monitorFile) {
    $content = Get-Content $monitorFile -Raw
    $content = $content -replace "use std::io::{self, Read};", "use std::io::Read;"
    $content = $content -replace "let mut rows = vec!\[", "let rows = vec!["
    Set-Content -Path $monitorFile -Value $content
    Write-Host "Fixed monitor.rs"
}

# ps.rsの修正
$psFile = "crates\nxsh_builtins\src\ps.rs"
if (Test-Path $psFile) {
    $content = Get-Content $psFile -Raw
    $content = $content -replace ", ProgressBar", ""
    Set-Content -Path $psFile -Value $content
    Write-Host "Fixed ps.rs"
}

# df.rsの修正
$dfFile = "crates\nxsh_builtins\src\df.rs"
if (Test-Path $dfFile) {
    $content = Get-Content $dfFile -Raw
    $content = $content -replace "use crate::ui_design::{TableFormatter, Colorize};", "use crate::ui_design::TableFormatter;"
    $content = $content -replace "use std::ffi::{CString, OsStr};", "use std::ffi::OsStr;"
    $content = $content -replace "use winapi::shared::winerror::ERROR_SUCCESS;", ""
    $content = $content -replace "    use std::ptr;", ""
    Set-Content -Path $dfFile -Value $content
    Write-Host "Fixed df.rs"
}

# help.rsの修正
$helpFile = "crates\nxsh_builtins\src\help.rs"
if (Test-Path $helpFile) {
    $content = Get-Content $helpFile -Raw
    $content = $content -replace "use nxsh_core::{Context, ExecutionResult};", "use nxsh_core::ExecutionResult;"
    $content = $content -replace "    TableFormatter, Colorize, Animation, ProgressBar, Notification, NotificationType,", "    Colorize,"
    $content = $content -replace "    TableOptions, BorderStyle, Alignment", ""
    Set-Content -Path $helpFile -Value $content
    Write-Host "Fixed help.rs"
}

Write-Host "Batch fix completed"
