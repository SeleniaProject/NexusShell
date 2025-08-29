// FAT12/16 mirror synchronization commit tests
// Format small images as FAT12 and FAT16, corrupt FAT1, apply SyncFatMirrors from FAT0, verify hashes.

use nxsh_builtins::fsck::fsck_cli;

#[cfg(unix)]
use nxsh_builtins::mkfs::mkfs_cli;

#[cfg(unix)]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(unix)]
fn fat_hash_pair(image_path: &str) -> (String, String) {
    use sha2::{Digest, Sha256};
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .open(image_path)
        .unwrap();
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).unwrap();
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let nfats = bpb[16] as u64;
    assert!(nfats >= 2, "need at least two FATs");
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;

    let mut hash_fat = |idx: u64| -> String {
        let mut hasher = Sha256::new();
        let mut remaining = fat_bytes;
        let mut buf = vec![0u8; 64 * 1024];
        let start = base0 + idx * fat_bytes;
        f.seek(SeekFrom::Start(start)).unwrap();
        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
            let n = f.read(&mut buf[..to_read]).unwrap();
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            remaining -= n as u64;
        }
        hex::encode(hasher.finalize())
    };

    (hash_fat(0), hash_fat(1))
}

#[cfg(unix)]
async fn run_sync_case(ftype: &str) {
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join(format!("fsck_sync_{}.img", ftype));
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&img_path)
            .unwrap();
        file.set_len(4 * 1024 * 1024).unwrap();
    }
    // Format as requested FAT type
    mkfs_cli(&vec![
        "-t".into(),
        ftype.into(),
        img_path.to_string_lossy().to_string(),
    ])
    .expect("mkfs");

    // Corrupt second FAT
    {
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&img_path)
            .unwrap();
        let mut bpb = [0u8; 512];
        f.read_exact(&mut bpb).unwrap();
        let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
        let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
        let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
        let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
        let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
        let base0 = rsvd * bps;
        let fat1_off = base0 + fat_bytes; // second FAT
        f.seek(SeekFrom::Start(fat1_off + 2048)).unwrap();
        f.write_all(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        f.flush().unwrap();
    }

    let (h0_before, h1_before) = fat_hash_pair(&img_path.to_string_lossy());
    assert_ne!(h0_before, h1_before, "mirrors should differ before sync");

    // Journal proposing sync
    let journal = serde_json::json!({
        "device": img_path.to_string_lossy(),
        "filesystem": ftype.to_uppercase(),
        "files_scanned": 0u64,
        "cross_links": 0u64,
        "lost_clusters": [],
        "actions_proposed": [ { "action": "sync_fat_mirrors", "source_fat": 0u8 } ],
        "fat_mirror_consistent": false,
        "fat_mirror_hashes": [""],
        "fat_mirror_mismatch_samples": [],
        "report_hash": "",
        "signature": null,
        "public_key_hint": null
    });
    let journal_path = dir.path().join("report.json");
    std::fs::write(&journal_path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();

    fsck_cli(&[
        "apply-journal".into(),
        "dummy.json".into(),
        "--commit".into(),
    ])
    .expect("apply sync");

    let (h0_after, h1_after) = fat_hash_pair(&img_path.to_string_lossy());
    assert_eq!(h0_after, h1_after, "mirrors should match after sync");
}

#[cfg(unix)]
#[tokio::test]
async fn fsck_sync_fat12() {
    run_sync_case("fat12");
}

#[cfg(unix)]
#[tokio::test]
async fn fsck_sync_fat16() {
    run_sync_case("fat16");
}

#[cfg(not(unix))]
#[tokio::test]
async fn fsck_sync_not_supported_on_non_unix() {
    let res = fsck_cli(&[
        "apply-journal".into(),
        "dummy.json".into(),
        "--commit".into(),
    ]);
    assert!(res.is_err());
}
