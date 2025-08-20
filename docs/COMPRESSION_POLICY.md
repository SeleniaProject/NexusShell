# Compression Policy (Pure Rust Only)

This project enforces a Pure Rust policy for compression/archiving:

- gzip: compress/decompress via flate2 (rust_backend)
- xz: compress/decompress via lzma-rs
- bzip2: decompress-only via bzip2-rs
- zstd: decompress via ruzstd; compress via Pure Rust store-mode (RAW block frame)

Rationale: avoid non-Rust FFI dependencies for portability and easier builds across platforms. For zstd,
we implement a standards-compliant "store-mode" encoder that writes a valid frame with RAW blocks only
(no entropy compression). This guarantees round-trip functionality without external binaries while keeping
the codebase Pure Rust. When a high-ratio Pure Rust encoder becomes viable, we can upgrade transparently.
