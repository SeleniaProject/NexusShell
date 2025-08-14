use nxsh_builtins::unzstd::unzstd_cli;

#[test]
fn unzstd_invalid_file_errors() {
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("bad.zst");
    std::fs::write(&bad_path, b"this is not zstd").unwrap();
    let res = unzstd_cli(&[bad_path.to_string_lossy().to_string()]);
    assert!(res.is_err());
}

#[test]
fn unzstd_test_mode_invalid_reports_failure() {
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("bad2.zst");
    std::fs::write(&bad_path, b"not a zstd file").unwrap();
    let res = unzstd_cli(&["-t".to_string(), bad_path.to_string_lossy().to_string()]);
    assert!(res.is_err());
}

#[test]
fn unzstd_roundtrip_external_when_available() {
    // Only run this when external zstd is available
    if which::which("zstd").is_err() {
        return;
    }

    use nxsh_builtins::zstd::zstd_cli;
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("roundtrip_unzstd.txt");
    let original = "Zstd roundtrip via external compressor and pure-rust decompressor\n".repeat(64);
    std::fs::write(&input_path, original.as_bytes()).unwrap();

    // Compress using external zstd via our zstd_cli wrapper
    assert!(zstd_cli(&[input_path.to_string_lossy().to_string()]).is_ok());
    let zst_path = input_path.with_extension("txt.zst");
    assert!(zst_path.exists());

    // Remove original to avoid "File exists" on decompression
    std::fs::remove_file(&input_path).unwrap();

    // Decompress with -k to keep the .zst
    assert!(unzstd_cli(&["-k".to_string(), zst_path.to_string_lossy().to_string()]).is_ok());

    // Validate content restored
    let restored = std::fs::read_to_string(&input_path).unwrap();
    assert_eq!(restored, original);
}


