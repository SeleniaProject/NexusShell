//! Internal Pure-Rust Zstandard encoder implementation (work-in-progress).
//! This is a shim that points to the real implementation under `src/zstd/zstd_impl/`.

// Re-export the entire module tree from the real path so that callers can use
// `zstd_impl::{huffman, fse, bitstream, lz77, seq, encoder, ...}` uniformly.
#[path = "zstd/zstd_impl/mod.rs"]
mod real;
pub use real::*;
