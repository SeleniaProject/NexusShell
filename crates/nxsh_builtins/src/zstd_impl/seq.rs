#[derive(Debug, Clone, Copy)]
pub struct Seq {
    pub litlen: u32,
    pub matchlen: u32,
    pub of_code: u32,
    pub of_extra: u32,
}

/// Convert raw matches and literals to a flat sequence stream (very simplified for now).
pub fn tokenize_sequences(input: &[u8]) -> (Vec<Seq>, Vec<u8>) {
    // Placeholder: no sequences yet, only literals all-in-one block
    let mut lit = Vec::with_capacity(input.len());
    lit.extend_from_slice(input);
    (Vec::new(), lit)
}
