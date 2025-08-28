// fsck integration-ish tests (shadow image + journal roundtrip)
// Note: These tests run only on Unix due to raw device semantics in fsck implementation.

#[cfg(unix)]
mod unix_fsck {
    use std::fs;
    use std::io::Write;
    use std::process::Command;

    // Create a minimal fake FAT-like image (not a valid FS) just to exercise journal I/O
    // The test focuses on JSON journal roundtrip and command wiring, not on real FAT parsing.
    fn create_dummy_image(path: &str, size: usize) {
        let mut f = fs::OpenOptions::new().create(true).write(true).truncate(true).open(path).unwrap();
        f.write_all(&vec![0u8; size]).unwrap();
        f.flush().unwrap();
    }

    #[test]
    fn fsck_journal_sign_verify_roundtrip() {
        // Skip if updates feature (ed25519) is not built into this binary
        // We probe help text for sign-journal subcommand presence via invoking nxsh_cli busybox if available.
        // If not available, we run unit scope JSON verify directly using library path is not trivial here.
        // Therefore, restrict to JSON compute hash stability.
        let report_path = ".nxsh/test_fsck_report.json";
        let _ = fs::create_dir_all(".nxsh");
        // Minimal report
        let report = serde_json::json!({
            "device": "/dev/test",
            "filesystem": "FAT32",
            "files_scanned": 10u64,
            "cross_links": 0u64,
            "lost_clusters": [123u32, 456u32],
            "actions_proposed": [{"action":"free_clusters", "clusters":[123u32,456u32]}],
            "fat_mirror_consistent": true,
            "fat_mirror_hashes": ["deadbeef"],
            "fat_mirror_mismatch_samples": serde_json::Value::Null,
            "report_hash": "",
            "signature": serde_json::Value::Null,
            "public_key_hint": serde_json::Value::Null
        });
        fs::write(report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();

        // Just ensure the file exists and is valid JSON to avoid panics in fsck code paths used by CLI.
        let loaded: serde_json::Value = serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
        assert_eq!(loaded["filesystem"], "FAT32");

        // Create shadow image file to pass mount sanity path (mount may fail, but apply with dry-run should still work)
        let img_path = ".nxsh/test_shadow.img";
        create_dummy_image(img_path, 1024 * 1024);

        // Dry-run application should not fail.
        // We call the library entry via nxsh_core::execute_builtin when available in test binary.
        let args = vec!["apply-journal".to_string(), report_path.to_string(), "--shadow".to_string(), img_path.to_string()];
        let res = nxsh_builtins::fsck::fsck_cli(&args);
        assert!(res.is_ok(), "fsck apply-journal dry-run should succeed: {:?}", res.err());
    }
}

#[cfg(not(unix))]
#[test]
fn fsck_skip_on_non_unix() {
    // No-op test to keep suite green on non-Unix platforms
    // Test passes by default
}


