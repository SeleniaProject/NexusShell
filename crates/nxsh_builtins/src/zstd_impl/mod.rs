//! Internal Pure-Rust Zstandard encoder implementation (work-in-progress).
//! Public surface mirrors what the higher-level writer expects, with
//! minimal, spec-compliant stubs for areas not yet fully implemented.

pub mod bitstream;
pub mod encoder;
pub mod seq;

// Lightweight, public stubs used by higher layers (to be replaced with full impl)
pub mod fse;
pub mod huffman;
pub mod seq_write;
