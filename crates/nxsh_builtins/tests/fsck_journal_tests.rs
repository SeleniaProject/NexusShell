// fsck journal signing and verification tests
// These tests focus on the JSON journal flow (sign-journal, verify-journal) and
// do not require a real FAT image. They validate cryptographic signing paths and
// tamper detection using the public CLI entry.

use std::fs;
use std::io::Write;
use tempfile::tempdir;

// Use tokio runtime because fsck_cli is async
use nxsh_builtins::fsck::fsck_cli;

#[cfg(feature = "updates")]
fn make_keypair_hex() -> (String, String) {
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use rand::RngCore;
    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    let sk = SigningKey::from_bytes(&seed);
    let vk: VerifyingKey = (&sk).into();
    (hex::encode(seed), hex::encode(vk.to_bytes()))
}

#[tokio::test]
#[cfg(feature = "updates")]
async fn fsck_journal_sign_and_verify_success() {
    let dir = tempdir().unwrap();
    let journal_path = dir.path().join("report.json");
    let (sk_hex, pk_hex) = make_keypair_hex();
    let sk_path = dir.path().join("ed25519.sk");
    let pk_path = dir.path().join("ed25519.pk");

    // Minimal valid report JSON for signing (fields must match FsckReport)
    let report_json = serde_json::json!({
        "device": "/dev/mock0p1",
        "filesystem": "FAT32",
        "files_scanned": 10u64,
        "cross_links": 0u64,
        "lost_clusters": [2u32, 3u32],
        "actions_proposed": [{ "action": "free_clusters", "clusters": [2u32, 3u32] }],
        "fat_mirror_consistent": true,
        "fat_mirror_hashes": [""],
        "fat_mirror_mismatch_samples": null,
        "report_hash": "",
        "signature": null,
        "public_key_hint": null
    });
    fs::write(&journal_path, serde_json::to_vec_pretty(&report_json).unwrap()).unwrap();

    // Write keys
    fs::write(&sk_path, sk_hex.as_bytes()).unwrap();
    fs::write(&pk_path, pk_hex.as_bytes()).unwrap();

    // Sign
    fsck_cli(&vec![
        "sign-journal".to_string(),
        journal_path.to_string_lossy().to_string(),
        "--key".to_string(),
        sk_path.to_string_lossy().to_string(),
    ]).await.expect("sign-journal should succeed");

    // Verify
    fsck_cli(&vec![
        "verify-journal".to_string(),
        journal_path.to_string_lossy().to_string(),
        "--pub".to_string(),
        pk_path.to_string_lossy().to_string(),
    ]).await.expect("verify-journal should succeed");
}

#[tokio::test]
#[cfg(feature = "updates")]
async fn fsck_journal_verify_detects_tamper() {
    let dir = tempdir().unwrap();
    let journal_path = dir.path().join("report.json");
    let (sk_hex, pk_hex) = make_keypair_hex();
    let sk_path = dir.path().join("ed25519.sk");
    let pk_path = dir.path().join("ed25519.pk");

    // Write initial report
    let report_json = serde_json::json!({
        "device": "/dev/mock0p1",
        "filesystem": "FAT16",
        "files_scanned": 5u64,
        "cross_links": 1u64,
        "lost_clusters": [4u32],
        "actions_proposed": [{ "action": "free_clusters", "clusters": [4u32] }],
        "fat_mirror_consistent": true,
        "fat_mirror_hashes": ["deadbeef"],
        "fat_mirror_mismatch_samples": null,
        "report_hash": "",
        "signature": null,
        "public_key_hint": null
    });
    fs::write(&journal_path, serde_json::to_vec_pretty(&report_json).unwrap()).unwrap();
    fs::write(&sk_path, sk_hex.as_bytes()).unwrap();
    fs::write(&pk_path, pk_hex.as_bytes()).unwrap();

    // Sign
    fsck_cli(&vec![
        "sign-journal".to_string(),
        journal_path.to_string_lossy().to_string(),
        "--key".to_string(),
        sk_path.to_string_lossy().to_string(),
    ]).await.expect("sign-journal should succeed");

    // Tamper with the journal: change lost_clusters
    let mut tampered = fs::read_to_string(&journal_path).unwrap();
    tampered = tampered.replace("[\n    4\n  ]", "[\n    5\n  ]");
    let mut f = fs::OpenOptions::new().write(true).truncate(true).open(&journal_path).unwrap();
    f.write_all(tampered.as_bytes()).unwrap();

    // Verify should fail due to hash mismatch or signature failure
    let res = fsck_cli(&vec![
        "verify-journal".to_string(),
        journal_path.to_string_lossy().to_string(),
        "--pub".to_string(),
        pk_path.to_string_lossy().to_string(),
    ]).await;
    assert!(res.is_err(), "verify-journal should detect tampering");
}


