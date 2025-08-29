Param(
  [switch]$IncludeCli
)
$ErrorActionPreference = 'Stop'
## Prepare log file to capture all outputs
$repoRoot = Join-Path $PSScriptRoot '..'
$logDir = Join-Path $repoRoot 'reports'
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Force -Path $logDir | Out-Null }
$logPath = Join-Path $logDir 'test_last.log'
# Pre-create an empty log to ensure the path exists even if early failure occurs
New-Item -ItemType File -Force -Path $logPath | Out-Null
Write-Host "[nxsh] Logging to: $logPath"
Write-Host "[nxsh] Stopping possible leftover test processes..."
# Stop common rust build/test related processes that may hold file locks
foreach ($p in @('nxsh','cargo','rustc','rust-analyzer')) {
  Get-Process $p -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# Stop hashed test executables under target/*/deps
$targetRoot = Join-Path $PSScriptRoot '..' | Join-Path -ChildPath 'target'
if (Test-Path $targetRoot) {
  Get-ChildItem $targetRoot -Recurse -ErrorAction SilentlyContinue -Filter *.exe |
    Where-Object { $_.FullName -match '\\deps\\' } |
    ForEach-Object {
      $base = $_.BaseName
      try { Stop-Process -Name $base -Force -ErrorAction SilentlyContinue } catch {}
    }
}
Write-Host "[nxsh] cargo clean..."
(& { cargo clean 2>&1 | Tee-Object -FilePath $logPath -Append })
Write-Host "[nxsh] cargo test (workspace, exclude nxsh_cli)"
(& { cargo test -q --workspace --exclude nxsh_cli -- --test-threads=1 2>&1 | Tee-Object -FilePath $logPath -Append })
if ($LASTEXITCODE -ne 0) { Write-Host "[nxsh] Test failed. See: $logPath"; exit $LASTEXITCODE }
if ($IncludeCli) {
  Write-Host "[nxsh] cargo test (include cli)"
  (& { cargo test -q -- --test-threads=1 2>&1 | Tee-Object -FilePath $logPath -Append })
  Write-Host "[nxsh] CLI Test exit: $LASTEXITCODE. Log: $logPath"
  exit $LASTEXITCODE
}
Write-Host "[nxsh] Test log: $logPath"
