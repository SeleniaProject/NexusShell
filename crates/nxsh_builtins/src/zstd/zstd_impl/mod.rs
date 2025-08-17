//! Internal Pure-Rust Zstandard encoder implementation (work-in-progress).
//! This module will host the full RFC 8878-compliant encoder: LZ77 + Huffman + FSE + Dict.

pub mod bitstream;
pub mod lz77;
pub mod seq;
pub mod encoder;
pub mod huffman;
pub mod fse;

// Re-export primary entry points for integration
pub use encoder::{FullZstdOptions, compress_reader_to_writer};
