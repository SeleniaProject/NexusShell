Param(
  [string]$Profile = "release-small", # Can be comma-separated list (e.g. "release-small,release")
  [switch]$NoBuild,
  [string]$Variant, # busybox-min | busybox-max (default busybox-min)
  [string]$OutJson = "size_report.json",
  [string]$PrevJson, # Previous report path for delta computation
  [string]$HistoryDir = "size_history", # Directory to append timestamped snapshots
  [int]$DeltaFailPct # Override env if provided (percentage, e.g. 5 = +5%)
)

# -------------------------------------------------------------------------------------------------
# NexusShell BusyBox size report script (PowerShell)
# Responsibilities:
#  - Build the busybox-min binary with a size-optimized profile (unless -NoBuild specified)
#  - Measure raw size, gzip size, (optional) UPX-compressed size if upx exists in PATH
#  - Enforce maximum size threshold (default 1 MiB) unless overridden
#  - Emit machine-readable JSON + human readable colored summary
#  - Fail with non-zero exit code on threshold breach (for CI gating) unless override env set
# Environment Variables:
#  NXSH_SIZE_MAX             : Max allowed bytes (default 1048576)
#  NXSH_ALLOW_SIZE_FAILURE   : If set (any value), do not fail even if size exceeds threshold
#  NXSH_DISABLE_UPX          : If set, skip UPX attempt
#  NXSH_SIZE_DELTA_FAIL_PCT  : If set (int), and PrevJson provided, fail when size increase >= this percent
$envDelta = if($DeltaFailPct){ $DeltaFailPct } else { [Environment]::GetEnvironmentVariable('NXSH_SIZE_DELTA_FAIL_PCT') }
if([string]::IsNullOrEmpty($envDelta)) { $envDelta = $null } else {
  try { $envDelta = [int]$envDelta } catch { Write-Warn "Invalid NXSH_SIZE_DELTA_FAIL_PCT value '$envDelta' ignored"; $envDelta = $null }
}
#  NXSH_VERBOSE              : If set, prints extra diagnostic info
#  (Note) We deliberately avoid calling external *nix tools to stay pure PowerShell / Rust.
# -------------------------------------------------------------------------------------------------

$ErrorActionPreference = 'Stop'

function Write-Info($msg){ Write-Host $msg -ForegroundColor Cyan }
function Write-Warn($msg){ Write-Host $msg -ForegroundColor DarkYellow }
function Write-ErrMsg($msg){ Write-Host $msg -ForegroundColor Red }
function Write-Ok($msg){ Write-Host $msg -ForegroundColor Green }

$envMax = [Environment]::GetEnvironmentVariable('NXSH_SIZE_MAX')
# Default threshold relaxed per BusyBox sizing policy: < 1.5 MiB when not overridden
if([string]::IsNullOrEmpty($envMax)){ $envMax = '1572864' }
$maxBytes = [int]$envMax
$allowFailure = [Environment]::GetEnvironmentVariable('NXSH_ALLOW_SIZE_FAILURE')
$disableUpx = [Environment]::GetEnvironmentVariable('NXSH_DISABLE_UPX')
$verbose = [Environment]::GetEnvironmentVariable('NXSH_VERBOSE')

if(-not $Variant -or $Variant -eq ''){ $Variant = [Environment]::GetEnvironmentVariable('NXSH_VARIANT') }
if(-not $Variant -or $Variant -eq ''){ $Variant = 'busybox-min' }
if($Variant -notin @('busybox-min','busybox-max')){ throw "Invalid -Variant '$Variant' (expected busybox-min|busybox-max)" }

function Invoke-ProfileReport($p){
  if(-not $NoBuild){
    Write-Info "[size-report] Building $Variant profile=$p"
    cargo build -p nxsh_cli --no-default-features --features $Variant --profile $p | Out-Null
  } else {
    Write-Info "[size-report] Skipping build (NoBuild) for variant=$Variant profile=$p"
  }
  $exe = Join-Path (Join-Path "target" $p) "nxsh.exe"
  if (-not (Test-Path $exe)) { throw "Executable not found: $exe" }

  $fileInfo = Get-Item $exe
  $rawBytes = [int64]$fileInfo.Length
  $rawMiB = [Math]::Round($rawBytes/1MB, 3)

  # Optional strip (unless disabled)
  $stripBytes = $null
  if(-not [Environment]::GetEnvironmentVariable('NXSH_DISABLE_STRIP')){
    $stripTool = Get-Command rust-objcopy -ErrorAction SilentlyContinue
    if(-not $stripTool){ $stripTool = Get-Command llvm-strip -ErrorAction SilentlyContinue }
    if(-not $stripTool){ $stripTool = Get-Command strip -ErrorAction SilentlyContinue }
    if($stripTool){
      $tmpStrip = "$exe.stripped_tmp"
      Copy-Item $exe $tmpStrip -Force
      & $stripTool $tmpStrip 2>$null
      if(Test-Path $tmpStrip){ $stripBytes = (Get-Item $tmpStrip).Length; Remove-Item $tmpStrip -Force }
    }
  }

# Compute gzip size in memory (no temp file) – use API compatible with Windows PowerShell (.NET Framework)
Add-Type -AssemblyName System.IO.Compression.FileSystem
$gzipBytes = $null
try {
  $bytes = [System.IO.File]::ReadAllBytes($exe)
  $msOut = New-Object System.IO.MemoryStream
  # Older framework lacks 3‑arg ctor taking CompressionLevel + leaveOpen; use mode Compress
  $gzipStream = New-Object System.IO.Compression.GZipStream($msOut, [System.IO.Compression.CompressionMode]::Compress)
  $gzipStream.Write($bytes, 0, $bytes.Length)
  $gzipStream.Flush()
  $gzipStream.Dispose()
  $gzipBytes = ($msOut.ToArray()).Length
} catch {
  Write-Warn "Failed to compute gzip size: $_"
}

# Optional UPX compression attempt (only if upx present & not disabled)
$upxPath = if($disableUpx){ $null } else { Get-Command upx -ErrorAction SilentlyContinue }
$upxBytes = $null
if($upxPath){
  Write-Info "Attempting UPX compression (temporary copy)"
  $tmp = "$exe.upx_tmp"
  Copy-Item $exe $tmp -Force
  $upxResult = & upx --best --lzma $tmp 2>&1 | Out-String
  if($LASTEXITCODE -eq 0 -and (Test-Path $tmp)){
    $upxBytes = (Get-Item $tmp).Length
    Remove-Item $tmp -Force
  } else {
    Write-Warn "UPX compression failed or produced no output. Output: $upxResult"
    if(Test-Path $tmp){ Remove-Item $tmp -Force }
  }
} else {
  Write-Info "UPX not available or disabled; skipping. (Set NXSH_DISABLE_UPX=1 to silence)"
}

$status = if($rawBytes -le $maxBytes){ 'pass' } else { 'fail' }
  $json = [pscustomobject]@{
    profile = $p
    variant = $Variant
    path = $exe
    size_bytes = $rawBytes
    size_mib = $rawMiB
    stripped_size_bytes = $stripBytes
    gzip_size_bytes = $gzipBytes
    upx_size_bytes = $upxBytes
    max_allowed_bytes = $maxBytes
    status = $status
    timestamp_utc = [DateTime]::UtcNow.ToString('o')
  }

  return $json
}

$profiles = $Profile.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
$reports = @()
foreach($p in $profiles){ $reports += Invoke-ProfileReport $p }

# If multiple profiles requested, write array JSON; else single object for backward compatibility
if($reports.Count -gt 1){ $reports | ConvertTo-Json -Depth 4 | Set-Content $OutJson -Encoding UTF8 } else { $reports[0] | ConvertTo-Json -Depth 4 | Set-Content $OutJson -Encoding UTF8 }

Write-Host "--- Size Report (profiles=${($profiles -join ',')}; variant=$Variant) ---" -ForegroundColor Cyan
foreach($r in $reports){
  $sz = $r.size_bytes
  $mib = [Math]::Round($sz/1MB,3)
  Write-Host ("Profile {0}: raw={1} bytes ({2} MiB)" -f $r.profile, $sz, $mib) -ForegroundColor Yellow
  if($r.gzip_size_bytes){ Write-Host ("           gzip={0} bytes" -f $r.gzip_size_bytes) -ForegroundColor DarkYellow }
  if($r.upx_size_bytes){ Write-Host ("           upx={0} bytes" -f $r.upx_size_bytes) -ForegroundColor DarkYellow }
}
$last = $reports[-1]
Write-Host "Threshold: $maxBytes bytes" -ForegroundColor Yellow

# Delta from previous JSON if provided
if($PrevJson -and (Test-Path $PrevJson)){
  try {
    $prev = Get-Content $PrevJson -Raw | ConvertFrom-Json
  $delta = [int64]$last.size_bytes - [int64]$prev.size_bytes
    $sign = if($delta -ge 0){ "+" } else { "" }
    Write-Host ("Delta vs prev: {0}{1} bytes" -f $sign, $delta) -ForegroundColor Cyan
  $last | Add-Member -NotePropertyName prev_size_bytes -NotePropertyValue ([int64]$prev.size_bytes)
  $last | Add-Member -NotePropertyName delta_bytes -NotePropertyValue ([int64]$delta)
  if($envDelta -and $prev.size_bytes -gt 0){
      $pct = [math]::Round(($delta / [double]$prev.size_bytes)*100, 3)
  $last | Add-Member -NotePropertyName delta_pct -NotePropertyValue $pct
      Write-Host ("Delta pct: {0}{1}%" -f (if($pct -ge 0){"+"} else {""}), $pct) -ForegroundColor Cyan
      if($pct -ge $envDelta -and -not $allowFailure){
        Write-ErrMsg "SIZE DELTA ALERT: increase ${pct}% >= threshold ${envDelta}%"
        [Environment]::Exit(3)
      } elseif($pct -ge $envDelta) {
        Write-Warn "SIZE DELTA ALERT (override active): increase ${pct}% >= threshold ${envDelta}%"
      }
    }
  } catch {
    Write-Warn "Failed to compute delta from $PrevJson : $_"
  }
} elseif($PrevJson){
  Write-Warn "PrevJson path '$PrevJson' not found"
}

# Optional history snapshot
if($HistoryDir){
  try {
    if(-not (Test-Path $HistoryDir)){ New-Item -ItemType Directory -Path $HistoryDir | Out-Null }
    $stamp = (Get-Date).ToString('yyyyMMdd_HHmmss')
    $histFile = Join-Path $HistoryDir ("size_${Variant}_${Profile}_$stamp.json")
  # Store entire array when multi-profile else single object
  if($reports.Count -gt 1){ $reports | ConvertTo-Json -Depth 4 | Set-Content $histFile -Encoding UTF8 } else { $last | ConvertTo-Json -Depth 4 | Set-Content $histFile -Encoding UTF8 }
    if($verbose){ Write-Info "History snapshot written to $histFile" }
  } catch {
    Write-Warn "Failed to write history snapshot: $_"
  }
}

if($reports | Where-Object { $_.status -eq 'fail' }){
  Write-ErrMsg "STATUS: One or more profiles exceeded threshold (>$maxBytes)"
  if(-not $allowFailure){ [Environment]::Exit(2) } else { Write-Warn "Override active (NXSH_ALLOW_SIZE_FAILURE). Not failing." }
} else {
  Write-Ok "STATUS: within threshold (<=$maxBytes) for all profiles"
}

if($verbose){ Write-Info ("JSON report written to {0}" -f (Resolve-Path $OutJson)) }
