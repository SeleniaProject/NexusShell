#!/usr/bin/env bash
set -euo pipefail

ROOT=${1:-/usr}

echo "Benchmark: recursive ls over $ROOT"

echo "== nxsh ls -R =="
/usr/bin/time -f '%E real, %U user, %S sys, %M KB maxrss' ./target/release/nxsh -c "ls -R $ROOT > /dev/null" || true

echo "== bash ls -R =="
/usr/bin/time -f '%E real, %U user, %S sys, %M KB maxrss' bash -lc "ls -R $ROOT > /dev/null" || true

echo "Done."

