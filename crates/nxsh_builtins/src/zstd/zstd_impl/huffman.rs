//! Huffman coding for literals (basic 1-stream encoder for <=128 symbols)
//! - Builds canonical codes from simple length-limited scheme
//! - Emits raw (direct) weights header (headerByte >= 128)
//! - Returns code map and header bytes for Compressed_Literals_Block

use std::cmp::Ordering;

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
    for s in 0..=max_sym { if freq[s] > 0 { present.push((s, freq[s])); } }
    if present.len() < 2 { return None; }

    // Determine code lengths: at most 11, but with <=128 symbols, ceil_log2(n) <= 7
    let n = present.len();
    let nb = ceil_log2(n); // shortest max bits
    let need_shorter = (1usize << nb) - n; // number of codes with length nb-1

    // Sort by frequency descending (better compression), then by symbol asc
    present.sort_by(|a,b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut lens = vec![nb; n];
    for i in 0..need_shorter { lens[i] = nb.saturating_sub(1); }

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
    for s in 0..=max_sym { let l = lengths[s] as usize; if l>0 { bl_count[l] += 1; } }
    let mut next_code = [0u16; 12];
    let mut code = 0u16;
    for l in 1..=max_bits as usize {
        code = (code + bl_count[l-1]) << 1;
        next_code[l] = code;
    }
    let mut codes: [Option<(u16,u8)>; 256] = [None; 256];
    // Assign codes: increasing length, increasing symbol value
    let mut by_len_then_sym: Vec<(usize,u8)> = Vec::new(); // (sym, len)
    for s in 0..=max_sym { let l = lengths[s]; if l>0 { by_len_then_sym.push((s, l)); } }
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
    let mut header = Vec::with_capacity(1 + (number_of_symbols + 1) / 2);
    header.push(header_byte);
    let mut i = 0usize;
    while i < number_of_symbols {
        let w0 = *weights.get(i).unwrap_or(&0) & 0x0F;
        let w1 = *weights.get(i+1).unwrap_or(&0) & 0x0F;
        header.push((w0 << 4) | w1);
        i += 2;
    }

    Some((HuffmanTable { weights, codes, num_symbols: number_of_symbols }, header))
}

/// Reverse the lowest `bits` bits of `code`.
pub fn reverse_bits(mut code: u16, bits: u8) -> u16 {
    let mut r = 0u16; let mut b = bits;
    while b > 0 { r = (r << 1) | (code & 1); code >>= 1; b -= 1; }
    r
}

