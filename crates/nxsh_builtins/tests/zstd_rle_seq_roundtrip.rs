#[cfg(feature = "compression-zstd")]
use ruzstd::streaming_decoder::StreamingDecoder;

#[test]
#[cfg(feature = "compression-zstd")]
fn zstd_rle_single_sequence_roundtrip() {
    // 小さめの入力（<32バイト）で Huffman を避け、単一のシーケンスが発生するようにする
    // 例: "hello___hello" のように、前半のリテラル + 直後に3文字以上の一致が1つ
    let mut input = Vec::new();
    input.extend_from_slice(b"abcXYZabc"); // "abc" が距離6で再出現（最低3で一致）

    // 圧縮（公開API: write_store_frame_stream_with_options を使用）
    let mut compressed = Vec::new();
    let mut cursor = std::io::Cursor::new(&input);
    nxsh_builtins::zstd::write_store_frame_stream_with_options(
        &mut compressed,
        &mut cursor,
        input.len() as u64,
        false,
        3,
    )
    .expect("compress");

    // 復号（ruzstd の標準API: Read実装として new(&[u8]) を使う）
    let mut decoder = StreamingDecoder::new(&compressed[..]).expect("decoder");
    let mut out = Vec::new();
    use std::io::Read;
    decoder.read_to_end(&mut out).expect("decompress");

    assert_eq!(out, input);
}
