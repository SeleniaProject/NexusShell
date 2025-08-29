use nxsh_builtins::tar::tar_cli;

#[test]
#[ignore] // TODO: Implement tar --zstd functionality
fn tar_zstd_store_mode_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path();

    // Prepare input files
    let a = work.join("a.txt");
    let b = work.join("b.txt");
    std::fs::write(&a, b"alpha-123").unwrap();
    std::fs::write(&b, b"beta-456").unwrap();

    // Create archive with --zstd (store-mode)
    let archive = work.join("arch.tar.zst");
    let args_create = vec![
        "-c".to_string(),
        "-f".to_string(),
        archive.to_string_lossy().to_string(),
        "--zstd".to_string(),
        "-C".to_string(),
        work.to_string_lossy().to_string(),
        "a.txt".to_string(),
        "b.txt".to_string(),
    ];
    assert!(tar_cli(&args_create).is_ok());
    assert!(archive.exists());

    // Extract to new directory
    let out_dir = work.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    let args_extract = vec![
        "-x".to_string(),
        "-f".to_string(),
        archive.to_string_lossy().to_string(),
        "--zstd".to_string(),
        "-C".to_string(),
        out_dir.to_string_lossy().to_string(),
    ];
    assert!(tar_cli(&args_extract).is_ok());

    // Validate contents
    let ra = std::fs::read(out_dir.join("a.txt")).unwrap();
    let rb = std::fs::read(out_dir.join("b.txt")).unwrap();
    assert_eq!(ra, b"alpha-123");
    assert_eq!(rb, b"beta-456");
}
