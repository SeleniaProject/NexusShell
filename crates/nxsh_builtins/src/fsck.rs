#[cfg(unix)]
fn apply_broken_chain_end(image_path: &str, start_cluster: u32) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(image_path)
        .with_context(|| format!("fsck: cannot open shadow image {} for write", image_path))?;
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;

    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u32;
    let spc = bpb[13] as u32;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u32;
    let nfats = bpb[16] as u32;
    let root_ent_cnt = u16::from_le_bytes([bpb[17], bpb[18]]) as u32;
    let tot_sec_16 = u16::from_le_bytes([bpb[19], bpb[20]]) as u32;
    let tot_sec_32 = u32::from_le_bytes([bpb[32], bpb[33], bpb[34], bpb[35]]);
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u32;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]);
    let fat_sz = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 };
    let total_sectors = if tot_sec_16 != 0 { tot_sec_16 } else { tot_sec_32 };

    let root_dir_sectors = ((root_ent_cnt * 32) + (bps - 1)) / bps;
    let data_sectors = total_sectors - (rsvd + (nfats * fat_sz) + root_dir_sectors);
    let cluster_count = data_sectors / spc;
    let fat_type = if cluster_count < 4085 { 12 } else if cluster_count < 65525 { 16 } else { 32 };

    // helper: read FAT entry
    let read_entry = |f: &mut std::fs::File, fat_base: u64, c: u32| -> Result<u32> {
        match fat_type {
            12 => {
                let n = c as u64;
                let byte_index = fat_base + (n + (n / 2)) as u64;
                let mut pair = [0u8; 2];
                f.seek(SeekFrom::Start(byte_index))?;
                f.read_exact(&mut pair)?;
                let val = if c % 2 == 0 {
                    u16::from_le_bytes([pair[0], pair[1] & 0x0F]) as u32
                } else {
                    (u16::from_le_bytes([pair[0], pair[1]]) >> 4) as u32
                };
                Ok(val)
            }
            16 => {
                let off = fat_base + ((c as u64) * 2);
                let mut b = [0u8; 2];
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut b)?;
                Ok(u16::from_le_bytes(b) as u32)
            }
            32 => {
                let off = fat_base + ((c as u64) * 4);
                let mut b = [0u8; 4];
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut b)?;
                Ok(u32::from_le_bytes(b) & 0x0FFFFFFF)
            }
            _ => Err(anyhow!("fsck: unsupported FAT type")),
        }
    };

    // helper: write EOC to entry
    let write_eoc = |f: &mut std::fs::File, fat_base: u64, c: u32| -> Result<()> {
        match fat_type {
            12 => {
                let n = c as u64;
                let byte_index = fat_base + (n + (n / 2)) as u64;
                let mut pair = [0u8; 2];
                f.seek(SeekFrom::Start(byte_index))?;
                f.read_exact(&mut pair)?;
                if c % 2 == 0 {
                    pair[0] = 0xFF;
                    pair[1] = (pair[1] & 0xF0) | 0x0F;
                } else {
                    pair[0] = (pair[0] & 0x0F) | 0xF0;
                    pair[1] = 0xFF;
                }
                f.seek(SeekFrom::Start(byte_index))?;
                f.write_all(&pair)?;
                Ok(())
            }
            16 => {
                let off = fat_base + ((c as u64) * 2);
                let val = 0xFFFFu16.to_le_bytes();
                f.seek(SeekFrom::Start(off))?;
                f.write_all(&val)?;
                Ok(())
            }
            32 => {
                let off = fat_base + ((c as u64) * 4);
                let mut val = 0x0FFFFFFFu32.to_le_bytes();
                f.seek(SeekFrom::Start(off))?;
                f.write_all(&val)?;
                Ok(())
            }
            _ => Err(anyhow!("fsck: unsupported FAT type")),
        }
    };

    let fat0_offset = (rsvd * bps) as u64;

    // traverse chain and find the first invalid link; set previous to EOC
    let mut prev = start_cluster;
    let mut cur = start_cluster;
    loop {
        let next = read_entry(&mut f, fat0_offset, cur)?;
        let invalid = match fat_type {
            12 => next >= 0xFF0 && next < 0xFF8,
            16 => next >= 0xFFF0 && next < 0xFFF8,
            32 => (next >= 0x0FFFFFF0 && next < 0x0FFFFFF8) || next == 0,
            _ => false,
        };
        if next == 0 || invalid { // hole or reserved/bad before EOC
            write_eoc(&mut f, fat0_offset, prev)?;
            break;
        }
        if match fat_type { 12 => next >= 0xFF8, 16 => next >= 0xFFF8, 32 => next >= 0x0FFFFFF8, _ => false } {
            // already proper EOC
            break;
        }
        prev = cur;
        cur = next;
    }

    // Mirror to other FATs
    for i in 1..nfats {
        let src = 0u8;
        sync_fat_mirrors(image_path, src)?;
        let _ = i; // already synced wholesale
    }
    Ok(())
}

#[cfg(unix)]
fn apply_cross_link_break(image_path: &str, cluster: u32) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(image_path)
        .with_context(|| format!("fsck: cannot open shadow image {} for write", image_path))?;
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;

    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u32;
    let spc = bpb[13] as u32;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u32;
    let nfats = bpb[16] as u32;
    let root_ent_cnt = u16::from_le_bytes([bpb[17], bpb[18]]) as u32;
    let tot_sec_16 = u16::from_le_bytes([bpb[19], bpb[20]]) as u32;
    let tot_sec_32 = u32::from_le_bytes([bpb[32], bpb[33], bpb[34], bpb[35]]);
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u32;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]);
    let fat_sz = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 };
    let total_sectors = if tot_sec_16 != 0 { tot_sec_16 } else { tot_sec_32 };

    let root_dir_sectors = ((root_ent_cnt * 32) + (bps - 1)) / bps;
    let data_sectors = total_sectors - (rsvd + (nfats * fat_sz) + root_dir_sectors);
    let cluster_count = data_sectors / spc;
    let fat_type = if cluster_count < 4085 { 12 } else if cluster_count < 65525 { 16 } else { 32 };
    let fat0_offset = (rsvd * bps) as u64;

    // helper to write zero (free) entry
    let write_free = |f: &mut std::fs::File, fat_base: u64, c: u32| -> Result<()> {
        match fat_type {
            12 => {
                let n = c as u64;
                let byte_index = fat_base + (n + (n / 2)) as u64;
                let mut pair = [0u8; 2];
                f.seek(SeekFrom::Start(byte_index))?;
                f.read_exact(&mut pair)?;
                if c % 2 == 0 {
                    pair[0] = 0x00;
                    pair[1] &= 0xF0;
                } else {
                    pair[0] &= 0x0F;
                    pair[1] = 0x00;
                }
                f.seek(SeekFrom::Start(byte_index))?;
                f.write_all(&pair)?;
                Ok(())
            }
            16 => {
                let off = fat_base + ((c as u64) * 2);
                let val = 0u16.to_le_bytes();
                f.seek(SeekFrom::Start(off))?;
                f.write_all(&val)?;
                Ok(())
            }
            32 => {
                let off = fat_base + ((c as u64) * 4);
                let val = (0u32).to_le_bytes();
                f.seek(SeekFrom::Start(off))?;
                f.write_all(&val)?;
                Ok(())
            }
            _ => Err(anyhow!("fsck: unsupported FAT type")),
        }
    };

    write_free(&mut f, fat0_offset, cluster)?;
    // mirror to others
    sync_fat_mirrors(image_path, 0)?;
    Ok(())
}
// `fsck` builtin — filesystem consistency checker.
//
// Supported filesystems: FAT12/16/32 (via `fatfs` crate).
//
// Usage:
//     fsck DEVICE
//     fsck -a DEVICE    # auto-repair (currently read-only, reports issues)
//
// Behaviour:
// * Performs a read-only scan of FAT tables and directory trees.
// * Detects orphaned (lost) clusters, cross-linked clusters, and directory
//   entry inconsistencies. Results are printed as a report.
// * `-a` flag is accepted for compatibility; repair functionality is a TODO
//   and will be implemented with journalling once write-back safety guarantees
//   are in place.
//
// Platform: Unix-like systems. On non-Unix platforms the command gracefully
// degrades with an informative message.

use anyhow::{anyhow, Context, Result};

#[cfg(unix)]
use fatfs::{FatType, FileSystem, FsOptions, ReadWriteSeek, FileAttributes};
#[cfg(unix)]
use fscommon::BufStream;
#[cfg(unix)]
use std::{collections::HashSet, fs::OpenOptions, path::Path};
use serde::{Serialize, Deserialize};
#[cfg(unix)]
use std::{fs, io::{Read, Seek, SeekFrom}};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};

pub async fn fsck_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("fsck: missing operand (DEVICE | apply-journal | create-shadow | sign-journal | verify-journal)"));
    }

    // Subcommands: apply-journal <report.json>
    if args[0] == "apply-journal" {
        if args.len() < 2 { return Err(anyhow!("fsck: apply-journal requires path to report.json")); }
        // Optional: --shadow <image>
        let mut shadow: Option<String> = None;
        let mut commit: bool = false;
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--shadow" => { i += 1; if i < args.len() { shadow = Some(args[i].clone()); } },
                "--commit" => { commit = true; },
                other => return Err(anyhow!(format!("fsck: unknown option to apply-journal: {}", other))),
            }
            i += 1;
        }
        if commit { return apply_fsck_journal_commit(&args[1], shadow.as_deref()).await; }
        return apply_fsck_journal_with_shadow(&args[1], shadow.as_deref()).await;
    }

    // Subcommands: create-shadow <DEVICE> [OUTPUT_IMG]
    if args[0] == "create-shadow" {
        if args.len() < 2 { return Err(anyhow!("fsck: create-shadow requires DEVICE")); }
        let output = if args.len() >= 3 { Some(args[2].clone()) } else { None };
        return create_shadow_image(&args[1], output.as_deref());
    }

    // Subcommand: sign-journal <report.json> --key <ed25519_priv>
    if args[0] == "sign-journal" {
        if args.len() < 2 { return Err(anyhow!("fsck: sign-journal requires path to report.json")); }
        let mut key_path: Option<&str> = None;
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--key" => { i += 1; if i < args.len() { key_path = Some(&args[i]); } },
                other => return Err(anyhow!(format!("fsck: unknown option to sign-journal: {}", other))),
            }
            i += 1;
        }
        let key_path = key_path.ok_or_else(|| anyhow!("fsck: sign-journal requires --key <ed25519_priv>"))?;
        return sign_fsck_journal(&args[1], key_path).await;
    }

    // Subcommand: verify-journal <report.json> --pub <ed25519_pub>
    if args[0] == "verify-journal" {
        if args.len() < 2 { return Err(anyhow!("fsck: verify-journal requires path to report.json")); }
        let mut pub_path: Option<&str> = None;
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--pub" => { i += 1; if i < args.len() { pub_path = Some(&args[i]); } },
                other => return Err(anyhow!(format!("fsck: unknown option to verify-journal: {}", other))),
            }
            i += 1;
        }
        let pub_path = pub_path.ok_or_else(|| anyhow!("fsck: verify-journal requires --pub <ed25519_pub>"))?;
        return verify_fsck_journal(&args[1], pub_path).await;
    }

    let mut _auto = false; // track auto flag for future repair implementation
    let mut device: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "-a" | "--auto" => _auto = true,
            _ => device = Some(arg.clone()),
        }
    }
    let _dev = device.ok_or_else(|| anyhow!("fsck: missing DEVICE"))?; // placeholder until direct low-level access used

    #[cfg(unix)]
    {
    run_fat_check(&_dev, _auto)?;
    }
    #[cfg(not(unix))]
    {
        println!("fsck: FAT filesystem check is only supported on Unix-like systems");
        println!("fsck: On this platform, use native tools for filesystem checking");
    }

    Ok(())
}

#[cfg(unix)]
fn run_fat_check(device: &str, auto: bool) -> Result<()> {
    let path = Path::new(device);
    let file = OpenOptions::new().read(true).open(path)
        .with_context(|| format!("fsck: failed to open {device} for read"))?;
    let buf_stream = BufStream::new(file);
    let fs = FileSystem::new(buf_stream, FsOptions::new())
        .context("fsck: failed to mount FAT filesystem")?;

    println!("fsck: checking FAT{} filesystem on {}", 
        match fs.fat_type() {
            FatType::Fat12 => "12",
            FatType::Fat16 => "16", 
            FatType::Fat32 => "32",
        }, device);

    if auto { println!("fsck: auto-repair mode (read-only analysis)"); }

    // FATメタデータの読み出し（厳密検査用）
    let fat_type_u8 = match fs.fat_type() { FatType::Fat12=>12, FatType::Fat16=>16, FatType::Fat32=>32 };
    let fat_entries = read_fat_table(device, fat_type_u8)?;
    let max_cluster_num: u32 = (fat_entries.len() as u32) + 1; // クラスタ番号範囲 [2, len+1]

    // 参照マークとクロスリンク/破損チェイン収集
    let root_dir = fs.root_dir();
    let mut used_clusters: HashSet<u32> = HashSet::new();
    let mut cross_linked_clusters_traversal: HashSet<u32> = HashSet::new();
    let mut broken_chains_traversal: Vec<u32> = Vec::new();
    let mut scanned_files = 0;

    // ディレクトリ木を辿り、各エントリのFATチェインを厳密に追跡
    traverse_dir_with_fat(
        &root_dir,
        &fat_entries,
        fat_type_u8,
        &mut used_clusters,
        &mut cross_linked_clusters_traversal,
        &mut scanned_files,
        &mut broken_chains_traversal,
        max_cluster_num,
    )?;

    // Enhanced lost cluster detection with FAT chain analysis
    let mut lost_clusters: Vec<u32> = Vec::new();
    let mut cross_linked_clusters: HashSet<u32> = HashSet::new();
    
    // Perform comprehensive FAT chain analysis
    let fat_analysis = analyze_fat_chains(device, &fs)?;
    
    // Detect lost clusters: allocated in FAT but not referenced by directory tree
    for cluster in fat_analysis.allocated_clusters {
        if !used_clusters.contains(&cluster) {
            lost_clusters.push(cluster);
            if lost_clusters.len() >= 1024 { break; }
        }
    }
    
    // Update cross-links count from detailed analysis
    cross_linked_clusters = fat_analysis.cross_linked_clusters
        .union(&cross_linked_clusters_traversal)
        .copied()
        .collect();
    let cross_links = cross_linked_clusters.len();

    // 破損チェインの統合
    let mut broken_chains: Vec<u32> = fat_analysis.broken_chains.clone();
    broken_chains.extend(broken_chains_traversal);
    broken_chains.sort_unstable();
    broken_chains.dedup();
    
    // Compute FAT mirror consistency and hashes
    let (fat_mirror_consistent, fat_mirror_hashes, fat_mirror_mismatch_samples) =
        compute_fat_mirror_consistency(device).unwrap_or((true, Vec::new(), None));

    // Build structured report (hash computed after fields assembled)
    let mut actions: Vec<FsckAction> = Vec::new();
    if !lost_clusters.is_empty() { actions.push(FsckAction::FreeClusters { clusters: lost_clusters.clone() }); }
    if !fat_mirror_consistent { actions.push(FsckAction::SyncFatMirrors { source_fat: 0 }); }
    // 破損チェイン修復候補
    for &sc in &fat_analysis.broken_chains {
        actions.push(FsckAction::BrokenChainEnd { start_cluster: sc });
    }
    // クロスリンク切断候補（ターゲットクラスタの切断）
    for &c in &fat_analysis.cross_link_targets {
        actions.push(FsckAction::CrossLinkBreak { cluster: c });
    }

    let mut report = FsckReport {
        device: device.to_string(),
        filesystem: format!("FAT{}", match fs.fat_type() { FatType::Fat12=>12, FatType::Fat16=>16, FatType::Fat32=>32 }),
        files_scanned: scanned_files as u64,
        cross_links,
        lost_clusters: lost_clusters.clone(),
        actions_proposed: actions,
        fat_mirror_consistent,
        fat_mirror_hashes,
        fat_mirror_mismatch_samples,
        broken_chains,
        report_hash: String::new(),
    };

    // Compute and set stable report hash
    report.report_hash = compute_report_hash(&report);

    println!("fsck: checked {} files/directories", report.files_scanned);
    if report.cross_links == 0 && report.lost_clusters.is_empty() {
        println!("fsck: CLEAN");
    } else {
        if report.cross_links > 0 { println!("fsck: warning  E{} cross-linked cluster(s) detected", report.cross_links); }
        if !report.lost_clusters.is_empty() { println!("fsck: warning  E{} lost cluster(s) (showing up to 10) {:?}", report.lost_clusters.len(), &report.lost_clusters[..std::cmp::min(10, report.lost_clusters.len())]); }
        if !report.fat_mirror_hashes.is_empty() {
            println!("fsck: FAT mirror consistency: {}", if report.fat_mirror_consistent { "OK" } else { "MISMATCH" });
            if !report.fat_mirror_consistent {
                if let Some(samples) = &report.fat_mirror_mismatch_samples { println!("fsck: FAT mismatch sample offsets: {:?}", samples); }
            }
        }

        // Always persist a journal to allow manual review or later application
        let journal_dir = Path::new(".nxsh");
        let _ = fs::create_dir_all(journal_dir);
        let journal_path = journal_dir.join("fsck_report.json");
        match fs::write(&journal_path, serde_json::to_vec_pretty(&report).unwrap_or_default()) {
            Ok(_) => println!("fsck: report written to {}", journal_path.display()),
            Err(e) => eprintln!("fsck: failed to write report: {}", e),
        }

        if auto {
            println!("fsck: auto (-a) requested — safe mode: generated actions in journal, no write-back performed");
            println!("fsck: review {} and re-run with dedicated repair tool to apply changes on a shadow copy", journal_path.display());
        } else {
            println!("fsck: issues found; run with -a to emit a repair journal");
        }
    }

    Ok(())
}

#[cfg(unix)]
fn traverse_dir_with_fat<D: ReadWriteSeek + 'static>(
    dir: &fatfs::Dir<D>,
    fat_entries: &[u32],
    fat_type: u8,
    used: &mut HashSet<u32>,
    cross_linked: &mut HashSet<u32>,
    files: &mut usize,
    broken: &mut Vec<u32>,
    max_cluster_num: u32,
) -> Result<()> {
    for entry in dir.iter() {
        let entry = entry?;
        *files += 1;

        let start_cluster = entry.first_cluster();
        if start_cluster >= 2 {
            let mut current = start_cluster;
            let mut local_seen: HashSet<u32> = HashSet::new();
            let mut loop_detected = false;

            while current >= 2 {
                if current > max_cluster_num { broken.push(start_cluster); break; }
                if !local_seen.insert(current) { loop_detected = true; break; }

                if !used.insert(current) { cross_linked.insert(current); }

                match get_next_cluster(current, fat_entries, fat_type) {
                    Some(next) => { current = next; }
                    None => { break; } // EOC or invalid
                }
            }
            if loop_detected { broken.push(start_cluster); }
        }

        if entry.attributes().contains(FileAttributes::DIRECTORY) {
            let sub = entry.to_dir();
            traverse_dir_with_fat(&sub, fat_entries, fat_type, used, cross_linked, files, broken, max_cluster_num)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
#[derive(Debug)]
struct FatAnalysis {
    allocated_clusters: HashSet<u32>,
    cross_linked_clusters: HashSet<u32>,
    broken_chains: Vec<u32>,
    fat_type: u8,
    cross_link_targets: Vec<u32>,
}

#[cfg(unix)]
fn analyze_fat_chains(device: &str, fs: &FileSystem<BufStream<std::fs::File>>) -> Result<FatAnalysis> {
    let mut allocated_clusters = HashSet::new();
    let mut cross_linked_clusters = HashSet::new();
    let mut broken_chains = Vec::new();
    let mut cross_link_targets = Vec::new();
    
    // Determine FAT type
    let fat_type = match fs.fat_type() {
        FatType::Fat12 => 12,
        FatType::Fat16 => 16,
        FatType::Fat32 => 32,
    };
    
    // Read FAT table directly for comprehensive analysis
    let fat_entries = read_fat_table(device, fat_type)?;
    
    // Analyze each cluster entry
    for (cluster, &entry) in fat_entries.iter().enumerate() {
        let cluster = cluster as u32 + 2; // FAT entries start at cluster 2
        
        if is_allocated_entry(entry, fat_type) {
            allocated_clusters.insert(cluster);
            
            // Check for broken chains (pointing to invalid clusters)
            if is_bad_cluster_entry(entry, fat_type) {
                broken_chains.push(cluster);
            }
        }
    }
    
    // Detect cross-links by following chains and checking for convergence
    let mut cluster_references: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    
    for &cluster in &allocated_clusters {
        if let Some(next) = get_next_cluster(cluster, &fat_entries, fat_type) {
            cluster_references.entry(next).or_default().push(cluster);
        }
    }
    
    // Clusters referenced by multiple predecessors are cross-linked
    for (cluster, refs) in cluster_references {
        if refs.len() > 1 {
            cross_linked_clusters.insert(cluster);
            for &ref_cluster in &refs {
                cross_linked_clusters.insert(ref_cluster);
            }
            cross_link_targets.push(cluster);
        }
    }
    
    Ok(FatAnalysis {
        allocated_clusters,
        cross_linked_clusters,
        broken_chains,
    fat_type,
    cross_link_targets,
    })
}

#[cfg(unix)]
fn read_fat_table(device: &str, fat_type: u8) -> Result<Vec<u32>> {
    let mut file = std::fs::OpenOptions::new().read(true).open(device)
        .with_context(|| format!("fsck: cannot open {} for FAT analysis", device))?;
    
    // Read boot sector to get FAT parameters
    let mut bpb = [0u8; 512];
    file.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;
    
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_sz = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 };
    
    let fat_offset = rsvd * bps;
    let fat_bytes = fat_sz * bps;
    
    // Read FAT data
    file.seek(SeekFrom::Start(fat_offset))?;
    let mut fat_data = vec![0u8; fat_bytes as usize];
    file.read_exact(&mut fat_data)?;
    
    // Parse FAT entries based on type
    let mut entries = Vec::new();
    match fat_type {
        12 => {
            // FAT12: 12-bit entries, packed 2 entries per 3 bytes
            for i in (0..fat_data.len()).step_by(3) {
                if i + 2 < fat_data.len() {
                    let bytes = [fat_data[i], fat_data[i + 1], fat_data[i + 2]];
                    let entry1 = u16::from_le_bytes([bytes[0], bytes[1] & 0x0F]) as u32;
                    let entry2 = (u16::from_le_bytes([bytes[1], bytes[2]]) >> 4) as u32;
                    entries.push(entry1);
                    entries.push(entry2);
                }
            }
        }
        16 => {
            // FAT16: 16-bit entries
            for chunk in fat_data.chunks_exact(2) {
                let entry = u16::from_le_bytes([chunk[0], chunk[1]]) as u32;
                entries.push(entry);
            }
        }
        32 => {
            // FAT32: 32-bit entries (only lower 28 bits used)
            for chunk in fat_data.chunks_exact(4) {
                let entry = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) & 0x0FFFFFFF;
                entries.push(entry);
            }
        }
        _ => return Err(anyhow!("fsck: unsupported FAT type: {}", fat_type)),
    }
    
    Ok(entries)
}

#[cfg(unix)]
fn is_allocated_entry(entry: u32, fat_type: u8) -> bool {
    match fat_type {
        12 => entry != 0 && entry < 0xFF0,
        16 => entry != 0 && entry < 0xFFF0,
        32 => entry != 0 && entry < 0x0FFFFFF0,
        _ => false,
    }
}

#[cfg(unix)]
fn is_bad_cluster_entry(entry: u32, fat_type: u8) -> bool {
    match fat_type {
        12 => entry == 0xFF7,
        16 => entry == 0xFFF7,
        32 => entry == 0x0FFFFFF7,
        _ => false,
    }
}

#[cfg(unix)]
fn get_next_cluster(cluster: u32, fat_entries: &[u32], fat_type: u8) -> Option<u32> {
    let index = (cluster - 2) as usize;
    if index >= fat_entries.len() {
        return None;
    }
    
    let entry = fat_entries[index];
    if is_end_of_chain(entry, fat_type) || is_bad_cluster_entry(entry, fat_type) {
        None
    } else if is_allocated_entry(entry, fat_type) {
        Some(entry)
    } else {
        None
    }
}

#[cfg(unix)]
fn is_end_of_chain(entry: u32, fat_type: u8) -> bool {
    match fat_type {
        12 => entry >= 0xFF8,
        16 => entry >= 0xFFF8,
        32 => entry >= 0x0FFFFFF8,
        _ => false,
    }
} 

#[derive(Debug, Serialize, Deserialize)]
struct FsckReport {
    device: String,
    filesystem: String,
    files_scanned: u64,
    cross_links: usize,
    lost_clusters: Vec<u32>,
    actions_proposed: Vec<FsckAction>,
    fat_mirror_consistent: bool,
    fat_mirror_hashes: Vec<String>,
    fat_mirror_mismatch_samples: Option<Vec<u64>>, // byte offsets relative to FAT start
    #[serde(default)]
    broken_chains: Vec<u32>,
    report_hash: String,
    signature: Option<String>, // base64(ed25519 signature over report_hash)
    public_key_hint: Option<String>, // hex(pubkey) or fingerprint for operator reference
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum FsckAction {
    FreeClusters { clusters: Vec<u32> },
    SyncFatMirrors { source_fat: u8 },
    /// 修復: FATチェーンの終端を強制的に修正する（broken chain）
    BrokenChainEnd { start_cluster: u32 },
    /// 修復: クロスリンクを切断し、クラスタを解放する
    CrossLinkBreak { cluster: u32 },
}

#[cfg(unix)]
fn compute_fat_mirror_consistency(device: &str) -> Result<(bool, Vec<String>, Option<Vec<u64>>)> {
    // Read BPB and compute FAT locations, then hash each FAT copy and compare a few sample windows
    let mut f = std::fs::OpenOptions::new().read(true).open(device)
        .with_context(|| format!("fsck: cannot open {} for read", device))?;
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let nfats = bpb[16] as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_sz = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let fat0 = rsvd * bps;

    let mut hashes = Vec::new();
    for i in 0..nfats {
        let start = fat0 + i * fat_sz;
        f.seek(SeekFrom::Start(start)).ok();
        let mut remaining = fat_sz;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
            let n = f.read(&mut buf[..to_read])?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
            remaining -= n as u64;
        }
        hashes.push(hex::encode(hasher.finalize()));
    }

    let all_equal = hashes.windows(2).all(|w| w[0] == w[1]);
    let mismatch_samples = if all_equal || nfats < 2 {
        None
    } else {
        // Sample first few KiB mismatches by naive comparison between FAT0 and FAT1
        let start0 = fat0;
        let start1 = fat0 + fat_sz;
        let mut off = 0u64;
        let mut samples = Vec::new();
        let mut buf0 = vec![0u8; 4096];
        let mut buf1 = vec![0u8; 4096];
        while off < fat_sz && samples.len() < 8 {
            f.seek(SeekFrom::Start(start0 + off)).ok();
            f.read_exact(&mut buf0).ok();
            f.seek(SeekFrom::Start(start1 + off)).ok();
            f.read_exact(&mut buf1).ok();
            if buf0 != buf1 { samples.push(off); }
            off += 4096;
        }
        Some(samples)
    };

    Ok((all_equal, hashes, mismatch_samples))
}

fn compute_report_hash(report: &FsckReport) -> String {
    let mut hasher = Sha256::new();
    hasher.update(report.device.as_bytes());
    hasher.update(report.filesystem.as_bytes());
    hasher.update(report.files_scanned.to_le_bytes());
    hasher.update(report.cross_links.to_le_bytes());
    for cl in &report.lost_clusters { hasher.update(cl.to_le_bytes()); }
    for h in &report.fat_mirror_hashes { hasher.update(h.as_bytes()); }
    for b in &report.broken_chains { hasher.update(b.to_le_bytes()); }
    // include actions (stable JSON)
    if let Ok(bytes) = serde_json::to_vec(&report.actions_proposed) {
        hasher.update(bytes);
    }
    let digest = hasher.finalize();
    hex::encode(digest)
}

// (single definition for all targets)

#[cfg(feature = "updates")]
async fn sign_fsck_journal(path: &str, key_path: &str) -> Result<()> {
    use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
    let data = std::fs::read(path).with_context(|| format!("fsck: cannot read journal {path}"))?;
    let mut report: FsckReport = serde_json::from_slice(&data).context("fsck: invalid journal format")?;
    // Ensure hash up-to-date
    report.report_hash = compute_report_hash(&report);
    // Load private key (raw hex or base64, 32 bytes)
    let key_bytes = std::fs::read_to_string(key_path)
    .with_context(|| format!("fsck: cannot read key {key_path}"))?;
    let key_bytes = key_bytes.trim();
    let sk_bytes: Vec<u8> = if let Ok(hex_bytes) = hex::decode(key_bytes) { hex_bytes } else {
        general_purpose::STANDARD.decode(key_bytes).context("fsck: invalid key (expected hex/base64 of 32 bytes)")?
    };
    if sk_bytes.len() != 32 { return Err(anyhow!("fsck: invalid key length (need 32 bytes)")); }
    let signing_key = SigningKey::from_bytes(sk_bytes.as_slice().try_into().unwrap());
    let verifying_key: VerifyingKey = (&signing_key).into();
    // Sign the stable hash string as bytes
    let sig = signing_key.sign(report.report_hash.as_bytes());
    report.signature = Some(general_purpose::STANDARD.encode(sig.to_bytes()));
    report.public_key_hint = Some(hex::encode(verifying_key.to_bytes()));
    // Persist back
    let out = serde_json::to_vec_pretty(&report)?;
    std::fs::write(path, out).with_context(|| format!("fsck: failed to write signed journal {path}"))?;
    println!("fsck: journal signed (pubhint={})", report.public_key_hint.as_deref().unwrap_or(""));
    Ok(())
}

#[cfg(not(feature = "updates"))]
async fn sign_fsck_journal(_path: &str, _key_path: &str) -> Result<()> {
    Err(anyhow!("fsck: sign-journal requires 'updates' feature (ed25519)"))
}

#[cfg(feature = "updates")]
async fn verify_fsck_journal(path: &str, pub_path: &str) -> Result<()> {
    use ed25519_dalek::{VerifyingKey, Signature, Verifier};
    let data = std::fs::read(path).with_context(|| format!("fsck: cannot read journal {path}"))?;
    let report: FsckReport = serde_json::from_slice(&data).context("fsck: invalid journal format")?;
    let hash = compute_report_hash(&report);
    if hash != report.report_hash { return Err(anyhow!("fsck: report hash mismatch (journal tampered?)")); }
    let sig_b64 = report.signature.as_ref().ok_or_else(|| anyhow!("fsck: journal has no signature"))?;
    let sig_bytes = general_purpose::STANDARD.decode(sig_b64).context("fsck: invalid base64 signature")?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|_| anyhow!("fsck: invalid signature format"))?;
    let pk_bytes_raw = std::fs::read_to_string(pub_path).with_context(|| format!("fsck: cannot read pubkey {pub_path}"))?;
    let pk_bytes_raw = pk_bytes_raw.trim();
    let pk_bytes: Vec<u8> = if let Ok(hex_bytes) = hex::decode(pk_bytes_raw) { hex_bytes } else { general_purpose::STANDARD.decode(pk_bytes_raw).context("fsck: invalid pubkey (expected hex/base64 of 32 bytes)")? };
    if pk_bytes.len() != 32 { return Err(anyhow!("fsck: invalid pubkey length (need 32 bytes)")); }
    let vk = VerifyingKey::from_bytes(pk_bytes.as_slice().try_into().unwrap())
        .map_err(|_| anyhow!("fsck: invalid ed25519 pubkey"))?;
    vk.verify(report.report_hash.as_bytes(), &sig)
        .map_err(|_| anyhow!("fsck: signature verification failed"))?;
    println!("fsck: signature OK (pub={})", hex::encode(vk.to_bytes()));
    Ok(())
}

#[cfg(not(feature = "updates"))]
async fn verify_fsck_journal(_path: &str, _pub_path: &str) -> Result<()> {
    Err(anyhow!("fsck: verify-journal requires 'updates' feature (ed25519)"))
}

#[cfg(unix)]
async fn apply_fsck_journal(path: &str) -> Result<()> {
    // Read and validate report; do not perform write-back. Show a dry-run plan instead.
    let data = std::fs::read(path).with_context(|| format!("fsck: cannot read journal {path}"))?;
    let report: FsckReport = serde_json::from_slice(&data).context("fsck: invalid journal format")?;
    println!("fsck: applying journal (dry-run) for {} [{}]", report.device, report.filesystem);
    if report.actions_proposed.is_empty() {
        println!("fsck: no actions to apply");
        return Ok(());
    }
    for action in &report.actions_proposed {
        match action {
            FsckAction::FreeClusters { clusters } => {
                println!(" - would free {} cluster(s) (first 16 shown): {:?}", clusters.len(), &clusters[..std::cmp::min(16, clusters.len())]);
            }
            FsckAction::SyncFatMirrors { source_fat } => {
                println!(" - would sync FAT mirrors from FAT{}", source_fat);
            }
            FsckAction::BrokenChainEnd { start_cluster } => {
                println!(" - would fix broken chain end starting at cluster {}", start_cluster);
            }
            FsckAction::CrossLinkBreak { cluster } => {
                println!(" - would break cross-link by freeing cluster {}", cluster);
            }
        }
    }
    println!("fsck: dry-run completed. Use dedicated repair tool to perform write-back on a shadow copy.");
    Ok(())
}

#[cfg(not(unix))]
async fn apply_fsck_journal(_path: &str) -> Result<()> {
    println!("fsck: apply-journal is not supported on this platform");
    Ok(())
}

#[cfg(unix)]
async fn apply_fsck_journal_with_shadow(path: &str, shadow_img: Option<&str>) -> Result<()> {
    // Load journal
    let data = std::fs::read(path).with_context(|| format!("fsck: cannot read journal {path}"))?;
    let report: FsckReport = serde_json::from_slice(&data).context("fsck: invalid journal format")?;

    println!("fsck: journal actions for device {} ({} actions)", report.device, report.actions_proposed.len());
    // If shadow specified, sanity-mount it
    if let Some(img) = shadow_img {
        let file = std::fs::OpenOptions::new().read(true).write(true).open(img)
            .with_context(|| format!("fsck: failed to open shadow image {}", img))?;
        let buf = fscommon::BufStream::new(file);
        let fs = FileSystem::new(buf, FsOptions::new())
            .context("fsck: failed to mount shadow image (FAT)")?;
        println!("fsck: shadow image {} mounted as FAT{}", img, match fs.fat_type() { FatType::Fat12=>12, FatType::Fat16=>16, FatType::Fat32=>32 });
    }

    // Dry-run only in this path; committing is handled by apply_fsck_journal_commit
    for action in &report.actions_proposed {
        match action {
            FsckAction::FreeClusters { clusters } => {
                println!(" - would free {} cluster(s) (first 16 shown): {:?}", clusters.len(), &clusters[..std::cmp::min(16, clusters.len())]);
            }
            FsckAction::SyncFatMirrors { source_fat } => {
                println!(" - would sync FAT mirrors from FAT{}", source_fat);
            }
            FsckAction::BrokenChainEnd { start_cluster } => {
                println!(" - would fix broken chain end starting at cluster {}", start_cluster);
            }
            FsckAction::CrossLinkBreak { cluster } => {
                println!(" - would break cross-link by freeing cluster {}", cluster);
            }
        }
    }
    println!("fsck: dry-run only. Re-run with --commit to apply on --shadow image.");

    Ok(())
}

#[cfg(not(unix))]
async fn apply_fsck_journal_with_shadow(path: &str, _shadow_img: Option<&str>) -> Result<()> { apply_fsck_journal(path).await }

#[cfg(unix)]
async fn apply_fsck_journal_commit(path: &str, shadow_img: Option<&str>) -> Result<()> {
    let img = shadow_img.ok_or_else(|| anyhow!("fsck: --commit requires --shadow <image>"))?;
    let data = std::fs::read(path).with_context(|| format!("fsck: cannot read journal {path}"))?;
    let report: FsckReport = serde_json::from_slice(&data).context("fsck: invalid journal format")?;
    for action in &report.actions_proposed {
        match action {
            FsckAction::FreeClusters { clusters } => {
                apply_free_clusters_fat(img, clusters)?;
            }
            FsckAction::SyncFatMirrors { source_fat } => {
                sync_fat_mirrors(img, *source_fat)?;
            }
            FsckAction::BrokenChainEnd { start_cluster } => {
                apply_broken_chain_end(img, *start_cluster)?;
            }
            FsckAction::CrossLinkBreak { cluster } => {
                apply_cross_link_break(img, *cluster)?;
            }
        }
    }
    println!("fsck: commit completed on shadow image {}", img);
    Ok(())
}

#[cfg(not(unix))]
async fn apply_fsck_journal_commit(_path: &str, _shadow_img: Option<&str>) -> Result<()> {
    Err(anyhow!("fsck: commit not supported on this platform"))
}

#[cfg(unix)]
fn apply_free_clusters_fat(image_path: &str, clusters: &[u32]) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(image_path)
        .with_context(|| format!("fsck: cannot open shadow image {} for write", image_path))?;
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;

    // Parse BPB
    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u32;
    let spc = bpb[13] as u32;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u32;
    let nfats = bpb[16] as u32;
    let root_ent_cnt = u16::from_le_bytes([bpb[17], bpb[18]]) as u32;
    let tot_sec_16 = u16::from_le_bytes([bpb[19], bpb[20]]) as u32;
    let tot_sec_32 = u32::from_le_bytes([bpb[32], bpb[33], bpb[34], bpb[35]]);
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u32;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]);
    let fat_sz = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 };
    let total_sectors = if tot_sec_16 != 0 { tot_sec_16 } else { tot_sec_32 };

    // Compute cluster count to determine FAT type
    let root_dir_sectors = ((root_ent_cnt * 32) + (bps - 1)) / bps;
    let data_sectors = total_sectors - (rsvd + (nfats * fat_sz) + root_dir_sectors);
    let cluster_count = data_sectors / spc;
    let fat_type = if cluster_count < 4085 { 12 } else if cluster_count < 65525 { 16 } else { 32 };

    // Base offsets
    let fat0_offset = (rsvd * bps) as u64; // first FAT start
    let fat_bytes_len = (fat_sz as u64) * (bps as u64);

    // Writer closures per FAT type
    let write_fat12 = |f: &mut std::fs::File, fat_base: u64, cluster: u32| -> Result<()> {
        // Each entry is 12 bits; two entries span 3 bytes
        let n = cluster as u64;
        let byte_index = fat_base + (n + (n / 2)) as u64; // floor(1.5 * n)
        let mut pair = [0u8; 2];
        f.seek(SeekFrom::Start(byte_index))?;
        f.read_exact(&mut pair)?;
        if cluster % 2 == 0 {
            // even: entry uses pair[0] and low nibble of pair[1]
            pair[0] = 0x00;
            pair[1] &= 0xF0; // preserve high nibble (belongs to next entry)
        } else {
            // odd: entry uses high nibble of pair[0] and pair[1]
            pair[0] &= 0x0F; // preserve low nibble (belongs to prev entry)
            pair[1] = 0x00;
        }
        f.seek(SeekFrom::Start(byte_index))?;
        f.write_all(&pair)?;
        Ok(())
    };

    let write_fat16 = |f: &mut std::fs::File, fat_base: u64, cluster: u32| -> Result<()> {
        let entry_off = fat_base + (cluster as u64) * 2u64;
        f.seek(SeekFrom::Start(entry_off))?;
        f.write_all(&[0x00, 0x00])?;
        Ok(())
    };

    let write_fat32 = |f: &mut std::fs::File, fat_base: u64, cluster: u32| -> Result<()> {
        let entry_off = fat_base + (cluster as u64) * 4u64;
        f.seek(SeekFrom::Start(entry_off))?;
        // Clear full 32 bits (lower 28 are the actual value; clearing all is fine)
        f.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        Ok(())
    };

    for &cl in clusters {
        if cl < 2 { continue; } // reserved clusters
        for fat_idx in 0..nfats {
            let base = fat0_offset + (fat_idx as u64) * fat_bytes_len;
            match fat_type {
                12 => write_fat12(&mut f, base, cl).context("fsck: write FAT12 entry")?,
                16 => write_fat16(&mut f, base, cl).context("fsck: write FAT16 entry")?,
                32 => write_fat32(&mut f, base, cl).context("fsck: write FAT32 entry")?,
                _ => unreachable!(),
            }
        }
    }
    f.flush().ok();
    Ok(())
}

#[cfg(unix)]
fn sync_fat_mirrors(image_path: &str, source_fat: u8) -> Result<()> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut f = std::fs::OpenOptions::new().read(true).write(true).open(image_path)
        .with_context(|| format!("fsck: cannot open shadow image {} for write", image_path))?;
    let mut bpb = [0u8; 512];
    f.read_exact(&mut bpb).context("fsck: failed to read boot sector")?;

    let bps = u16::from_le_bytes([bpb[11], bpb[12]]) as u64;
    let rsvd = u16::from_le_bytes([bpb[14], bpb[15]]) as u64;
    let nfats = bpb[16] as u64;
    let fat_sz_16 = u16::from_le_bytes([bpb[22], bpb[23]]) as u64;
    let fat_sz_32 = u32::from_le_bytes([bpb[36], bpb[37], bpb[38], bpb[39]]) as u64;
    let fat_bytes = if fat_sz_16 != 0 { fat_sz_16 } else { fat_sz_32 } * bps;
    let base0 = rsvd * bps;

    if nfats < 2 { return Ok(()); }
    let src = source_fat.min((nfats - 1) as u8) as u64;
    let src_off = base0 + src * fat_bytes;

    // Read source FAT entirely in chunks and copy to others
    let mut buf = vec![0u8; 256 * 1024];
    for dst in 0..nfats {
        if dst == src { continue; }
        let dst_off = base0 + dst * fat_bytes;
        let mut remaining = fat_bytes;
        let mut offset = 0u64;
        while remaining > 0 {
            let to_io = std::cmp::min(remaining, buf.len() as u64) as usize;
            f.seek(SeekFrom::Start(src_off + offset))?;
            f.read_exact(&mut buf[..to_io])?;
            f.seek(SeekFrom::Start(dst_off + offset))?;
            f.write_all(&buf[..to_io])?;
            remaining -= to_io as u64;
            offset += to_io as u64;
        }
    }
    f.flush().ok();
    Ok(())
}

#[cfg(not(unix))]
fn apply_free_clusters_fat(_image_path: &str, _clusters: &[u32]) -> Result<()> {
    Err(anyhow!("fsck: commit not supported on this platform"))
}

#[cfg(unix)]
fn create_shadow_image(device: &str, output: Option<&str>) -> Result<()> {
    use std::io::{Read, Write};
    let out_path = output.map(|s| s.to_string())
        .unwrap_or_else(|| ".nxsh/fsck_shadow.img".to_string());
    let out_parent = std::path::Path::new(&out_path).parent()
        .map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
    let _ = std::fs::create_dir_all(&out_parent);

    let mut src = std::fs::OpenOptions::new().read(true).open(device)
        .with_context(|| format!("fsck: cannot open {} for read", device))?;
    let mut dst = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&out_path)
        .with_context(|| format!("fsck: cannot create {}", out_path))?;

    let mut buf = vec![0u8; 2 * 1024 * 1024]; // 2 MiB chunks
    let mut total: u64 = 0;
    loop {
        let n = src.read(&mut buf)?;
        if n == 0 { break; }
        dst.write_all(&buf[..n])?;
        total += n as u64;
        if total % (100 * 1024 * 1024) < (2 * 1024 * 1024) { // coarse progress every ~100MiB
            println!("fsck: shadow copy progress ~{} MiB", total / (1024 * 1024));
        }
    }
    dst.flush()?;
    println!("fsck: created shadow image {} ({} bytes)", out_path, total);
    Ok(())
}

#[cfg(not(unix))]
fn create_shadow_image(_device: &str, output: Option<&str>) -> Result<()> {
    let out = output.unwrap_or(".nxsh/fsck_shadow.img");
    println!("fsck: create-shadow is not supported on this platform (requested output: {out})");
    Ok(())
}
