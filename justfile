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

bench:
    cargo bench 

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

# Development convenience target for updating all generated files
update-generated: command-status busybox-size-report
    echo "Updated all generated files: COMMAND_STATUS.md and size reports"

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