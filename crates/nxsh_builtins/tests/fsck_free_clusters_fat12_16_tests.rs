// Validate FreeClusters commit on FAT12 and FAT16 by formatting small images
// and directly clearing an allocated cluster via fsck journal commit.

use nxsh_builtins::fsck::fsck_cli;

#[cfg(unix)]
use nxsh_builtins::mkfs::mkfs_cli;

#[cfg(unix)]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(unix)]
fn read_fat_entry_generic(image_path: &str, cl: u32, entry_bytes: u8) -> u32 {
    let mut f = std::fs::OpenOptions::new().read(true).open(image_path).unwrap();
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).unwrap();
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;
    let off = base0 + (cl as u64) * (entry_bytes as u64);
    assert!(off < base0 + fat_bytes);
    f.seek(SeekFrom::Start(off)).unwrap();
    let mut entry = [0u8; 4];
    f.read_exact(&mut entry[..entry_bytes as usize]).unwrap();
    u32::from_le_bytes(entry)
}

#[cfg(unix)]
fn allocate_one_cluster_by_hand(image_path: &str, entry_bytes: u8) -> u32 {
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(image_path).unwrap();
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).unwrap();
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;
    // pick cluster 3
    let cl = 3u32;
    let off = base0 + (cl as u64) * (entry_bytes as u64);
    use std::cmp::min;
    for fat_idx in 0..2u64 {
        let dst_off = base0 + fat_idx * fat_bytes + (cl as u64) * (entry_bytes as u64);
        f.seek(SeekFrom::Start(dst_off)).unwrap();
        match entry_bytes {
            2 => { f.write_all(&[0xFF, 0xFF]).unwrap(); }
            4 => { f.write_all(&[0xFF, 0xFF, 0x0F, 0x00]).unwrap(); }
            _ => { /* 12-bit handled via fsck later; we just set both bytes to non-zero */ f.write_all(&[0xFF, 0x0F]).unwrap(); }
        }
    }
    cl
}

#[cfg(unix)]
async fn run_free_cluster_case(ftype: &str, entry_bytes: u8) {
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join(format!("fsck_free_{}.img", ftype));
    {
        let file = std::fs::OpenOptions::new().create(true).read(true).write(true).truncate(true).open(&img_path).unwrap();
        file.set_len(2 * 1024 * 1024).unwrap();
    }
    mkfs_cli(&vec!["-t".into(), ftype.into(), img_path.to_string_lossy().to_string()]).await.expect("mkfs should succeed");

    let cl = allocate_one_cluster_by_hand(&img_path.to_string_lossy(), entry_bytes);
    let before = read_fat_entry_generic(&img_path.to_string_lossy(), cl, entry_bytes);
    assert_ne!(before & 0xFFFF, 0);

    let journal = serde_json::json!({
        "device": img_path.to_string_lossy(),
        "filesystem": ftype.to_uppercase(),
        "files_scanned": 0u64,
        "cross_links": 0u64,
        "lost_clusters": [ cl ],
        "actions_proposed": [ { "action": "free_clusters", "clusters": [ cl ] } ],
        "fat_mirror_consistent": true,
        "fat_mirror_hashes": [""],
        "fat_mirror_mismatch_samples": null,
        "report_hash": "",
        "signature": null,
        "public_key_hint": null
    });
    let journal_path = dir.path().join("report_free.json");
    std::fs::write(&journal_path, serde_json::to_vec_pretty(&journal).unwrap()).unwrap();

    fsck_cli(&vec!["apply-journal".into(), journal_path.to_string_lossy().to_string(), "--shadow".into(), img_path.to_string_lossy().to_string(), "--commit".into()]).await.expect("apply commit");

    let after = read_fat_entry_generic(&img_path.to_string_lossy(), cl, entry_bytes);
    assert_eq!(after & 0xFFFF, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn fsck_free_cluster_fat16() { run_free_cluster_case("fat16", 2).await; }

#[cfg(unix)]
#[tokio::test]
async fn fsck_free_cluster_fat12() { run_free_cluster_case("fat12", 2).await; }

#[cfg(not(unix))]
#[tokio::test]
async fn fsck_free_clusters_commit_not_supported_on_non_unix_fat12_16() {
    let res = fsck_cli(&vec!["apply-journal".into(), "dummy.json".into(), "--commit".into()]).await;
    assert!(res.is_err());
}


