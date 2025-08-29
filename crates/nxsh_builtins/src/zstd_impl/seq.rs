#[derive(Debug, Clone, Copy)]
pub struct Seq {
    pub litlen: u32,
    pub matchlen: u32,
    pub of_code: u32,
    pub of_extra: u32,
}

/// Convert input into (sequences, literals_stream). Minimal stub: no sequences.
pub fn tokenize_sequences(input: &[u8]) -> (Vec<Seq>, Vec<u8>) {
    (Vec::new(), input.to_vec())
}

/// Older call-sites expect a (seq_bytes, literals_stream) pair.
/// Minimal stub: return empty seq_bytes and a clone of input.
pub fn tokenize_full(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    (Vec::new(), input.to_vec())
}

/// Return one minimal "sequence" candidate and the literals stream; used for fallbacks.
pub fn tokenize_first(input: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if input.is_empty() {
        return None;
    }
    Some((vec![0u8], input.to_vec()))
}
