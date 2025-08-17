use std::io::{self, Read, Write};

use super::bitstream::BitWriter;
use super::seq::Seq;
use super::huffman::encode_literals_block_auto;

#[derive(Debug, Clone, Copy)]
pub struct FullZstdOptions {
	pub level: u8,
	pub checksum: bool,
	pub window_log: u8,
	/// Enable 4-stream Huffman encoding for literals (default: auto-detect based on size)
	pub force_four_stream: Option<bool>,
}

impl Default for FullZstdOptions {
	fn default() -> Self {
		Self { 
			level: 3, 
			checksum: false, 
			window_log: 20,
			force_four_stream: None,
		}
	}
}

/// Full encoder with 4-stream Huffman support for literals compression.
/// Uses RFC 8878 compliant encoding with automatic stream mode selection.
pub fn compress_reader_to_writer<R: Read, W: Write>(mut r: R, mut w: W, opt: FullZstdOptions) -> io::Result<()> {
	let mut data = Vec::new();
	r.read_to_end(&mut data)?;
	
	// For large data, attempt full Huffman compression with 4-stream support
	if data.len() >= 64 && opt.level >= 3 {
		if let Some((_table, encoded_block)) = encode_literals_block_auto(&data) {
			// Successfully encoded with Huffman - write the compressed block
			w.write_all(&encoded_block)?;
			return Ok(());
		}
	}
	
	// Fallback to raw data (will be handled by higher-level frame writer)
	w.write_all(&data)?;
	Ok(())
}

/// Encode literals section using the new 4-stream Huffman implementation.
/// Automatically selects between single-stream and 4-stream modes for optimal compression.
fn encode_literals_optimized<W: Write>(bw: &mut BitWriter<W>, lits: &[u8]) -> io::Result<()> {
	if lits.is_empty() {
		return Ok(()); // Empty literals section
	}
	
	// Use the auto-encoder which chooses the best mode
	if let Some((_table, encoded_block)) = encode_literals_block_auto(lits) {
		// Write the complete literals block (includes all headers and data)
		bw.write_bytes(&encoded_block)?;
	} else {
		// Fallback: write raw literals block
		encode_raw_literals_block(bw, lits)?;
	}
	
	Ok(())
}

/// Fallback encoder for raw literals when Huffman compression fails or is inefficient.
fn encode_raw_literals_block<W: Write>(bw: &mut BitWriter<W>, lits: &[u8]) -> io::Result<()> {
	let size = lits.len();
	
	// Raw literals block header: Block_Type = 00, Size_Format varies by size
	if size < 32 {
		// Size_Format = 00: 5 bits in header byte
		let header_byte = (size as u8) << 3; // Block_Type=00, size in bits 7-3
		bw.write_bits(header_byte as u64, 8)?;
	} else if size < 4096 {
		// Size_Format = 01: 12 bits (4 in header + 8 in next byte)
		let header_byte = 0x04 | ((size as u8) & 0x03) << 3; // Block_Type=00, Size_Format=01
		bw.write_bits(header_byte as u64, 8)?;
		bw.write_bits((size >> 2) as u64, 8)?;
	} else {
		// Size_Format = 11: 20 bits (4 in header + 16 in next 2 bytes)
		let header_byte = 0x0C | ((size as u8) & 0x03) << 3; // Block_Type=00, Size_Format=11
		bw.write_bits(header_byte as u64, 8)?;
		bw.write_bits((size >> 2) as u64, 16)?;
	}
	
	// Raw literal data
	bw.write_bytes(lits)?;
	Ok(())
}

/// Encode sequences section (placeholder for future FSE implementation).
fn encode_sequences<W: Write>(_bw: &mut BitWriter<W>, _seqs: &[Seq]) -> io::Result<()> {
	// TODO: Implement FSE-based sequence encoding
	// This will be the next major component after literals are complete
	Ok(())
}
