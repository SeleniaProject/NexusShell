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

# Run benches and enforce JIT/MIR 2x speedup using Criterion outputs
bench-gate:
    cargo bench -p nxsh_core --bench jit_vs_interp
    python scripts/check_jit_speedup.py --target-dir target/criterion --bench-group jit_vs_interp --interp-name interp_execute --jit-name mir_execute --required-speedup 2.0

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
    powershell -NoLogo -ExecutionPolicy Bypass -File scripts/size_report.ps1

# Enhanced size report (Bash for *nix)
busybox-size-report-sh:
    bash scripts/size_report.sh

# Enforce size threshold (fails if exceeded) - uses NXSH_SIZE_MAX if set
busybox-size-gate:
    powershell -NoLogo -ExecutionPolicy Bypass -File scripts/size_report.ps1; if($LastExitCode -ne 0){ exit $LastExitCode }

# Cargo bloat (requires cargo-bloat installed) for busybox-min
busybox-bloat:
    cargo bloat -p nxsh_cli --release --features busybox-min --no-default-features -n 40

# Generate command status and fail on diff (exit 4 like script) for CI
command-status-check:
    # Compile and run the standalone generator (no Cargo package). Output diff causes non-zero exit.
    if (-not (Test-Path target)){ mkdir target | Out-Null }
    rustc --edition 2021 scripts/gen_command_status.rs -o target/gen_command_status.exe
    ./target/gen_command_status.exe

# Generate command status report (always succeeds, updates COMMAND_STATUS.md)
command-status:
    # Generate updated command status without failing on changes
    if (-not (Test-Path target)){ mkdir target | Out-Null }
    rustc --edition 2021 scripts/gen_command_status.rs -o target/gen_command_status.exe
    ./target/gen_command_status.exe --update

# Full CI pipeline including command status verification and size checks
full-ci: ci command-status-check busybox-size-gate
    echo "All CI checks passed successfully"

# Full CI with performance gate
full-ci-with-bench: ci command-status-check busybox-size-gate bench-gate
    echo "All CI checks + bench gate passed successfully"

# Development convenience target for updating all generated files
update-generated: command-status busybox-size-report
    echo "Updated all generated files: COMMAND_STATUS.md and size reports"
    # Also refresh theme validation report (Markdown)
    if (-not (Test-Path reports)) { mkdir reports | Out-Null }
    cargo run -p nxsh_ui --bin theme_validator_test -- --dir assets/themes --out-format md --out reports/theme_validation.md

# Docs/spec consistency quick check (alias of command-status-check)
docs-check: command-status-check
    echo "Docs/spec command catalogs are consistent"

# Show project statistics (commands implemented, binary size, dependencies)
stats:
    echo "=== NexusShell Project Statistics ==="
    echo ""
    echo "Command Implementation Status:"
    powershell -c "if (Test-Path 'COMMAND_STATUS.md') { (Get-Content 'COMMAND_STATUS.md' | Select-String '✅|⚠|💤|🔍').Count } else { 'COMMAND_STATUS.md not found' }"
    echo ""
    echo "Binary Sizes:"
    just busybox-size-report
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