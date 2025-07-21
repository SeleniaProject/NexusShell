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