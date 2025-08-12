#!/usr/bin/env bash
# -------------------------------------------------------------------------------------------------
# NexusShell BusyBox size report script (POSIX shell / bash)
# Mirrors functionality of size_report.ps1 for CI on Linux/macOS.
# Responsibilities:
#  - Build busybox-min binary (unless NXSH_NO_BUILD=1)
#  - Measure raw size, gzip size, optional UPX size
#  - Enforce size threshold (default 1 MiB) failing with non-zero exit unless override
#  - Emit JSON report (jq-free: we construct manually) to size_report.json
# Environment Variables:
#  NXSH_SIZE_MAX           : Max allowed bytes (default 1048576)
#  NXSH_ALLOW_SIZE_FAILURE : If set, do not fail on threshold breach
#  NXSH_DISABLE_UPX        : If set, skip UPX attempt
#  NXSH_NO_BUILD           : Skip cargo build
#  PROFILE                 : Cargo profile (default release-small)
# -------------------------------------------------------------------------------------------------
set -euo pipefail
PROFILE="${PROFILE:-release-small}"
MAX_BYTES="${NXSH_SIZE_MAX:-1048576}"
OUT_JSON="size_report.json"
if [[ -z "${NXSH_NO_BUILD:-}" ]]; then
  echo "[size-report] Building busybox-min profile=$PROFILE" >&2
  cargo build -p nxsh_cli --no-default-features --features busybox-min --profile "$PROFILE" >/dev/null
else
  echo "[size-report] Skipping build (NXSH_NO_BUILD=1)" >&2
fi
EXE="target/${PROFILE}/nxsh"
if [[ "$OS" == "Windows_NT" ]]; then
  EXE+=".exe"
fi
if [[ ! -f "$EXE" ]]; then
  echo "Executable not found: $EXE" >&2
  exit 1
fi
RAW_BYTES=$(stat -c %s "$EXE" 2>/dev/null || stat -f %z "$EXE")
RAW_MIB=$(awk -v b="$RAW_BYTES" 'BEGIN{printf "%.3f", b/1024/1024}')
# gzip size (pipe to gzip -9)
GZIP_BYTES=$(gzip -9 <"$EXE" | wc -c | tr -d ' ')
UPX_BYTES="null"
if [[ -z "${NXSH_DISABLE_UPX:-}" ]] && command -v upx >/dev/null 2>&1; then
  TMP="$EXE.upx_tmp"
  cp "$EXE" "$TMP"
  if upx --best --lzma "$TMP" >/dev/null 2>&1; then
    UPX_BYTES=$(stat -c %s "$TMP" 2>/dev/null || stat -f %z "$TMP")
  else
    echo "[size-report] UPX compression failed (ignored)" >&2
    UPX_BYTES="null"
  fi
  rm -f "$TMP"
fi
STATUS="fail"
if [[ "$RAW_BYTES" -le "$MAX_BYTES" ]]; then STATUS="pass"; fi
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
# Manual JSON construction (values numeric except strings)
cat >"$OUT_JSON" <<EOF
{
  "profile": "${PROFILE}",
  "path": "${EXE}",
  "size_bytes": ${RAW_BYTES},
  "size_mib": ${RAW_MIB},
  "gzip_size_bytes": ${GZIP_BYTES},
  "upx_size_bytes": ${UPX_BYTES},
  "max_allowed_bytes": ${MAX_BYTES},
  "status": "${STATUS}",
  "timestamp_utc": "${TS}"
}
EOF
printf 'Raw size: %s bytes (%s MiB)\n' "$RAW_BYTES" "$RAW_MIB"
printf 'Gzip size: %s bytes\n' "$GZIP_BYTES"
if [[ "$UPX_BYTES" != "null" ]]; then printf 'UPX size: %s bytes\n' "$UPX_BYTES"; fi
printf 'Threshold: %s bytes\n' "$MAX_BYTES"
if [[ "$STATUS" = "pass" ]]; then
  echo "STATUS: within threshold (<=$MAX_BYTES)"
else
  echo "STATUS: EXCEEDED threshold (>$MAX_BYTES)" >&2
  if [[ -z "${NXSH_ALLOW_SIZE_FAILURE:-}" ]]; then
    exit 2
  else
    echo "Override active (NXSH_ALLOW_SIZE_FAILURE). Not failing." >&2
  fi
fi
