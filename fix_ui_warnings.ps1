# UI関連ファイルの警告修正

# enhanced_line_editor.rs
$enhancedFile = "crates\nxsh_ui\src\enhanced_line_editor.rs"
if (Test-Path $enhancedFile) {
    $content = Get-Content $enhancedFile -Raw
    $content = $content -replace "use anyhow::{Result, Context};", "use anyhow::Result;"
    $content = $content -replace "    terminal::{enable_raw_mode, disable_raw_mode, size},", "    terminal::{enable_raw_mode, disable_raw_mode},"
    $content = $content -replace "    cursor::{MoveTo, Show, Hide},", "    cursor::MoveTo,"
    $content = $content -replace "    style::{Print, ResetColor},", "    style::ResetColor,"
    $content = $content -replace "    sync::{Arc, Mutex},", ""
    $content = $content -replace "    completion_panel::CompletionPanel,", ""
    Set-Content -Path $enhancedFile -Value $content
    Write-Host "Fixed enhanced_line_editor.rs"
}

# completion.rs
$completionFile = "crates\nxsh_ui\src\completion.rs"
if (Test-Path $completionFile) {
    $content = Get-Content $completionFile -Raw
    $content = $content -replace "use crate::completion_engine::{AdvancedCompletionEngine, CompletionResult};", "use crate::completion_engine::AdvancedCompletionEngine;"
    Set-Content -Path $completionFile -Value $content
    Write-Host "Fixed completion.rs"
}

# completion_panel.rs
$panelFile = "crates\nxsh_ui\src\completion_panel.rs"
if (Test-Path $panelFile) {
    $content = Get-Content $panelFile -Raw
    $content = $content -replace "    time::{Duration, Instant},", "    time::Instant,"
    $content = $content -replace "...ompletionCandidate, CompletionResult, CompletionContext};", "...ompletionCandidate};"
    Set-Content -Path $panelFile -Value $content
    Write-Host "Fixed completion_panel.rs"
}

# tab_completion.rs
$tabFile = "crates\nxsh_ui\src\tab_completion.rs"
if (Test-Path $tabFile) {
    $content = Get-Content $tabFile -Raw
    $content = $content -replace "    event::{Event, KeyCode, KeyEvent, KeyModifiers},", "    event::{KeyCode, KeyEvent, KeyModifiers},"
    $content = $content -replace "    io::{self, Write},", "    io,"
    $content = $content -replace "    completion_engine::{AdvancedCompletionEngine, CompletionContext},", "    completion_engine::AdvancedCompletionEngine,"
    Set-Content -Path $tabFile -Value $content
    Write-Host "Fixed tab_completion.rs"
}

# completion_metrics.rs
$metricsFile = "crates\nxsh_ui\src\completion_metrics.rs"
if (Test-Path $metricsFile) {
    $content = Get-Content $metricsFile -Raw
    $content = $content -replace "use std::time::{Instant, Duration};", "use std::time::Instant;"
    Set-Content -Path $metricsFile -Value $content
    Write-Host "Fixed completion_metrics.rs"
}

# advanced_cui.rs
$cuiFile = "crates\nxsh_ui\src\advanced_cui.rs"
if (Test-Path $cuiFile) {
    $content = Get-Content $cuiFile -Raw
    $content = $content -replace "use anyhow::{Result, Context};", "use anyhow::Result;"
    $content = $content -replace "    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},", "    style::Color,"
    $content = $content -replace "    execute, terminal,", "    terminal,"
    $content = $content -replace "    collections::HashMap,", ""
    $content = $content -replace "    fmt::Write as FmtWrite,", ""
    $content = $content -replace "    io::{self, Write},", ""
    Set-Content -Path $cuiFile -Value $content
    Write-Host "Fixed advanced_cui.rs"
}

# universal_formatter.rs
$formatterFile = "crates\nxsh_ui\src\universal_formatter.rs"
if (Test-Path $formatterFile) {
    $content = Get-Content $formatterFile -Raw
    $content = $content -replace "use std::collections::HashMap;", ""
    Set-Content -Path $formatterFile -Value $content
    Write-Host "Fixed universal_formatter.rs"
}

Write-Host "UI files batch fix completed"
