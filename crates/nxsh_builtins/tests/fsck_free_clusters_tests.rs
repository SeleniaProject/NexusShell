// fsck apply-journal commit tests for freeing clusters on FAT32
// Create a FAT32 image, allocate clusters by writing a file, then free one cluster via journal.

use nxsh_builtins::fsck::fsck_cli;

#[cfg(unix)]
use nxsh_builtins::mkfs::mkfs_cli;

#[cfg(unix)]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(unix)]
fn read_fat32_entry(image_path: &str, cluster: u32) -> u32 {
    let mut f = std::fs::OpenOptions::new().read(true).open(image_path).unwrap();
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).unwrap();
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;
    let off = base0 + (cluster as u64) * 4;
    assert!(off < base0 + fat_bytes);
    f.seek(SeekFrom::Start(off)).unwrap();
    let mut entry = [0u8; 4];
    f.read_exact(&mut entry).unwrap();
    u32::from_le_bytes(entry)
}

#[cfg(unix)]
fn find_allocated_cluster(image_path: &str) -> Option<u32> {
    let mut f = std::fs::OpenOptions::new().read(true).open(image_path).unwrap();
    let mut bpb = [0u8; 512];
    use std::cmp::min;
    use sha2::{Digest, Sha256};
    f.read_exact(&mut bpb).unwrap();
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;
    let mut offset = 2u64 * 4; // start at cluster 2
    let mut buf = vec![0u8; 64 * 1024];
    while offset < fat_bytes {
        let to_read = min(fat_bytes - offset, buf.len() as u64) as usize;
        use std::io::Seek;
        f.seek(SeekFrom::Start(base0 + offset)).unwrap();
        let n = f.read(&mut buf[..to_read]).unwrap();
        if n == 0 { break; }
        for i in (0..n).step_by(4) {
            if i + 4 <= n {
                let val = u32::from_le_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
                if val != 0 { // allocated or EOC
                    let cl = (offset / 4) as u32 + (i as u32 / 4);
                    if cl >= 2 { return Some(cl); }
                }
            }
        }
        offset += n as u64;
    }
    None
}

#[cfg(unix)]
#[tokio::test]
async fn fsck_apply_free_clusters_commits_on_shadow() {
    // Create temp FAT32 image
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("fsck_free.img");
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&img_path)
            .unwrap();
        file.set_len(16 * 1024 * 1024).unwrap();
    }

    // Format FAT32
    mkfs_cli(&vec!["-t".into(), "fat32".into(), img_path.to_string_lossy().to_string()])
        .await
        .expect("mkfs should succeed");

    // Allocate clusters by writing a file using fatfs
    {
        use fscommon::BufStream;
        use fatfs::{FileSystem, FsOptions};
        let f = std::fs::OpenOptions::new().read(true).write(true).open(&img_path).unwrap();
        let buf = BufStream::new(f);
        let fs = FileSystem::new(buf, FsOptions::new()).unwrap();
        let root = fs.root_dir();
        let mut file = root.create_file("big.bin").unwrap();
        let data = vec![0x7Au8; 128 * 1024]; // 128 KiB should allocate a few clusters
        file.write_all(&data).unwrap();
        file.flush().unwrap();
    }

    // Find an allocated cluster
    let cl = find_allocated_cluster(&img_path.to_string_lossy()).expect("allocated cluster not found");
    assert!(cl >= 2);
    let before = read_fat32_entry(&img_path.to_string_lossy(), cl);
    assert_ne!(before, 0);

    // Prepare journal to free this cluster
    let journal = serde_json::json!({
        "device": img_path.to_string_lossy(),
        "filesystem": "FAT32",
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

    // Apply commit on shadow image
    fsck_cli(&vec![
        "apply-journal".into(),
        journal_path.to_string_lossy().to_string(),
        "--shadow".into(),
        img_path.to_string_lossy().to_string(),
        "--commit".into(),
    ])
    .await
    .expect("apply-journal --commit should succeed");

    // The FAT entry should become zero
    let after = read_fat32_entry(&img_path.to_string_lossy(), cl);
    assert_eq!(after, 0, "cluster entry should be cleared");
}

#[cfg(not(unix))]
#[tokio::test]
async fn fsck_free_clusters_commit_not_supported_on_non_unix() {
    let res =     fsck_cli(&["apply-journal".into(), "dummy.json".into(), "--commit".into()]);
    assert!(res.is_err());
}


