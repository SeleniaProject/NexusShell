# Contributing to NexusShell

Thanks for your interest in contributing!

## Getting Started

1. Fork and clone the repo
2. Ensure a recent stable Rust toolchain (`rustup toolchain install stable`)
3. Build and test:
   - `cargo check --workspace --all-features`
   - `cargo test --workspace --all-features`
   - `cargo fmt --all -- --check`

## Pull Requests

- Use a small, focused scope per PR
- Update docs/README if behavior changes
- Ensure CI is green on all platforms

## Commit Messages

- Conventional style is appreciated (e.g., `feat:`, `fix:`, `docs:`)

## Security

If you believe you’ve found a security issue, please do not open a public issue.
Follow the guidelines in `SECURITY.md`.

## Code Style

- Follow `rustfmt` and `clippy` suggestions where reasonable
- Prefer explicit names and early returns; keep functions focused

## Internationalization (i18n)

- Keep user-facing strings translatable
- Update `.po` files when adding new messages

## Tests

- Add unit/integration tests for new functionality
- Keep tests deterministic and fast

## Release

- Tag with `vX.Y.Z` to trigger release workflow

Thank you for your contributions!


