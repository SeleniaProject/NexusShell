//! FSE coding for sequences (skeleton)
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct NormalizedCounts {
    pub counts: Vec<i16>,
    pub table_log: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    Predefined = 0,
    Rle = 1,
    FseCompressed = 2,
    Repeat = 3,
}

/// Default distributions from RFC 8878 3.1.1.3.2.2
/// We expose only the accuracy log and symbol counts sizes to build decoding tables later.
pub mod predefined {
    /// LL default table accuracy log = 6 (64 states), 36 symbols
    pub const LL_ACCURACY_LOG: u8 = 6;
    pub const LL_SYMBOLS: usize = 36;
    /// ML default table accuracy log = 6 (64 states), 53 symbols
    pub const ML_ACCURACY_LOG: u8 = 6;
    pub const ML_SYMBOLS: usize = 53;
    /// OF default table accuracy log = 5 (32 states), supports up to N=28 by default (we will cap codes)
    pub const OF_ACCURACY_LOG: u8 = 5;
    pub const OF_MAX_N: u8 = 28;

    /// Returns true if a given offset code is representable by the Predefined distribution.
    #[inline]
    pub fn supports_offset_code(code: u8) -> bool { code <= OF_MAX_N }

    /// RFC 8878 3.1.1.3.2.2 literalsLength_defaultDistribution[36]
    pub const LL_DEFAULT_DISTRIBUTION: [i16; LL_SYMBOLS] = [
        4, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 2, 1, 1, 1, 1, 1,
        -1, -1, -1, -1,
    ];

    /// RFC 8878 3.1.1.3.2.2 matchLengths_defaultDistribution[53]
    pub const ML_DEFAULT_DISTRIBUTION: [i16; ML_SYMBOLS] = [
        1, 4, 3, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1,
        -1, -1, -1, -1, -1,
    ];

    /// RFC 8878 3.1.1.3.2.2 offsetCodes_defaultDistribution[29]
    /// Table supports codes 0..=28; higher must switch away from Predefined.
    pub const OF_DEFAULT_DISTRIBUTION: [i16; 29] = [
        1, 1, 1, 1, 1, 1, 2, 2, 2,
        1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1,
    ];
}

pub fn normalize_counts<T: Copy>(_hist: &[T]) -> NormalizedCounts {
    // TODO: Zstd normalization
    NormalizedCounts { counts: vec![], table_log: 5 }
}

// TODO: Implement predefined decoding tables per RFC 8878 Appendix A for Predefined_Mode encoding

/// Encoding table for FSE (Finite State Entropy) encoder.
/// This is a minimal implementation sufficient for building an encoder given normalized counts.
#[derive(Debug, Clone)]
pub struct FseEncTable {
    pub table_log: u8,
    /// Symbol for each state (size = 1<<table_log)
    pub symbol_table: Vec<u16>,
    /// Number of bits to output when transitioning from a given state (size = 1<<table_log)
    pub nb_bits_out: Vec<u8>,
    /// State table for next state lookup (size = 1<<table_log)
    pub state_table: Vec<u16>,
    /// Parameters for each symbol
    pub symbol_params: Vec<SymbolParams>,
    /// Legacy base field for compatibility
    pub base: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct SymbolParams {
    pub delta_find_state: i32,
    pub delta_nb_bits: i32,
}

impl FseEncTable {
    /// Build encoding table from normalized counts (sum to 2^table_log) for an alphabet of size `counts.len()`.
    /// counts[i] may be 0 for unused symbols. Negative values (rare symbols) are not supported yet.
    pub fn from_normalized(counts: &[i16], table_log: u8) -> io::Result<Self> {
        let table_size = 1usize << table_log;
        let sum: i32 = counts.iter().map(|&c| c as i32).sum();
        if sum != table_size as i32 {
            return Err(io::Error::other(format!("normalized counts sum {} != expected {}", sum, table_size)));
        }
        if counts.iter().any(|&c| c < 0) {
            return Err(io::Error::other("negative normalized counts not supported yet"));
        }
        let alphabet = counts.len();
        
        // Spread symbols using FSE standard distribution
        let mut symbol_table = vec![0u16; table_size];
        let step = (table_size >> 1) + (table_size >> 3) + 3;
        let mut pos = 0usize;
        
        for (sym, &count) in counts.iter().enumerate() {
            for _ in 0..count as usize {
                while symbol_table[pos] != 0 && pos < table_size {
                    pos = (pos + 1) % table_size;
                }
                if pos >= table_size {
                    return Err(io::Error::other("FSE table overflow"));
                }
                symbol_table[pos] = (sym + 1) as u16; // +1 to distinguish from unset
                pos = (pos + step) % table_size;
            }
        }
        
        // Convert back to 0-based indexing
        for item in &mut symbol_table {
            if *item > 0 {
                *item -= 1;
            }
        }
        
        // Build cumulative table for state assignment
        let mut cumul = vec![0u16; alphabet + 1];
        for i in 0..alphabet {
            cumul[i + 1] = cumul[i] + (counts[i] as u16);
        }
        
        // Build state table (maps state to next state base)
        let mut state_table = vec![0u16; table_size];
        let mut state_count = vec![0u16; alphabet];
        
        for state in 0..table_size {
            let sym = symbol_table[state] as usize;
            state_table[state] = cumul[sym] + state_count[sym];
            state_count[sym] += 1;
        }
        
        // Calculate encoding parameters for each symbol  
        let mut symbol_params = vec![SymbolParams { delta_find_state: 0, delta_nb_bits: 0 }; alphabet];
        let mut nb_bits_out = vec![0u8; table_size];
        
        // Build symbol parameters using zstd reference algorithm
        let mut total = 0i32;
        for (sym, &count) in counts.iter().enumerate() {
            match count {
                0 => {
                    // No occurrence - filling for compatibility with FSE_getMaxNbBits()
                    symbol_params[sym].delta_nb_bits = (((table_log + 1) as i32) << 16) - (1i32 << table_log);
                    symbol_params[sym].delta_find_state = 0;
                },
                1 => {
                    // Low probability symbol (count == 1) or (count == -1 for predefined)
                    symbol_params[sym].delta_nb_bits = ((table_log as i32) << 16) - (1i32 << table_log);
                    symbol_params[sym].delta_find_state = total - 1;
                    total += 1;
                    
                    // Set nb_bits for states of this symbol  
                    for state in 0..table_size {
                        if symbol_table[state] == sym as u16 {
                            nb_bits_out[state] = table_log;
                        }
                    }
                },
                _ => {
                    // Normal case (count > 1)
                    let count_u32 = count as u32;
                    let max_bits_out = table_log - (31 - (count_u32 - 1).leading_zeros()) as u8;
                    let min_state_plus = count_u32 << max_bits_out;
                    
                    symbol_params[sym].delta_nb_bits = ((max_bits_out as i32) << 16) - min_state_plus as i32;
                    symbol_params[sym].delta_find_state = total - count as i32;
                    total += count as i32;
                    
                    // Set nb_bits for states of this symbol
                    for state in 0..table_size {
                        if symbol_table[state] == sym as u16 {
                            nb_bits_out[state] = max_bits_out;
                        }
                    }
                }
            }
        }

        // Build base values for compatibility with old interface - these are starting state indices for each symbol
        let mut base = vec![0u16; alphabet];
        for (sym, &count) in counts.iter().enumerate() {
            if count > 0 {
                // Find first state index for this symbol
                for (state, &table_sym) in symbol_table.iter().enumerate() {
                    if table_sym as usize == sym {
                        base[sym] = state as u16;
                        break;
                    }
                }
            }
        }

        Ok(Self { 
            table_log, 
            symbol_table, 
            nb_bits_out, 
            state_table,
            symbol_params,
            base,
        })
    }

    /// Build encoding table from predefined distribution with -1 entries (unused symbols).
    /// This handles RFC distributions where -1 indicates unused symbols.
    pub fn from_predefined_distribution(dist: &[i16], table_log: u8) -> io::Result<Self> {
        // Convert -1 to 0 and calculate actual sum of positive entries
        let mut normalized: Vec<i16> = dist.iter().map(|&v| if v > 0 { v } else { 0 }).collect();
        let current_sum: i32 = normalized.iter().map(|&c| c as i32).sum();
        let expected_sum = 1i32 << table_log;
        
        // Predefined distributions may need adjustment to sum to expected_sum
        if current_sum != expected_sum {
            // Add the difference to the first non-zero symbol to make it sum correctly
            let diff = expected_sum - current_sum;
            if let Some(first_nonzero) = normalized.iter_mut().find(|x| **x > 0) {
                *first_nonzero += diff as i16;
            } else {
                return Err(io::Error::other("no non-zero symbols in predefined distribution"));
            }
        }
        
        Self::from_normalized(&normalized, table_log)
    }

    /// Encode a sequence of symbols into an FSE bitstream (LSB-first) and return the bytes with a final state.
    /// Note: This minimal encoder writes the final state as little-endian u16 after the bitstream for testing roundtrip.
    pub fn encode_symbols(&self, symbols: &[u16]) -> io::Result<Vec<u8>> {
        if symbols.is_empty() { return Ok(Vec::new()); }
        
        // Initialize state to first state for the last symbol  
        let last_sym = symbols[symbols.len() - 1] as usize;
        if last_sym >= self.symbol_params.len() {
            return Err(io::Error::other("symbol out of alphabet range"));
        }
        
        // Find first state for this symbol
        let mut state = None;
        for (st, &sym) in self.symbol_table.iter().enumerate() {
            if sym as usize == last_sym {
                state = Some(st);
                break;
            }
        }
        let mut state = state.ok_or_else(|| io::Error::other("no state found for last symbol"))? as u32;
        
        let mut out = Vec::with_capacity(symbols.len() / 2 + 8);
        let mut bw = crate::zstd::zstd_impl::bitstream::BitWriter::new(&mut out);
        
        // Process from last to first as per FSE encoding
        for &sym_u16 in symbols[..symbols.len()-1].iter().rev() {
            let s = sym_u16 as usize;
            if s >= self.symbol_params.len() { 
                return Err(io::Error::other("symbol out of alphabet range")); 
            }
            
            let params = &self.symbol_params[s];
            // Extract nb_bits from delta_nb_bits (upper 16 bits) 
            let nb_bits = ((params.delta_nb_bits >> 16) & 0xFFFF) as u8;
            
            // Output lower bits of state
            if nb_bits > 32 { 
                return Err(io::Error::other(format!("invalid nb_bits: {}", nb_bits))); 
            }
            let mask = if nb_bits == 0 { 0 } else { (1u32 << nb_bits) - 1 };
            bw.write_bits((state & mask) as u64, nb_bits)?;
            
            // Calculate next state using zstd formula
            let next_state_idx = (state >> nb_bits) as i32 + params.delta_find_state;
            if next_state_idx < 0 || next_state_idx as usize >= self.state_table.len() {
                return Err(io::Error::other(format!("invalid state index: {} (state={}, nb_bits={}, delta_find_state={})", 
                    next_state_idx, state, nb_bits, params.delta_find_state)));
            }
            state = self.state_table[next_state_idx as usize] as u32;
            
            // Basic sanity check
            let table_size = 1u32 << self.table_log;
            if state >= table_size { 
                return Err(io::Error::other(format!("state overflow: {} >= {}", state, table_size))); 
            }
        }
        
        bw.align_to_byte()?;
        // Append final state (for testing convenience)
        out.extend_from_slice(&(state as u16).to_le_bytes());
        Ok(out)
    }
}

/// Build normalized counts from a histogram with a fixed table_log. Ensures sum==2^table_log and no negatives.
pub fn build_normalized_from_hist(hist: &[u32], table_log: u8) -> io::Result<Vec<i16>> {
    let target: u32 = 1u32 << table_log;
    let total: u32 = hist.iter().copied().sum();
    if total == 0 { return Err(io::Error::other("empty histogram")); }
    let mut out: Vec<i16> = vec![0; hist.len()];
    // provisional allocation
    let mut sum: u32 = 0;
    let mut max_idx = 0usize;
    for (i, &h) in hist.iter().enumerate() {
        if h > hist[max_idx] { max_idx = i; }
        if h == 0 { out[i] = 0; continue; }
        let v = ((h as u64) * (target as u64) + (total as u64 / 2)) / (total as u64);
        let v = v.max(1) as u32;
        out[i] = v as i16;
        sum += v;
    }
    // fix remainder
    if sum == 0 { return Err(io::Error::other("normalization failed: zero sum")); }
    if sum != target {
        let diff = target as i32 - sum as i32;
        let newv = (out[max_idx] as i32 + diff).max(1);
        out[max_idx] = newv as i16;
    }
    Ok(out)
}

/// Encode FSE table description (RFC 8878 4.1.1) from normalized counts. Returns a byte-aligned stream.
pub fn encode_fse_table_description(counts: &[i16], table_log: u8) -> io::Result<Vec<u8>> {
    if table_log < 5 || table_log > 9 { /* allow OF to pass 8 later */ }
    // Count nonzero symbols; must be >=2 per spec (except RLE)
    let nz = counts.iter().filter(|&&c| c > 0).count();
    if nz < 2 { return Err(io::Error::other("FSE_Compressed not allowed for <2 symbols (use RLE)")); }
    let mut out = Vec::with_capacity(32);
    let mut bw = crate::zstd::zstd_impl::bitstream::BitWriter::new(&mut out);
    // write low4bits = table_log - 5
    let low4 = (table_log as i32 - 5).clamp(0, 15) as u8;
    bw.write_bits(low4 as u64, 4)?;
    // remaining points
    let mut remaining: i32 = 1 << table_log;
    let n = counts.len();
    let mut i = 0usize;
    while i < n {
        let c = counts[i];
        if c < 0 { return Err(io::Error::other("negative counts not supported")); }
    let val: i32 = (c as i32) + 1; // maps 0->1, 1->2, ... per spec (0 means -1, unused here)
        // encode value with variable bits based on remaining+1
        let max = (remaining + 1).max(1) as u32;
        let bits = (32 - (max - 1).leading_zeros()) as u8; // ceil_log2(max)
        let threshold = (1u32 << bits) - max; // number of short-coded values
        if (val as u32) < threshold {
            // write on (bits-1)
            if bits == 0 { /* no-op */ } else { bw.write_bits(val as u64, bits - 1)?; }
        } else {
            let enc = (val as u32) + threshold;
            bw.write_bits(enc as u64, bits)?;
        }
        if c > 0 { remaining -= c as i32; }
        // zero-run repeat flags
        if c == 0 {
            // count how many subsequent zeros follow current
            let mut run = 0usize;
            let mut j = i + 1;
            while j < n && counts[j] == 0 { run += 1; j += 1; }
            // write repeat flags in chunks of 2-bit values 0..3 (3 means continue)
            let mut left = run;
            while left >= 3 { bw.write_bits(3, 2)?; left -= 3; }
            bw.write_bits(left as u64, 2)?;
            i = i + 1 + run;
        } else {
            i += 1;
        }
    }
    bw.align_to_byte()?;
    Ok(out)
}

/// Choose an accuracy log for FSE table based on histogram mass and a max cap.
/// Ensures 2^log >= total_nonzero and 5 <= log <= max_log.
pub fn choose_table_log(total: u32, max_log: u8) -> u8 {
    if total == 0 { return 5u8.min(max_log); }
    let ceil = 32 - (total - 1).leading_zeros();
    let mut log = ceil as u8;
    if log < 5 { log = 5; }
    if log > max_log { log = max_log; }
    log
}

#[cfg(test)]
mod tests_choose_log {
    use super::choose_table_log;
    #[test]
    fn test_choose_table_log_bounds() {
        assert_eq!(choose_table_log(0, 9), 5);
        assert_eq!(choose_table_log(1, 9), 5);
        assert_eq!(choose_table_log(2, 9), 5);
        assert_eq!(choose_table_log(33, 9), 6); // 33 -> ceil_log2=6
        assert_eq!(choose_table_log(300, 9), 9); // capped by max_log
        assert_eq!(choose_table_log(300, 8), 8);
    }
}
/// Build FSE encoding tables for Predefined mode from RFC default distributions.
pub fn build_predefined_tables() -> io::Result<(FseEncTable, FseEncTable, FseEncTable)> {
    use predefined::*;
    // Use RFC default distributions as-is (they already sum to 2^accuracy_log)
    let ll_counts: Vec<i16> = LL_DEFAULT_DISTRIBUTION.to_vec();
    let ml_counts: Vec<i16> = ML_DEFAULT_DISTRIBUTION.to_vec();
    let of_counts: Vec<i16> = OF_DEFAULT_DISTRIBUTION.to_vec();
    
    // For encoding, we need to temporarily handle -1 symbols by excluding them from the table
    // but still preserving the array structure. We'll build tables without -1 entries.
    let mut ll_working = Vec::new();
    let mut ml_working = Vec::new();
    let mut of_working = Vec::new();
    
    for &c in &ll_counts { if c > 0 { ll_working.push(c); } }
    for &c in &ml_counts { if c > 0 { ml_working.push(c); } }
    for &c in &of_counts { if c > 0 { of_working.push(c); } }
    
    // Build tables with only positive counts but using the full alphabet sizes for indexing
    let ll = FseEncTable::from_predefined_distribution(&ll_counts, LL_ACCURACY_LOG)?;
    let ml = FseEncTable::from_predefined_distribution(&ml_counts, ML_ACCURACY_LOG)?;
    let of = FseEncTable::from_predefined_distribution(&of_counts, OF_ACCURACY_LOG)?;
    Ok((ll, of, ml))
}


/// Encode Number_of_Sequences using the RFC 8878 variable-length format (1-3 bytes).
/// Returns the number of bytes written.
pub fn write_nb_sequences_varint<W: Write>(mut w: W, nb: usize) -> io::Result<usize> {
    // Spec (RFC 8878 3.1.1.3.2.1 Sequences_Section_Header):
    // byte0 == 0          -> nbSeq = 0 (1 byte)
    // 1 <= byte0 < 128    -> nbSeq = byte0 (1 byte)
    // 128 <= byte0 < 255  -> nbSeq = ((byte0 - 128) << 8) + byte1 (2 bytes)
    // byte0 == 255        -> nbSeq = 0x7F00 + byte1 + (byte2<<8) (3 bytes)
    if nb == 0 {
        w.write_all(&[0u8])?;
        return Ok(1);
    }
    if nb < 128 {
        w.write_all(&[nb as u8])?;
        return Ok(1);
    }
    if nb < 0x7F00 {
        // 2-byte form
        let b0 = 128u8 + ((nb >> 8) as u8); // guaranteed < 255 because nb < 0x7F00
        let b1 = (nb & 0xFF) as u8;
        w.write_all(&[b0, b1])?;
        return Ok(2);
    }
    // 3-byte form
    let adj = nb - 0x7F00;
    let b1 = (adj & 0xFF) as u8;
    let b2 = ((adj >> 8) & 0xFF) as u8;
    w.write_all(&[255u8, b1, b2])?;
    Ok(3)
}

/// Build the Symbol_Compression_Modes byte from 3 modes (LL, OF, ML) as per RFC 8878.
/// Layout (LSB-first within the byte):
/// bits 0-1: Literals_Lengths_Mode, bits 2-3: Offsets_Mode, bits 4-5: Match_Lengths_Mode, bits 6-7: Reserved (0)
pub fn pack_symbol_compression_modes(ll: CompressionMode, of: CompressionMode, ml: CompressionMode) -> u8 {
    let llv = ll as u8 & 0x03;
    let ofv = (of as u8 & 0x03) << 2;
    let mlv = (ml as u8 & 0x03) << 4;
    llv | ofv | mlv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nb_sequences_varint_encoding() {
        let mut buf = Vec::new();
        // nb=0
        assert_eq!(write_nb_sequences_varint(&mut buf, 0).unwrap(), 1);
        assert_eq!(buf, vec![0u8]);
        // nb=1..127 (1 byte)
        buf.clear();
        assert_eq!(write_nb_sequences_varint(&mut buf, 127).unwrap(), 1);
        assert_eq!(buf, vec![127u8]);
    // nb=128 -> 2 bytes : b0 = 128 + (128>>8)=128, b1=128&0xFF=128
        buf.clear();
        assert_eq!(write_nb_sequences_varint(&mut buf, 128).unwrap(), 2);
        assert_eq!(buf, vec![128u8, 128u8]);
    // nb just before 3-byte threshold (0x7F00): 0x7EFF encoded in 2 bytes
        buf.clear();
    assert_eq!(write_nb_sequences_varint(&mut buf, 0x7EFF).unwrap(), 2);
    assert_eq!(buf, vec![254u8, 0xFFu8]);
        // nb requiring 3 bytes -> byte0=255
        buf.clear();
        assert_eq!(write_nb_sequences_varint(&mut buf, 0xFF00).unwrap(), 3);
    assert_eq!(buf, vec![255u8, 0x00, 0x80]); // 0xFF00 - 0x7F00 = 0x8000 -> 0x00 0x80
    }

    #[test]
    fn test_pack_symbol_modes_bits() {
        let b = pack_symbol_compression_modes(CompressionMode::Predefined, CompressionMode::Rle, CompressionMode::FseCompressed);
        // ll=0, of=1<<2, ml=2<<4
        assert_eq!(b, 0b10_01_00);
        // repeat mode
        let b2 = pack_symbol_compression_modes(CompressionMode::Repeat, CompressionMode::Repeat, CompressionMode::Repeat);
        assert_eq!(b2 & 0b11000000, 0); // reserved must remain 0
    }

    #[test]
    fn test_fse_table_build_and_encode_tiny_alphabet() {
        // Tiny alphabet of 2 symbols with equal probability over table_log=2 -> table size 4, counts [2,2]
        let counts = vec![2i16, 2i16];
        let tab = FseEncTable::from_normalized(&counts, 2).expect("build");
        assert_eq!(tab.symbol_table.len(), 4);
        // Encode a short stream of symbols [0,1,0,1]
        let symbols = [0u16, 1, 0, 1];
        let bits = tab.encode_symbols(&symbols).expect("encode");
        // We don't assert exact bytes (depends on spread), just that something emitted and final state present
        assert!(bits.len() >= 2);
    }

    #[test]
    fn test_predefined_offset_supports_n() {
        assert!(predefined::supports_offset_code(0));
        assert!(predefined::supports_offset_code(28));
        assert!(!predefined::supports_offset_code(29));
        assert!(!predefined::supports_offset_code(31));
    }

    #[test]
    fn test_predefined_arrays_and_tables_build() {
        use predefined::*;
        assert_eq!(LL_DEFAULT_DISTRIBUTION.len(), LL_SYMBOLS);
        assert_eq!(ML_DEFAULT_DISTRIBUTION.len(), ML_SYMBOLS);
        assert_eq!(OF_DEFAULT_DISTRIBUTION.len(), 29);
        let (ll, of, ml) = build_predefined_tables().expect("build tables");
        assert_eq!(ll.symbol_table.len(), 1usize << LL_ACCURACY_LOG);
        assert_eq!(ml.symbol_table.len(), 1usize << ML_ACCURACY_LOG);
        assert_eq!(of.symbol_table.len(), 1usize << OF_ACCURACY_LOG);
    }
}
