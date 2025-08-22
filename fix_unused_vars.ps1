# 未使用変数の警告修正

# mv.rsの未使用変数を修正
$mvFile = "crates\nxsh_builtins\src\mv.rs"
if (Test-Path $mvFile) {
    $content = Get-Content $mvFile -Raw
    $content = $content -replace "let dst_wide: Vec<u16>", "let _dst_wide: Vec<u16>"
    Set-Content -Path $mvFile -Value $content
    Write-Host "Fixed mv.rs unused variable"
}

# cut.rsの未使用変数を修正
$cutFile = "crates\nxsh_builtins\src\cut.rs"
if (Test-Path $cutFile) {
    $content = Get-Content $cutFile -Raw
    $content = $content -replace "let formatter = TableFormatter::new\(\);", "let _formatter = TableFormatter::new();"
    Set-Content -Path $cutFile -Value $content
    Write-Host "Fixed cut.rs unused variable"
}

# smart_alias.rsの未使用変数を修正
$aliasFile = "crates\nxsh_builtins\src\smart_alias.rs"
if (Test-Path $aliasFile) {
    $content = Get-Content $aliasFile -Raw
    $content = $content -replace "if let Ok\(content\) = fs::read_to_string\(path\)", "if let Ok(_content) = fs::read_to_string(path)"
    $content = $content -replace "let formatter = TableFormatter::new\(\);", "let _formatter = TableFormatter::new();"
    Set-Content -Path $aliasFile -Value $content
    Write-Host "Fixed smart_alias.rs unused variables"
}

# json_commands.rsの未使用変数を修正
$jsonFile = "crates\nxsh_builtins\src\json_commands.rs"
if (Test-Path $jsonFile) {
    $content = Get-Content $jsonFile -Raw
    $content = $content -replace "pub fn to_json_cli\(args: &\[String\]\)", "pub fn to_json_cli(_args: &[String])"
    Set-Content -Path $jsonFile -Value $content
    Write-Host "Fixed json_commands.rs unused variable"
}

# ps.rsの未使用変数を修正
$psFile = "crates\nxsh_builtins\src\ps.rs"
if (Test-Path $psFile) {
    $content = Get-Content $psFile -Raw
    $content = $content -replace "fn display_processes_json\(processes: &\[ProcessEntry\], fields: &\[&str\], options: &PsOptions\)", "fn display_processes_json(processes: &[ProcessEntry], fields: &[&str], _options: &PsOptions)"
    Set-Content -Path $psFile -Value $content
    Write-Host "Fixed ps.rs unused variable"
}

# chown.rsの未使用変数を修正
$chownFile = "crates\nxsh_builtins\src\chown.rs"
if (Test-Path $chownFile) {
    $content = Get-Content $chownFile -Raw
    $content = $content -replace "fn set_windows_owner\(path: &Path, uid: Option<u32>, gid: Option<u32>, dereference: bool\)", "fn set_windows_owner(path: &Path, uid: Option<u32>, gid: Option<u32>, _dereference: bool)"
    Set-Content -Path $chownFile -Value $content
    Write-Host "Fixed chown.rs unused variable"
}

# arp.rsの未使用変数を修正
$arpFile = "crates\nxsh_builtins\src\arp.rs"
if (Test-Path $arpFile) {
    $content = Get-Content $arpFile -Raw
    $content = $content -replace "fn arp_display\(target_ip: Option<&str>, display_all: bool, interface: Option<&str>\)", "fn arp_display(target_ip: Option<&str>, _display_all: bool, interface: Option<&str>)"
    $content = $content -replace "fn add_arp_entry\(ip: &str, hw_addr: &str, interface: Option<&str>\)", "fn add_arp_entry(ip: &str, hw_addr: &str, _interface: Option<&str>)"
    $content = $content -replace "fn delete_arp_entry\(ip: &str, interface: Option<&str>\)", "fn delete_arp_entry(ip: &str, _interface: Option<&str>)"
    $content = $content -replace "let mut cmd_args = vec!\[", "let cmd_args = vec!["
    Set-Content -Path $arpFile -Value $content
    Write-Host "Fixed arp.rs unused variables"
}

Write-Host "Unused variables batch fix completed"
