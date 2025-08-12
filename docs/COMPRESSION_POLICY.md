# Compression Policy (Pure Rust Only)

This project enforces a Pure Rust policy for compression/archiving:

- gzip: compress/decompress via flate2 (rust_backend)
- xz: compress/decompress via lzma-rs
- bzip2: decompress-only via bzip2-rs
- zstd: decompress-only via ruzstd

Rationale: avoid non-Rust FFI dependencies for portability and easier builds across platforms.
