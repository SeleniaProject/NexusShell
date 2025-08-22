# 特殊ケースの修正

# find.rsの到達不能パターンを修正（重複したパターンマッチを削除）
$findFile = "crates\nxsh_builtins\src\find.rs"
if (Test-Path $findFile) {
    $content = Get-Content $findFile -Raw
    # 2つ目のis_executableパラメータに_を追加
    $content = $content -replace "fn is_executable\(path: &Path, metadata: &Metadata\)", "fn is_executable(path: &Path, _metadata: &Metadata)"
    # 重複した到達不能パターンを削除（後で詳細な修正が必要かも）
    Set-Content -Path $findFile -Value $content
    Write-Host "Fixed find.rs unused parameter"
}

# zstd.rsの未使用代入警告を修正
$zstdFile = "crates\nxsh_builtins\src\zstd.rs"
if (Test-Path $zstdFile) {
    $content = Get-Content $zstdFile -Raw
    # MODES静的変数の安全でない参照に関する警告は複雑なので、後で対処
    Write-Host "zstd.rs requires manual review for static_mut_refs warning"
}

# groups.rsの未使用インポートを修正
$groupsFile = "crates\nxsh_builtins\src\groups.rs"
if (Test-Path $groupsFile) {
    $content = Get-Content $groupsFile -Raw
    $content = $content -replace "use std::collections::HashSet;", ""
    $content = $content -replace "use windows_sys::Win32::System::Threading::GetCurrentThread;", ""
    $content = $content -replace "use windows_sys::Win32::Security::{GetTokenInformation, TokenGroups, TOKEN_QUERY};", ""
    $content = $content -replace "use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};", ""
    Set-Content -Path $groupsFile -Value $content
    Write-Host "Fixed groups.rs unused imports"
}

# chown.rsとchgrp.rsの未使用インポートを修正
$chownFile = "crates\nxsh_builtins\src\chown.rs"
if (Test-Path $chownFile) {
    $content = Get-Content $chownFile -Raw
    $content = $content -replace "use windows::Win32::Security::SID_NAME_USE;", ""
    $content = $content -replace "    use std::fs;", ""
    $content = $content -replace "    use std::path::Path;", ""
    Set-Content -Path $chownFile -Value $content
    Write-Host "Fixed chown.rs unused imports"
}

$chgrpFile = "crates\nxsh_builtins\src\chgrp.rs"
if (Test-Path $chgrpFile) {
    $content = Get-Content $chgrpFile -Raw
    $content = $content -replace "use windows::Win32::Security::SID_NAME_USE;", ""
    $content = $content -replace "    use std::fs;", ""
    $content = $content -replace "    use std::path::Path;", ""
    Set-Content -Path $chgrpFile -Value $content
    Write-Host "Fixed chgrp.rs unused imports"
}

# arp.rsの残りの未使用インポートを修正
$arpFile = "crates\nxsh_builtins\src\arp.rs"
if (Test-Path $arpFile) {
    $content = Get-Content $arpFile -Raw
    $content = $content -replace "use std::collections::HashMap;", ""
    $content = $content -replace "use std::net::{IpAddr, Ipv4Addr};", "use std::net::IpAddr;"
    Set-Content -Path $arpFile -Value $content
    Write-Host "Fixed arp.rs remaining unused imports"
}

# time_cmd.rsの未使用インポートを修正
$timeFile = "crates\nxsh_builtins\src\time_cmd.rs"
if (Test-Path $timeFile) {
    $content = Get-Content $timeFile -Raw
    $content = $content -replace "use std::time::Duration;", ""
    Set-Content -Path $timeFile -Value $content
    Write-Host "Fixed time_cmd.rs unused import"
}

# cat.rsの未使用インポートを修正
$catFile = "crates\nxsh_builtins\src\cat.rs"
if (Test-Path $catFile) {
    $content = Get-Content $catFile -Raw
    $content = $content -replace "use std::fmt::Write as FmtWrite;", ""
    Set-Content -Path $catFile -Value $content
    Write-Host "Fixed cat.rs unused import"
}

Write-Host "Special cases batch fix completed"
