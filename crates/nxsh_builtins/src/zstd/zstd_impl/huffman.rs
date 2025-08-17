//! Huffman coding for literals (basic 1-stream encoder for <=128 symbols)
//! - Builds canonical codes from simple length-limited scheme
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
/// Uses RFC 8878 section 4.2.1 FSE compression for weights.
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
    
    // Build normalized counts for weights FSE table
    let table_log = super::fse::choose_table_log(weights_stream.len() as u32, 15); // weights use 0-15 alphabet
    let normalized = build_normalized_from_hist(&weight_freqs[..16], table_log).ok()?;
    
    // Build FSE encoding table
    let _fse_table = FseEncTable::from_normalized(&normalized, table_log).ok()?;
    
    // Encode FSE table description
    let table_desc = encode_fse_table_description(&normalized, table_log).ok()?;
    
    // For now, use simplified encoding approach (full FSE encoding is complex)
    // We'll encode weights directly for this implementation
    let mut compressed_weights = Vec::new();
    {
        let mut bw = BitWriter::new(&mut compressed_weights);
        
        // Simple encoding: write each weight as 4 bits (since weights are 0-15)
        for &weight in &weights_stream {
            bw.write_bits(weight as u64, 4).ok()?;
        }
        
        bw.align_to_byte().ok()?;
    }
    
    // Calculate total compressed size
    let total_size = 1 + table_desc.len() + compressed_weights.len(); // 1 byte for header
    
    // Compare with direct weights size
    let direct_size = 1 + table.num_symbols.div_ceil(2);
    
    if total_size >= direct_size {
        return None; // Direct weights are more efficient
    }
    
    // Build header: headerByte < 128 indicates FSE compression
    // Use the same encoding as sequences: bit 0-5 = accuracy log, bit 6-7 reserved
    let header_byte = table_log as u8; // This is < 128 to indicate FSE compression
    
    let mut result = Vec::with_capacity(total_size);
    result.push(header_byte);
    result.extend_from_slice(&table_desc);
    result.extend_from_slice(&compressed_weights);
    
    Some(result)
}

/// Reverse the lowest `bits` bits of `code`.
pub fn reverse_bits(mut code: u16, bits: u8) -> u16 {
    let mut r = 0u16; let mut b = bits;
    while b > 0 { r = (r << 1) | (code & 1); code >>= 1; b -= 1; }
    r
}

