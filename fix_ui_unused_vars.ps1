# UI関連の未使用変数修正

# completion.rsの未使用変数を修正
$completionFile = "crates\nxsh_ui\src\completion.rs"
if (Test-Path $completionFile) {
    $content = Get-Content $completionFile -Raw
    $content = $content -replace "let engine_clone = Arc::clone\(engine\);", "let _engine_clone = Arc::clone(engine);"
    $content = $content -replace "let ctx_str = ctx.to_string\(\);", "let _ctx_str = ctx.to_string();"
    Set-Content -Path $completionFile -Value $content
    Write-Host "Fixed completion.rs unused variables"
}

# completion_engine.rsの未使用変数を修正
$engineFile = "crates\nxsh_ui\src\completion_engine.rs"
if (Test-Path $engineFile) {
    $content = Get-Content $engineFile -Raw
    $content = $content -replace "let command_cache = Arc::clone\(&self.command_cache\);", "let _command_cache = Arc::clone(&self.command_cache);"
    Set-Content -Path $engineFile -Value $content
    Write-Host "Fixed completion_engine.rs unused variable"
}

# tab_completion.rsの未使用変数を修正
$tabFile = "crates\nxsh_ui\src\tab_completion.rs"
if (Test-Path $tabFile) {
    $content = Get-Content $tabFile -Raw
    $content = $content -replace "\(KeyCode::Char\(ch\), _\)", "(KeyCode::Char(_ch), _)"
    $content = $content -replace "let \(term_width, term_height\)", "let (_term_width, term_height)"
    Set-Content -Path $tabFile -Value $content
    Write-Host "Fixed tab_completion.rs unused variables"
}

# completion_panel.rsの未使用変数を修正
$panelFile = "crates\nxsh_ui\src\completion_panel.rs"
if (Test-Path $panelFile) {
    $content = Get-Content $panelFile -Raw
    $content = $content -replace "let alpha = \(self.animation_state.current_opacity \* 255.0\) as u8;", "let _alpha = (self.animation_state.current_opacity * 255.0) as u8;"
    Set-Content -Path $panelFile -Value $content
    Write-Host "Fixed completion_panel.rs unused variable"
}

# visual_completion_demo.rsの未使用変数を修正
$demoFile = "crates\nxsh_cli\examples\visual_completion_demo.rs"
if (Test-Path $demoFile) {
    $content = Get-Content $demoFile -Raw
    $content = $content -replace "let editor = EnhancedLineEditor::with_config\(config\)\?;", "let _editor = EnhancedLineEditor::with_config(config)?;"
    Set-Content -Path $demoFile -Value $content
    Write-Host "Fixed visual_completion_demo.rs unused variable"
}

# main.rsの未使用変数を修正
$mainFile = "crates\nxsh_cli\src\main.rs"
if (Test-Path $mainFile) {
    $content = Get-Content $mainFile -Raw
    $content = $content -replace "let execution_time = start_time.elapsed\(\);", "let _execution_time = start_time.elapsed();"
    Set-Content -Path $mainFile -Value $content
    Write-Host "Fixed main.rs unused variable"
}

# xz.rsのmutの警告を修正
$xzFile = "crates\nxsh_builtins\src\xz.rs"
if (Test-Path $xzFile) {
    $content = Get-Content $xzFile -Raw
    $content = $content -replace "let mut temp_file = NamedTempFile::new\(\).unwrap\(\);", "let temp_file = NamedTempFile::new().unwrap();"
    Set-Content -Path $xzFile -Value $content
    Write-Host "Fixed xz.rs unnecessary mut"
}

Write-Host "UI unused variables batch fix completed"
