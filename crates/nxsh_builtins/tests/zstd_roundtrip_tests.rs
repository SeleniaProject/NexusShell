use nxsh_builtins::zstd::zstd_cli;

#[test]
fn zstd_roundtrip_external_when_available() {
    // Prepare input file
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("roundtrip.txt");
    let original = "The quick brown fox jumps over the lazy dog\n".repeat(256);
    std::fs::write(&input_path, original.as_bytes()).unwrap();

    // Pure Rust store-mode compression should always work
    assert!(zstd_cli(&[input_path.to_string_lossy().to_string()]).is_ok());
    let zst_path = input_path.with_extension("txt.zst");
    assert!(zst_path.exists());

    // Decompress back to roundtrip.txt, forcing overwrite if necessary
    assert!(zstd_cli(&["-d".into(), "-f".into(), zst_path.to_string_lossy().to_string()]).is_ok());
    let round = std::fs::read_to_string(&input_path).unwrap();
    assert_eq!(round, original);
}


