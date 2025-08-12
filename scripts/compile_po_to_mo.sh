#!/usr/bin/env bash
set -euo pipefail

# Compile all i18n/po/<lang>/messages.po into i18n/mo/<lang>/LC_MESSAGES/messages.mo
ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
PO_DIR="$ROOT_DIR/i18n/po"
MO_DIR="$ROOT_DIR/i18n/mo"

if ! command -v msgfmt >/dev/null 2>&1; then
  echo "msgfmt not found. Please install gettext." >&2
  exit 1
fi

mkdir -p "$MO_DIR"

shopt -s nullglob
for po in "$PO_DIR"/*/messages.po; do
  lang=$(basename "$(dirname "$po")")
  outdir="$MO_DIR/$lang/LC_MESSAGES"
  mkdir -p "$outdir"
  echo "Compiling $po -> $outdir/messages.mo"
  msgfmt "$po" -o "$outdir/messages.mo"
done
echo "Done."

