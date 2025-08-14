use nxsh_builtins::zstd::zstd_cli;

#[test]
fn zstd_roundtrip_external_when_available() {
    // Prepare input file
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("roundtrip.txt");
    let original = "The quick brown fox jumps over the lazy dog\n".repeat(256);
    std::fs::write(&input_path, original.as_bytes()).unwrap();

    // If external zstd exists, perform roundtrip; otherwise assert compression errors
    if which::which("zstd").is_ok() {
        // Compress: produces roundtrip.txt.zst
        assert!(zstd_cli(&[input_path.to_string_lossy().to_string()]).is_ok());
        let zst_path = input_path.with_extension("txt.zst");
        assert!(zst_path.exists());

        // Decompress back to roundtrip.txt (overwrites after removing)
        // Remove original to ensure decompressor writes it back
        std::fs::remove_file(&input_path).unwrap();
        assert!(zstd_cli(&["-d".into(), zst_path.to_string_lossy().to_string()]).is_ok());
        let round = std::fs::read_to_string(&input_path).unwrap();
        assert_eq!(round, original);
    } else {
        // Expect clean error due to missing external binary
        let res = zstd_cli(&[input_path.to_string_lossy().to_string()]);
        assert!(res.is_err());
    }
}


