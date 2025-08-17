# justfile for NexusShell

default := "build"

build:
    cargo build --workspace --release

test:
    cargo test --workspace

ci:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt -- --check
    cargo test --workspace
    # Validate themes and emit JSON report artifact
    if (-not (Test-Path reports)) { mkdir reports | Out-Null }
    cargo run -p nxsh_ui --bin theme_validator_test -- --dir assets/themes --out-format json --out reports/theme_validation.json

bench:
    cargo bench 

# Simple benches
bench-gate:
    # scripts removed; keep as alias to run the main bench only
    cargo bench -p nxsh_core --bench jit_vs_interp

miri:
    cargo +nightly miri test --workspace 

# Build minimal BusyBox-style binary (no UI) with size-oriented profile
busybox-build:
    cargo build -p nxsh_cli --no-default-features --features busybox-min --profile release-small

# Measure size (after strip); on Windows .exe already stripped due to profile
busybox-size: busybox-build
    # Print file size in bytes and human readable
    $size = (Get-Item target/release-small/nxsh.exe).Length; echo "nxsh.exe size(bytes)=$size"; $mb=[math]::Round($size/1MB,3); echo "size(MiB)=$mb" 

# Enhanced size report (PowerShell on Windows)
busybox-size-report:
    # scripts removed; show size with PowerShell inline
    $bin = "target/release-small/nxsh.exe"; if (!(Test-Path $bin)) { just busybox-build }
    $size = (Get-Item $bin).Length; echo "nxsh.exe size(bytes)=$size"; $mb=[math]::Round($size/1MB,3); echo "size(MiB)=$mb"; '{"size_bytes":'+$size+'}' | Set-Content -Encoding UTF8 size_report.json

# Enhanced size report (Bash for *nix)
busybox-size-report-sh:
    # scripts removed; compute size inline
    BIN=target/release-small/nxsh; [ -f "$BIN" ] || (just busybox-build)
    SIZE=$(stat -c%s "$BIN" 2>/dev/null || wc -c < "$BIN"); echo "nxsh size(bytes)=$SIZE"; echo "{\"size_bytes\":$SIZE}" > size_report.json

# Enforce size threshold (fails if exceeded) - uses NXSH_SIZE_MAX if set
busybox-size-gate:
    # scripts removed; enforce threshold inline
    $env:NXSH_SIZE_MAX = if ($env:NXSH_SIZE_MAX) { $env:NXSH_SIZE_MAX } else { "1572864" }
    $bin = "target/release-small/nxsh.exe"; if (!(Test-Path $bin)) { just busybox-build }
    $size = (Get-Item $bin).Length; if ($size -gt [int]$env:NXSH_SIZE_MAX) { echo "Size gate failed: $size > $env:NXSH_SIZE_MAX"; exit 2 } else { echo "Size gate OK ($size bytes)" }

# Cargo bloat (requires cargo-bloat installed) for busybox-min
busybox-bloat:
    cargo bloat -p nxsh_cli --release --features busybox-min --no-default-features -n 40

## command-status tasks removed (scripts deleted)

# Full CI pipeline including command status verification and size checks
full-ci: ci busybox-size-gate
    echo "All CI checks passed successfully"

# Full CI with performance gate
full-ci-with-bench: ci command-status-check busybox-size-gate bench-gate
    echo "All CI checks + bench gate passed successfully"

# Development convenience target for updating all generated files
update-generated: busybox-size-report
    echo "Updated size reports"
    # Also refresh theme validation report (Markdown)
    if (-not (Test-Path reports)) { mkdir reports | Out-Null }
    cargo run -p nxsh_ui --bin theme_validator_test -- --dir assets/themes --out-format md --out reports/theme_validation.md

# Docs/spec consistency quick check (alias of command-status-check)
## docs-check removed (command-status-check no longer available)

# Show project statistics (commands implemented, binary size, dependencies)
stats:
    echo "=== NexusShell Project Statistics ==="
    echo ""
    echo "Command Implementation Status:"
    powershell -c "if (Test-Path 'COMMAND_STATUS.md') { (Get-Content 'COMMAND_STATUS.md' | Select-String '✅|⚠|💤|🔍').Count } else { 'COMMAND_STATUS.md not found' }"
    echo ""
    echo "Binary Sizes:"
    just busybox-size
    echo ""
    echo "Dependency Count:"
    cargo tree --workspace | wc -l

# Validate all themes and write a Markdown report
themes-validate:
    if (-not (Test-Path reports)) { mkdir reports | Out-Null }
    cargo run -p nxsh_ui --bin theme_validator_test -- --dir assets/themes --out-format md --out reports/theme_validation.md --strict

# Validate themes and emit JSON summary for CI consumption
themes-validate-json:
    if (-not (Test-Path reports)) { mkdir reports | Out-Null }
    cargo run -p nxsh_ui --bin theme_validator_test -- --dir assets/themes --out-format json --out reports/theme_validation.json

# Render all mockup PNGs from ANSI using nxsh_ui batch tool
mockups-png:
    # Ensure font exists
    if (-not (Test-Path assets/fonts/JetBrainsMono-Regular.ttf)) { echo "Place a monospace font at assets/fonts/JetBrainsMono-Regular.ttf"; exit 1 }
    # Create default config if missing
    if (-not (Test-Path assets/mockups/generate_pngs.json)) {
      $cfg = '{"font":"assets/fonts/JetBrainsMono-Regular.ttf","size":18,"bg":"#282828","cols":100,"line_height":1.2,"inputs":[{"in":"assets/mockups/nxsh_splash.ans","out":"assets/mockups/nxsh_splash.png"},{"in":"assets/mockups/nxsh_prompt_status.ans","out":"assets/mockups/nxsh_prompt_status.png"},{"in":"assets/mockups/nxsh_table_sample.ans","out":"assets/mockups/nxsh_table_sample.png"},{"in":"assets/mockups/nxsh_git_panel.ans","out":"assets/mockups/nxsh_git_panel.png"},{"in":"assets/mockups/nxsh_completion_panel.ans","out":"assets/mockups/nxsh_completion_panel.png"},{"in":"assets/mockups/nxsh_progress_view.ans","out":"assets/mockups/nxsh_progress_view.png"},{"in":"assets/mockups/nxsh_error_view.ans","out":"assets/mockups/nxsh_error_view.png"},{"in":"assets/mockups/nxsh_output_scroll.ans","out":"assets/mockups/nxsh_output_scroll.png"}]}'
      if (-not (Test-Path assets/mockups)) { mkdir assets/mockups | Out-Null }
      Set-Content -Path assets/mockups/generate_pngs.json -Value $cfg -Encoding UTF8
    }
    cargo run -p nxsh_ui --bin ansi_to_png_batch -- --config assets/mockups/generate_pngs.json