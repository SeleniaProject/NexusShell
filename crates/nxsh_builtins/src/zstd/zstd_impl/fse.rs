//! FSE coding for sequences (skeleton)

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
}

pub fn normalize_counts<T: Copy>(_hist: &[T]) -> NormalizedCounts {
    // TODO: Zstd normalization
    NormalizedCounts { counts: vec![], table_log: 5 }
}
