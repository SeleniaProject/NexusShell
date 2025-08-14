use ruzstd::streaming_decoder::StreamingDecoder;

#[test]
fn zstd_store_empty_input_roundtrip() {
    let mut out: Vec<u8> = Vec::new();
    let mut in_data: &[u8] = &[];
    nxsh_builtins::zstd::write_store_frame_stream(&mut out, &mut in_data, 0).unwrap();
    let mut dec = StreamingDecoder::new(&out[..]).unwrap();
    let mut restored = Vec::new();
    use std::io::Read;
    dec.read_to_end(&mut restored).unwrap();
    assert_eq!(restored, b"");
}

#[test]
fn zstd_store_single_block_boundary_minus_one() {
    // Size exactly (1<<21)-1 bytes should fit in a single RAW block
    let size = (1u32 << 21) - 1;
    let data = vec![0xAA; size as usize];
    let mut out: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&data);
    nxsh_builtins::zstd::write_store_frame_stream(&mut out, &mut cursor, data.len() as u64).unwrap();
    let mut dec = StreamingDecoder::new(&out[..]).unwrap();
    let mut restored = Vec::new();
    use std::io::Read;
    dec.read_to_end(&mut restored).unwrap();
    assert_eq!(restored.len(), data.len());
    assert_eq!(restored[0], 0xAA);
}

#[test]
fn zstd_store_multi_block_boundary_plus_one() {
    // Size (1<<21)+1 ensures multi-block emission
    let size = (1usize << 21) + 1;
    let data = vec![0xBB; size];
    let mut out: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&data);
    nxsh_builtins::zstd::write_store_frame_stream(&mut out, &mut cursor, data.len() as u64).unwrap();
    let mut dec = StreamingDecoder::new(&out[..]).unwrap();
    let mut restored = Vec::new();
    use std::io::Read;
    dec.read_to_end(&mut restored).unwrap();
    assert_eq!(restored.len(), data.len());
    assert_eq!(restored[0], 0xBB);
    assert_eq!(restored[restored.len()-1], 0xBB);
}


