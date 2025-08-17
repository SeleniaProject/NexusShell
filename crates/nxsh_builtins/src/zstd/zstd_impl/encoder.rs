use std::io::{self, Read, Write};

use super::bitstream::BitWriter;
use super::seq::Seq;

#[derive(Debug, Clone, Copy)]
pub struct FullZstdOptions {
	pub level: u8,
	pub checksum: bool,
	pub window_log: u8,
}

impl Default for FullZstdOptions {
	fn default() -> Self {
		Self { level: 3, checksum: false, window_log: 20 }
	}
}

/// Placeholder full encoder entry. Currently emits a single RAW block via higher layer.
/// Will be extended to output Compressed blocks with literals and sequences using Huffman/FSE.
pub fn compress_reader_to_writer<R: Read, W: Write>(mut r: R, mut w: W, _opt: FullZstdOptions) -> io::Result<()> {
	let mut data = Vec::new();
	r.read_to_end(&mut data)?;
	// For now, just passthrough to caller; higher-level frame writer decides block type.
	// Soon: build sequences, Huffman for literals, FSE for LL/ML/OF, pack into Compressed block.
	w.write_all(&data)?;
	Ok(())
}

// Sketches for upcoming pieces
fn _encode_literals<W: Write>(_bw: &mut BitWriter<W>, _lits: &[u8]) -> io::Result<()> {
	Ok(())
}
fn _encode_sequences<W: Write>(_bw: &mut BitWriter<W>, _seqs: &[Seq]) -> io::Result<()> {
	Ok(())
}
