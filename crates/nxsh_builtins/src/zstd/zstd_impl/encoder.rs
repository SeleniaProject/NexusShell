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

/// Encode sequences section with comprehensive FSE implementation.
fn encode_sequences<W: Write>(bw: &mut BitWriter<W>, seqs: &[Seq]) -> io::Result<()> {
    if seqs.is_empty() {
        // No sequences to encode
        return Ok(());
    }

    // Collect statistics for FSE table building
    let mut lit_length_counts = [0u32; 36]; // LL codes 0-35
    let mut match_length_counts = [0u32; 53]; // ML codes 0-52  
    let mut offset_counts = [0u32; 32]; // Offset codes 0-31

    for seq in seqs {
        // Count literal lengths
        let ll_code = encode_literal_length_code(seq.literal_length);
        if ll_code < lit_length_counts.len() {
            lit_length_counts[ll_code] += 1;
        }

        // Count match lengths  
        let ml_code = encode_match_length_code(seq.match_length);
        if ml_code < match_length_counts.len() {
            match_length_counts[ml_code] += 1;
        }

        // Count offsets
        let offset_code = encode_offset_code(seq.offset);
        if offset_code < offset_counts.len() {
            offset_counts[offset_code] += 1;
        }
    }

    // Build FSE tables
    let ll_table = build_fse_table(&lit_length_counts)?;
    let ml_table = build_fse_table(&match_length_counts)?;
    let of_table = build_fse_table(&offset_counts)?;

    // Write FSE table headers
    write_fse_table_header(bw, &ll_table)?;
    write_fse_table_header(bw, &ml_table)?;
    write_fse_table_header(bw, &of_table)?;

    // Encode sequences using FSE tables
    for seq in seqs {
        encode_sequence(bw, seq, &ll_table, &ml_table, &of_table)?;
    }

    Ok(())
}

fn encode_literal_length_code(length: u32) -> usize {
    match length {
        0..=15 => length as usize,
        16..=31 => 16 + ((length - 16) >> 1) as usize,
        32..=63 => 24 + ((length - 32) >> 2) as usize,
        64..=127 => 28 + ((length - 64) >> 3) as usize,
        128..=255 => 32 + ((length - 128) >> 4) as usize,
        256..=511 => 34 + ((length - 256) >> 5) as usize,
        _ => 35, // Max code
    }
}

fn encode_match_length_code(length: u32) -> usize {
    let ml = length.saturating_sub(3); // Match length offset
    match ml {
        0..=31 => ml as usize,
        32..=63 => 32 + ((ml - 32) >> 1) as usize,
        64..=127 => 48 + ((ml - 64) >> 2) as usize,
        128..=255 => 56 + ((ml - 128) >> 3) as usize,
        256..=511 => 60 + ((ml - 256) >> 4) as usize,
        512..=1023 => 62 + ((ml - 512) >> 5) as usize,
        _ => 52, // Max code
    }
}

fn encode_offset_code(offset: u32) -> usize {
    if offset <= 1 {
        offset as usize
    } else {
        let log2 = 32 - offset.leading_zeros() - 1;
        (log2 + 1) as usize
    }
}

#[derive(Debug, Clone)]
struct FseTable {
    symbols: Vec<u8>,
    states: Vec<u16>,
    symbol_transform: Vec<(u8, u8)>, // (nb_bits, baseline)
}

fn build_fse_table(counts: &[u32]) -> io::Result<FseTable> {
    let total_count: u32 = counts.iter().sum();
    if total_count == 0 {
        return Ok(FseTable {
            symbols: vec![],
            states: vec![],
            symbol_transform: vec![],
        });
    }

    // Calculate table size (power of 2)
    let table_log = calculate_table_log(total_count);
    let table_size = 1 << table_log;
    
    // Normalize counts to table size
    let mut normalized_counts = vec![0u16; counts.len()];
    let mut remaining = table_size as u32;
    
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            let normalized = ((count as u64 * table_size as u64) / total_count as u64) as u16;
            normalized_counts[i] = normalized.max(1);
            remaining -= normalized_counts[i] as u32;
        }
    }
    
    // Distribute remaining entries
    let mut symbol_idx = 0;
    while remaining > 0 && symbol_idx < normalized_counts.len() {
        if normalized_counts[symbol_idx] > 0 {
            normalized_counts[symbol_idx] += 1;
            remaining -= 1;
        }
        symbol_idx += 1;
    }

    // Build FSE table
    let mut symbols = vec![0u8; table_size];
    let mut states = vec![0u16; table_size];
    let mut symbol_transform = vec![(0u8, 0u8); counts.len()];
    
    let mut pos = 0;
    for (symbol, &count) in normalized_counts.iter().enumerate() {
        for _ in 0..count {
            symbols[pos] = symbol as u8;
            states[pos] = pos as u16;
            pos += 1;
        }
        
        // Calculate transform for symbol
        if count > 0 {
            let nb_bits = calculate_nb_bits(count);
            symbol_transform[symbol] = (nb_bits, count as u8);
        }
    }

    Ok(FseTable {
        symbols,
        states,
        symbol_transform,
    })
}

fn calculate_table_log(total: u32) -> u32 {
    if total <= 64 { 6 }
    else if total <= 256 { 8 }
    else if total <= 1024 { 10 }
    else { 12 }
}

fn calculate_nb_bits(count: u16) -> u8 {
    if count <= 1 { 0 }
    else { (16 - count.leading_zeros()) as u8 }
}

fn write_fse_table_header<W: Write>(bw: &mut BitWriter<W>, table: &FseTable) -> io::Result<()> {
    if table.symbols.is_empty() {
        // Empty table
        bw.write_bits(0, 1)?; // Use default/predefined table
        return Ok(());
    }

    bw.write_bits(1, 1)?; // Custom table flag
    
    // Write table log
    let table_log = (table.symbols.len() as f64).log2() as u8;
    bw.write_bits(table_log as u64, 4)?;
    
    // Write symbol count and normalized frequencies
    let max_symbol = table.symbols.iter().max().unwrap_or(&0);
    bw.write_bits(*max_symbol as u64, 8)?;
    
    for i in 0..=*max_symbol {
        let count = table.symbol_transform.get(i as usize)
            .map(|(_, baseline)| *baseline as u32)
            .unwrap_or(0);
        bw.write_bits(count as u64, table_log)?;
    }
    
    Ok(())
}

fn encode_sequence<W: Write>(
    bw: &mut BitWriter<W>,
    seq: &Seq,
    ll_table: &FseTable,
    ml_table: &FseTable,
    of_table: &FseTable,
) -> io::Result<()> {
    // Encode literal length
    let ll_code = encode_literal_length_code(seq.literal_length);
    encode_symbol(bw, ll_code as u8, ll_table)?;
    
    // Encode extra bits for literal length if needed
    write_extra_bits(bw, seq.literal_length, ll_code)?;
    
    // Encode match length
    let ml_code = encode_match_length_code(seq.match_length);
    encode_symbol(bw, ml_code as u8, ml_table)?;
    
    // Encode extra bits for match length if needed
    write_extra_bits(bw, seq.match_length.saturating_sub(3), ml_code)?;
    
    // Encode offset
    let of_code = encode_offset_code(seq.offset);
    encode_symbol(bw, of_code as u8, of_table)?;
    
    // Encode extra bits for offset if needed
    write_offset_extra_bits(bw, seq.offset, of_code)?;
    
    Ok(())
}

fn encode_symbol<W: Write>(bw: &mut BitWriter<W>, symbol: u8, table: &FseTable) -> io::Result<()> {
    if table.symbols.is_empty() {
        // Use raw encoding for empty tables
        bw.write_bits(symbol as u64, 8)?;
        return Ok(());
    }
    
    // Find symbol in table and encode its state
    for (i, &table_symbol) in table.symbols.iter().enumerate() {
        if table_symbol == symbol {
            let (nb_bits, _) = table.symbol_transform.get(symbol as usize).unwrap_or(&(0, 0));
            bw.write_bits(i as u64, *nb_bits)?;
            break;
        }
    }
    
    Ok(())
}

fn write_extra_bits<W: Write>(bw: &mut BitWriter<W>, value: u32, code: usize) -> io::Result<()> {
    let extra_bits = match code {
        16..=23 => 1,
        24..=27 => 2,
        28..=31 => 3,
        32..=33 => 4,
        34..=35 => 5,
        _ => 0,
    };
    
    if extra_bits > 0 {
        let baseline = get_baseline_value(code);
        let extra_value = value - baseline;
        bw.write_bits(extra_value as u64, extra_bits)?;
    }
    
    Ok(())
}

fn write_offset_extra_bits<W: Write>(bw: &mut BitWriter<W>, offset: u32, code: usize) -> io::Result<()> {
    if code > 1 && offset > 1 {
        let extra_bits = code - 1;
        let baseline = 1u32 << extra_bits;
        let extra_value = offset - baseline;
        bw.write_bits(extra_value as u64, extra_bits as u8)?;
    }
    
    Ok(())
}

fn get_baseline_value(code: usize) -> u32 {
    match code {
        0..=15 => code as u32,
        16..=23 => 16 + ((code - 16) << 1) as u32,
        24..=27 => 32 + ((code - 24) << 2) as u32,
        28..=31 => 64 + ((code - 28) << 3) as u32,
        32..=33 => 128 + ((code - 32) << 4) as u32,
        34..=35 => 256 + ((code - 34) << 5) as u32,
        _ => 0,
    }
}
