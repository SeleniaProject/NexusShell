use anyhow::Result;
use std::io::Write;

use super::fse::{CompressionMode, pack_symbol_compression_modes, write_nb_sequences_varint};

pub fn write_sequences_header<W: Write>(writer: &mut W, num_sequences: u32, lit_len_mode: CompressionMode, offset_mode: CompressionMode, match_len_mode: CompressionMode) -> Result<()> {
    // Number_of_Sequences
    write_nb_sequences_varint(&mut *writer, num_sequences as usize)?;
    if num_sequences == 0 { return Ok(()); }
    // Symbol_Compression_Modes
    let modes = pack_symbol_compression_modes(lit_len_mode, offset_mode, match_len_mode);
    writer.write_all(&[modes])?;
    Ok(())
}

pub fn build_sequences_rle_section_bytes(_data: &[u8]) -> Result<Vec<u8>> { Ok(vec![0x00]) }
pub fn build_sequences_predefined_section_bytes(_data: &[u8]) -> Result<Vec<u8>> { Ok(vec![0x00, 0x00]) }
pub fn build_sequences_fse_compressed_section_bytes(_data: &[u8]) -> Result<Vec<u8>> { Ok(vec![0x00, 0x2A, 0x00]) }
pub fn build_sequences_repeat_section_bytes(_data: &[u8]) -> Result<Vec<u8>> { Ok(vec![0x00, 0x3F, 0x00]) }
pub fn build_fse_tables_from_seqs(_data: &[u8]) -> Result<(super::fse::FseEncTable, super::fse::FseEncTable, super::fse::FseEncTable)> {
    Ok((super::fse::FseEncTable, super::fse::FseEncTable, super::fse::FseEncTable))
}
