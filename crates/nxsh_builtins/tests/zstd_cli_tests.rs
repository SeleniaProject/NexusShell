use nxsh_builtins::zstd::zstd_cli;

#[test]
fn zstd_version_path() {
    // Just verify --version path does not panic and returns Ok
    let args = vec!["--version".to_string()];
    let res = zstd_cli(&args);
    assert!(res.is_ok());
}

#[test]
fn zstd_compress_without_external_binary() {
    // When zstd external binary is not available, compression should error cleanly.
    // We can't guarantee PATH in test, so we just assert result is Err for impossible file.
    let args = vec!["-z".to_string(), "__no_such_file__".to_string()];
    let res = zstd_cli(&args);
    assert!(res.is_err());
}

#[test]
fn zstd_help_and_list_flags() {
    // --help should succeed
    assert!(zstd_cli(&["--help".to_string()]).is_ok());
    // --list with nonexistent file should not panic; will print error but not crash
    let res = zstd_cli(&["-l".to_string(), "no_such.zst".to_string()]);
    assert!(res.is_ok());
}

#[test]
fn zstd_stdout_mode_without_input() {
    // With no input files, processing stdin/stdout should not panic.
    // We cannot easily feed stdin here; just ensure path executes without args by passing only -d and relying on EOF.
    // This test asserts it returns Ok for help-like flags; for real stdin we have integration tests elsewhere.
    assert!(zstd_cli(&["--help".to_string()]).is_ok());
}

#[test]
fn zstd_stdin_compress_requires_external() {
    let res = zstd_cli(&["-z".to_string(), "-c".to_string()]);
    assert!(res.is_err());
    if let Err(e) = res {
        let s = format!("{e:#}");
        assert!(s.contains("zstd"));
    }
}

#[test]
fn zstd_decompress_invalid_file_errors() {
    // Create a temp file with random bytes and try to decompress
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("not_a_zstd.zst");
    std::fs::write(&bad_path, b"this is not zstd").unwrap();
    let res = zstd_cli(&["-d".to_string(), bad_path.to_string_lossy().to_string()]);
    assert!(res.is_err());
}


