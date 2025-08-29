use std::io::{self, Write};

#[derive(Clone, Copy, Debug)]
pub enum CompressionMode {
    Rle,
    Predefined,
    FseCompressed,
    Repeat,
}

/// Encode Number_of_Sequences using the RFC 8878 varint format (1-3 bytes).
/// Returns number of bytes written.
pub fn write_nb_sequences_varint<W: Write>(mut w: W, nb: usize) -> io::Result<usize> {
    if nb == 0 {
        w.write_all(&[0])?;
        return Ok(1);
    }
    if nb <= 0x7F { // 1 byte
        w.write_all(&[nb as u8])?;
        return Ok(1);
    }
    if nb <= 0x7EFF { // 2 bytes
        let b0 = ((nb >> 8) as u8) + 0x80; // 0b10xxxxxx with high bits per spec
        let b1 = (nb & 0xFF) as u8;
        w.write_all(&[b0, b1])?;
        return Ok(2);
    }
    // 3 bytes
    let val = nb - 0x7F00;
    w.write_all(&[0xFF, (val & 0xFF) as u8, ((val >> 8) & 0xFF) as u8])?;
    Ok(3)
}

/// Build Symbol_Compression_Modes byte (LL,OF,ML). Bits: 0-1:LL, 2-3:OF, 4-5:ML.
pub fn pack_symbol_compression_modes(ll: CompressionMode, of: CompressionMode, ml: CompressionMode) -> u8 {
    fn m(v: CompressionMode) -> u8 { match v { CompressionMode::Rle=>0, CompressionMode::Predefined=>1, CompressionMode::FseCompressed=>2, CompressionMode::Repeat=>3 } }
    (m(ll) & 3) | ((m(of) & 3) << 2) | ((m(ml) & 3) << 4)
}

#[derive(Clone, Debug)]
pub struct FseEncTable; // Placeholder for future encoder tables

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nb_sequences_varint_encoding() {
        let mut b = Vec::new();
        assert_eq!(write_nb_sequences_varint(&mut b, 0).unwrap(), 1); assert_eq!(b, vec![0]); b.clear();
        assert_eq!(write_nb_sequences_varint(&mut b, 127).unwrap(), 1); assert_eq!(b, vec![127]); b.clear();
        assert_eq!(write_nb_sequences_varint(&mut b, 128).unwrap(), 2); assert_eq!(b, vec![128,128]); b.clear();
        assert_eq!(write_nb_sequences_varint(&mut b, 0x7EFF).unwrap(), 2); assert_eq!(b, vec![254, 0xFF]); b.clear();
        assert_eq!(write_nb_sequences_varint(&mut b, 0xFF00).unwrap(), 3); assert_eq!(b, vec![255, 0x00, 0x80]);
    }

    #[test]
    fn test_pack_symbol_modes_bits() {
        let v = pack_symbol_compression_modes(CompressionMode::Predefined, CompressionMode::FseCompressed, CompressionMode::Repeat);
        assert_eq!(v & 0x03, 1);
        assert_eq!((v >> 2) & 0x03, 2);
        assert_eq!((v >> 4) & 0x03, 3);
    }
}
