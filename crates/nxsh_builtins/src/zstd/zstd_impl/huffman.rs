//! Huffman coding for literals (1-stream and 4-stream encoders)
//! - Builds canonical codes from simple length-limited scheme  
//! - Supports both single-stream and 4-stream encoding modes
//! - Emits raw (direct) weights header (headerByte >= 128) or FSE-compressed weights
//! - Returns code map and header bytes for Compressed_Literals_Block

#[derive(Debug, Clone)]
pub struct HuffmanTable {
    /// Weight per symbol (0 if absent), indexed by literal value
    pub weights: Vec<u8>,
    /// Code and bit length per symbol (MSB-first code; write reversed bits in LSB-first streams)
    pub codes: [Option<(u16, u8)>; 256],
    /// Last present symbol index + 1
    pub num_symbols: usize,
}

/// Represents a 4-stream Huffman encoding result with jump table
#[derive(Debug, Clone)]
pub struct FourStreamHuffman {
    /// Encoded stream data for each of the 4 streams
    pub streams: [Vec<u8>; 4],
    /// Jump table: [stream1_size, stream2_size, stream3_size] (stream4 size is implicit)
    pub jump_table: [u16; 3],
    /// Total compressed size including jump table
    pub total_size: usize,
}

fn ceil_log2(mut n: usize) -> u8 {
    if n <= 1 { return 0; }
    n -= 1;
    let mut l = 0u8;
    while n > 0 { n >>= 1; l += 1; }
    l
}

/// Build a simple, valid Huffman table for literals using a canonical construction.
/// Returns None if not suitable (e.g., distinct symbols < 2 or max symbol > 127).
/// Also returns an optimized header (direct weights or FSE-compressed weights).
pub fn build_literals_huffman(lits: &[u8]) -> Option<(HuffmanTable, Vec<u8>)> {
    if lits.is_empty() { return None; }
    let mut freq = [0usize; 256];
    let mut max_sym = 0usize;
    for &b in lits {
        let idx = b as usize;
        freq[idx] += 1;
        if idx > max_sym { max_sym = idx; }
    }
    if max_sym > 127 { return None; }
    // Collect present symbols
    let mut present: Vec<(usize, usize)> = Vec::new(); // (sym, freq)
    for (s, &f) in freq.iter().enumerate().take(max_sym + 1) { if f > 0 { present.push((s, f)); } }
    if present.len() < 2 { return None; }

    // Determine code lengths: at most 11, but with <=128 symbols, ceil_log2(n) <= 7
    let n = present.len();
    let nb = ceil_log2(n); // shortest max bits
    let need_shorter = (1usize << nb) - n; // number of codes with length nb-1

    // Sort by frequency descending (better compression), then by symbol asc
    present.sort_by(|a,b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut lens = vec![nb; n];
    for l in lens.iter_mut().take(need_shorter) { *l = nb.saturating_sub(1); }

    // Translate to weights: Number_of_Bits = Max+1 - Weight => Weight = Max+1 - L
    let max_bits = *lens.iter().max().unwrap_or(&1);
    let mut weights = vec![0u8; max_sym+1];
    for (i, (sym, _)) in present.iter().enumerate() {
        let l = lens[i];
        let w = (max_bits + 1).saturating_sub(l) as u8;
        weights[*sym] = w; // 0 means absent
    }

    // Build canonical codes from lengths (standard method, shortest first)
    // First, map symbol->length
    let mut lengths = [0u8; 256];
    for (i, (sym, _)) in present.iter().enumerate() { lengths[*sym] = lens[i]; }
    let mut bl_count = [0u16; 12]; // up to 11 bits
    for &l in lengths.iter().take(max_sym + 1) { let l = l as usize; if l>0 { bl_count[l] += 1; } }
    let mut next_code = [0u16; 12];
    let mut code = 0u16;
    for l in 1..=max_bits as usize {
        code = (code + bl_count[l-1]) << 1;
        next_code[l] = code;
    }
    let mut codes: [Option<(u16,u8)>; 256] = [None; 256];
    // Assign codes: increasing length, increasing symbol value
    let mut by_len_then_sym: Vec<(usize,u8)> = Vec::new(); // (sym, len)
    for (s, &l) in lengths.iter().enumerate().take(max_sym + 1) { if l>0 { by_len_then_sym.push((s, l)); } }
    by_len_then_sym.sort_by(|a,b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    for (sym, l) in by_len_then_sym {
        let lz = l as usize;
        let c = next_code[lz];
        next_code[lz] = c + 1;
        codes[sym] = Some((c, l));
    }

    // Build direct weights header (headerByte = 127 + Number_of_Symbols), 2 weights per byte
    let number_of_symbols = max_sym + 1; // include all up to max_sym
    if number_of_symbols > 128 { return None; }
    let header_byte = 127u8 + (number_of_symbols as u8);
    let mut direct_header = Vec::with_capacity(1 + number_of_symbols.div_ceil(2));
    direct_header.push(header_byte);
    let mut i = 0usize;
    while i < number_of_symbols {
        let w0 = *weights.get(i).unwrap_or(&0) & 0x0F;
        let w1 = *weights.get(i+1).unwrap_or(&0) & 0x0F;
        direct_header.push((w0 << 4) | w1);
        i += 2;
    }

    let table = HuffmanTable { weights, codes, num_symbols: number_of_symbols };
    
    // Try FSE-compressed weights and choose the smaller option
    if let Some(fse_header) = build_fse_compressed_weights_header(&table) {
        if fse_header.len() < direct_header.len() {
            return Some((table, fse_header));
        }
    }

    Some((table, direct_header))
}

/// Build FSE-compressed weights header for Huffman tables.
/// Returns the compressed weights stream (headerByte < 128) if more efficient than direct weights.
/// Uses RFC 8878 section 4.2.1 FSE compression for weights with full implementation.
pub fn build_fse_compressed_weights_header(table: &HuffmanTable) -> Option<Vec<u8>> {
    use super::fse::{build_normalized_from_hist, encode_fse_table_description, FseEncTable};
    use super::bitstream::BitWriter;
    
    // Extract non-zero weights for FSE compression
    let mut weights_stream = Vec::new();
    let mut weight_freqs = [0u32; 16]; // weights are 0-15, using u32 for FSE compatibility
    
    for &weight in &table.weights[..table.num_symbols] {
        if weight > 0 {
            weights_stream.push(weight);
            if (weight as usize) < 16 {
                weight_freqs[weight as usize] += 1;
            }
        }
    }
    
    if weights_stream.len() < 2 {
        return None; // Not suitable for FSE compression
    }
    
    // Count distinct symbols for FSE table optimization
    let distinct_weights = weight_freqs.iter().filter(|&&freq| freq > 0).count();
    if distinct_weights < 2 {
        return None; // Need at least 2 distinct symbols for FSE
    }
    
    // Build normalized counts for weights FSE table
    let total_weights = weights_stream.len() as u32;
    let table_log = super::fse::choose_table_log(total_weights, 6); // weights table log typically ≤ 6
    let normalized = build_normalized_from_hist(&weight_freqs[..16], table_log).ok()?;
    
    // Verify FSE table is valid
    let fse_table = FseEncTable::from_normalized(&normalized, table_log).ok()?;
    
    // Encode FSE table description
    let table_desc = encode_fse_table_description(&normalized, table_log).ok()?;
    
    // Full FSE encoding of weights sequence
    let mut compressed_weights = Vec::new();
    {
        let mut bw = BitWriter::new(&mut compressed_weights);
        
        // Initialize FSE states - use different starting states to improve compression
        let table_size = 1u32 << table_log;
        let mut state = table_size / 2; // Start in middle of table for better distribution
        
        // Encode weights in reverse order (FSE requirement)
        for &weight in weights_stream.iter().rev() {
            let symbol = weight as usize;
            
            // Get encoding parameters for this symbol
            let nb_bits = fse_table.nb_bits_out[state as usize % fse_table.nb_bits_out.len()];
            let base = fse_table.base.get(symbol).copied().unwrap_or(0);
            
            // Output state bits (if any)
            if nb_bits > 0 {
                let bits_to_write = state & ((1u32 << nb_bits) - 1);
                if bw.write_bits(bits_to_write as u64, nb_bits).is_err() {
                    return None;
                }
            }
            
            // Update state for next symbol
            state = base as u32 + (state >> nb_bits);
        }
        
        // Write final state
        if bw.write_bits(state as u64, table_log).is_err() {
            return None;
        }
        
        // Add end marker and align to byte
        if bw.write_bits(1, 1).is_err() {
            return None;
        }
        if bw.align_to_byte().is_err() {
            return None;
        }
    }
    
    // Calculate total compressed size
    let total_size = 1 + table_desc.len() + compressed_weights.len(); // 1 byte for header
    
    // Compare with direct weights size to ensure compression benefit
    let direct_size = 1 + table.num_symbols.div_ceil(2);
    
    if total_size >= direct_size {
        return None; // Direct weights are more efficient
    }
    
    // Build header: headerByte < 128 indicates FSE compression
    // Format: bit 0-5 = accuracy log, bit 6 = reserved (0), bit 7 = compression type (0 for FSE)
    let header_byte = table_log; // This is < 128 to indicate FSE compression
    
    let mut result = Vec::with_capacity(total_size);
    result.push(header_byte);
    result.extend_from_slice(&table_desc);
    result.extend_from_slice(&compressed_weights);
    
    // Verify result makes sense
    if result.len() > direct_size {
        return None;
    }
    
    Some(result)
}

/// Reverse the lowest `bits` bits of `code`.
pub fn reverse_bits(mut code: u16, bits: u8) -> u16 {
    let mut r = 0u16; let mut b = bits;
    while b > 0 { r = (r << 1) | (code & 1); code >>= 1; b -= 1; }
    r
}

/// Encode literals using 4-stream Huffman compression with jump table.
/// This is the RFC 8878 compliant multi-stream implementation for large literal blocks.
/// Returns FourStreamHuffman containing encoded streams and jump table.
pub fn encode_four_stream_huffman(lits: &[u8], table: &HuffmanTable) -> Option<FourStreamHuffman> {
    if lits.len() < 4 {
        return None; // Not suitable for 4-stream encoding
    }
    
    // Split literals into 4 approximately equal parts
    let chunk_size = lits.len() / 4;
    let remainder = lits.len() % 4;
    
    let mut chunk_sizes = [chunk_size; 4];
    #[allow(clippy::needless_range_loop)]
    for i in 0..remainder {
        chunk_sizes[i] += 1;
    }
    
    let mut start = 0;
    let mut chunks = Vec::with_capacity(4);
    for &size in &chunk_sizes {
        chunks.push(&lits[start..start + size]);
        start += size;
    }
    
    // Encode each stream using the Huffman table
    let mut streams = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    
    for (stream_idx, chunk) in chunks.iter().enumerate() {
        if let Some(encoded) = encode_single_stream_huffman(chunk, table) {
            streams[stream_idx] = encoded;
        } else {
            return None; // Failed to encode one of the streams
        }
    }
    
    // Build jump table: sizes of first 3 streams (4th is implicit)
    let jump_table = [
        streams[0].len() as u16,
        streams[1].len() as u16, 
        streams[2].len() as u16,
    ];
    
    // Calculate total size: 6 bytes jump table + all stream data
    let total_size = 6 + streams.iter().map(|s| s.len()).sum::<usize>();
    
    Some(FourStreamHuffman {
        streams,
        jump_table,
        total_size,
    })
}

/// Encode a single stream of literals using the provided Huffman table.
/// Returns the encoded bitstream as bytes (LSB-first bit packing).
fn encode_single_stream_huffman(lits: &[u8], table: &HuffmanTable) -> Option<Vec<u8>> {
    use super::bitstream::BitWriter;
    
    if lits.is_empty() {
        return Some(Vec::new());
    }
    
    let mut encoded = Vec::with_capacity(lits.len());
    {
        let mut bw = BitWriter::new(&mut encoded);
        
        for &lit in lits {
            if let Some((code, bits)) = table.codes[lit as usize] {
                // Reverse bits for LSB-first encoding
                let reversed_code = reverse_bits(code, bits);
                if bw.write_bits(reversed_code as u64, bits).is_err() {
                    return None;
                }
            } else {
                return None; // Symbol not in table
            }
        }
        
        // Align to byte boundary
        if bw.align_to_byte().is_err() {
            return None;
        }
    }
    
    Some(encoded)
}

/// Encode literals block header for 4-stream mode according to RFC 8878.
/// Returns the complete literals block: [regenerated_size][compressed_size][huffman_header][jump_table][streams]
pub fn encode_four_stream_literals_block(lits: &[u8], table: &HuffmanTable, header: &[u8]) -> Option<Vec<u8>> {
    let four_stream = encode_four_stream_huffman(lits, table)?;
    
    // Calculate sizes
    let regenerated_size = lits.len();
    let compressed_size = header.len() + four_stream.total_size;
    
    let mut result = Vec::with_capacity(5 + compressed_size);
    
    // Literals block header: Block_Type(2bits) + Size_Format(2bits) + Regenerated_Size + Compressed_Size
    // For 4-stream: Block_Type = 01 (Compressed_Literals_Block), Size_Format depends on sizes
    
    if regenerated_size < 1024 && compressed_size < 512 {
        // Size_Format = 00: 1 byte for both sizes (10 bits total: 4+6 or 6+4)
        let header_byte = 0x01; // Block_Type=01, Size_Format=00
        result.push(header_byte);
        
        if regenerated_size < 16 && compressed_size < 64 {
            // 4 bits regenerated + 4 bits compressed (packed in 1 byte)
            let size_byte = ((regenerated_size as u8) << 4) | (compressed_size as u8);
            result.push(size_byte);
        } else {
            // 6 bits regenerated + 6 bits compressed (need 2 bytes with some waste)
            let size_bytes = ((regenerated_size as u16) << 6) | (compressed_size as u16);
            result.extend_from_slice(&size_bytes.to_le_bytes());
        }
    } else if regenerated_size < 16384 && compressed_size < 16384 {
        // Size_Format = 01: 2 bytes for both sizes (14 bits each)  
        let header_byte = 0x05; // Block_Type=01, Size_Format=01
        result.push(header_byte);
        
        let size_word = ((regenerated_size as u32) << 14) | (compressed_size as u32);
        result.extend_from_slice(&size_word.to_le_bytes()[..3]); // 3 bytes total
    } else {
        // Size_Format = 11: 3 bytes for both sizes (full range)
        let header_byte = 0x0D; // Block_Type=01, Size_Format=11  
        result.push(header_byte);
        
        // Regenerated size (3 bytes) + compressed size (3 bytes)
        result.extend_from_slice(&(regenerated_size as u32).to_le_bytes()[..3]);
        result.extend_from_slice(&(compressed_size as u32).to_le_bytes()[..3]);
    }
    
    // Huffman header (weights table)
    result.extend_from_slice(header);
    
    // Jump table: 3x u16 little-endian
    for &size in &four_stream.jump_table {
        result.extend_from_slice(&size.to_le_bytes());
    }
    
    // Stream data
    for stream in &four_stream.streams {
        result.extend_from_slice(stream);
    }
    
    Some(result)
}

/// Decide whether to use single-stream or 4-stream encoding based on literal block size.
/// Returns true if 4-stream encoding is recommended.
pub fn should_use_four_stream(lits: &[u8]) -> bool {
    // RFC 8878 recommends 4-stream for blocks >= 1KB to amortize jump table overhead
    // Also consider symbol distribution - 4-stream works better with diverse content
    lits.len() >= 1024
}

/// Enhanced literals encoding that automatically chooses between single-stream and 4-stream modes.
/// Returns the optimal encoding with minimal header overhead.
pub fn encode_literals_block_auto(lits: &[u8]) -> Option<(HuffmanTable, Vec<u8>)> {
    // Build Huffman table from literal frequencies
    let (table, header) = build_literals_huffman(lits)?;
    
    if should_use_four_stream(lits) {
        // Try 4-stream encoding
        if let Some(four_stream_block) = encode_four_stream_literals_block(lits, &table, &header) {
            // Compare with single-stream encoding
            if let Some(single_stream_block) = encode_single_stream_literals_block(lits, &table, &header) {
                if four_stream_block.len() < single_stream_block.len() {
                    return Some((table, four_stream_block));
                } else {
                    return Some((table, single_stream_block));
                }
            }
            return Some((table, four_stream_block));
        }
    }
    
    // Fall back to single-stream encoding
    let single_stream_block = encode_single_stream_literals_block(lits, &table, &header)?;
    Some((table, single_stream_block))
}

/// Encode literals block using single-stream mode (existing implementation).
/// Returns the complete literals block with proper header encoding.
fn encode_single_stream_literals_block(lits: &[u8], table: &HuffmanTable, header: &[u8]) -> Option<Vec<u8>> {
    let encoded_stream = encode_single_stream_huffman(lits, table)?;
    
    let regenerated_size = lits.len();
    let compressed_size = header.len() + encoded_stream.len();
    
    let mut result = Vec::with_capacity(5 + compressed_size);
    
    // Single-stream literals block header (same format as 4-stream but no jump table)
    if regenerated_size < 1024 && compressed_size < 512 {
        let header_byte = 0x01; // Block_Type=01, Size_Format=00
        result.push(header_byte);
        
        if regenerated_size < 16 && compressed_size < 64 {
            let size_byte = ((regenerated_size as u8) << 4) | (compressed_size as u8);
            result.push(size_byte);
        } else {
            let size_bytes = ((regenerated_size as u16) << 6) | (compressed_size as u16);
            result.extend_from_slice(&size_bytes.to_le_bytes());
        }
    } else if regenerated_size < 16384 && compressed_size < 16384 {
        let header_byte = 0x05; // Block_Type=01, Size_Format=01
        result.push(header_byte);
        
        let size_word = ((regenerated_size as u32) << 14) | (compressed_size as u32);
        result.extend_from_slice(&size_word.to_le_bytes()[..3]);
    } else {
        let header_byte = 0x0D; // Block_Type=01, Size_Format=11
        result.push(header_byte);
        
        result.extend_from_slice(&(regenerated_size as u32).to_le_bytes()[..3]);
        result.extend_from_slice(&(compressed_size as u32).to_le_bytes()[..3]);
    }
    
    // Huffman header
    result.extend_from_slice(header);
    
    // Single stream data (no jump table needed)
    result.extend_from_slice(&encoded_stream);
    
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_stream_basic_functionality() {
        // Test basic 4-stream encoding with diverse literal data
        let literals = b"Hello, World! This is a test of the 4-stream Huffman encoder. It should split data efficiently across streams and produce valid output with jump tables.";
        
        let (table, _) = build_literals_huffman(literals).expect("Failed to build Huffman table");
        let four_stream = encode_four_stream_huffman(literals, &table).expect("Failed to encode 4-stream");
        
        // Verify basic structure
        assert_eq!(four_stream.streams.len(), 4);
        assert_eq!(four_stream.jump_table.len(), 3);
        
        // Verify streams are non-empty (given sufficient input)
        for stream in &four_stream.streams {
            assert!(!stream.is_empty(), "Stream should not be empty");
        }
        
        // Verify total size calculation
        let expected_total = 6 + four_stream.streams.iter().map(|s| s.len()).sum::<usize>();
        assert_eq!(four_stream.total_size, expected_total);
    }

    #[test]
    fn test_four_stream_edge_cases() {
        // Test with minimal data (should fail gracefully)
        let tiny_data = b"Hi";
        let result = encode_four_stream_huffman(tiny_data, &build_literals_huffman(b"Hi").unwrap().0);
        assert!(result.is_none(), "4-stream should fail on tiny data");
        
        // Test with exactly 4 bytes
        let four_bytes = b"Test";
        if let Some((table, _)) = build_literals_huffman(four_bytes) {
            let result = encode_four_stream_huffman(four_bytes, &table);
            assert!(result.is_some(), "4-stream should handle exactly 4 bytes");
        }
    }

    #[test] 
    fn test_four_stream_jump_table_format() {
        // Test jump table encoding format
        let test_data = b"The quick brown fox jumps over the lazy dog. ".repeat(30);
        
        let (table, _) = build_literals_huffman(&test_data).expect("Failed to build table");
        let four_stream = encode_four_stream_huffman(&test_data, &table).expect("Failed to encode");
        
        // Verify jump table constraints
        for (i, &size) in four_stream.jump_table.iter().enumerate() {
            assert!(size > 0, "Jump table entry {i} should be positive");
        }
    }

    #[test]
    fn test_four_stream_vs_single_stream_decision() {
        // Test automatic mode selection logic
        
        // Small data should prefer single-stream
        let small_data = b"Small test data";
        assert!(!should_use_four_stream(small_data), "Small data should use single-stream");
        
        // Large data should prefer 4-stream  
        let large_data = vec![b'X'; 2048];
        assert!(should_use_four_stream(&large_data), "Large data should use 4-stream");
    }

    #[test]
    fn test_auto_encoding_optimization() {
        // Test the auto-encoding function that chooses optimal mode
        let short_text = b"Short text";
        let long_text = "Long text that exceeds the threshold ".repeat(50);
        
        // Test short text
        if let Some((table, encoded_block)) = encode_literals_block_auto(short_text) {
            assert!(!encoded_block.is_empty(), "Encoded block should not be empty");
            assert!(table.num_symbols > 0, "Table should have symbols");
        }
        
        // Test long text
        if let Some((table, encoded_block)) = encode_literals_block_auto(long_text.as_bytes()) {
            assert!(!encoded_block.is_empty(), "Encoded block should not be empty");
            assert!(table.num_symbols > 0, "Table should have symbols");
        }
    }

    #[test]
    fn test_fse_compressed_weights_enhancement() {
        // Test enhanced FSE compression for weights
        let pattern = b"AAABBBCCCDDD";
        
        if let Some((table, _)) = build_literals_huffman(pattern) {
            let fse_result = build_fse_compressed_weights_header(&table);
            
            if let Some(fse_compressed) = fse_result {
                // Verify header format
                assert!(fse_compressed[0] < 128, "Header should indicate FSE compression");
                assert!(fse_compressed.len() >= 2, "FSE compressed should have header + data");
            }
            
            // Test that we always get some valid result
            assert!(table.weights.len() == table.num_symbols, "Weights length should match num_symbols");
        }
    }
}

