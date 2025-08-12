#!/usr/bin/env bash
set -euo pipefail

TARGET_DIR=${1:-.}
PATTERN=${2:-TODO}

echo "Benchmark: grep -r ${PATTERN} ${TARGET_DIR} vs ripgrep"

command -v rg >/dev/null 2>&1 || { echo "ripgrep not installed"; exit 1; }

echo "== ripgrep =="
/usr/bin/time -f '%E real, %U user, %S sys, %M KB maxrss' rg -n "$PATTERN" "$TARGET_DIR" >/dev/null || true

echo "== grep =="
/usr/bin/time -f '%E real, %U user, %S sys, %M KB maxrss' grep -R -n "$PATTERN" "$TARGET_DIR" >/dev/null || true

echo "Done."

