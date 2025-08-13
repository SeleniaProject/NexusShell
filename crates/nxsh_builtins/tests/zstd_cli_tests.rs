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


