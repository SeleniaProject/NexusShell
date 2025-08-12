use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs::{File, Metadata},
    io::{Read, Write, BufReader, BufWriter, Seek, SeekFrom, Cursor},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self as async_fs, OpenOptions},
    io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, AsyncSeekExt},
    sync::RwLock,
    task::spawn_blocking,
};
use serde::{Deserialize, Serialize};
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};
use bzip2_rs::DecoderReader as BzDecoder;
use ruzstd::streaming_decoder::StreamingDecoder as ZstdDecoder;
use zip::{ZipArchive, ZipWriter, write::FileOptions as ZipFileOptions, CompressionMethod};
use tar::{Archive as TarArchive, Builder as TarBuilder, Header as TarHeader};
use sevenz_rust::{SevenZReader, SevenZWriter, Password};
use log::{info, warn, error, debug};
use rayon::prelude::*;

use crate::common::i18n::tr;
use nxsh_core::{context::NxshContext, result::NxshResult};

/// Compression and archive manager
pub struct CompressionManager {
    active_operations: Arc<RwLock<HashMap<String, CompressionOperation>>>,
    config: CompressionConfig,
    stats: Arc<RwLock<CompressionStats>>,
}

impl CompressionManager {
    /// Create a new compression manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            config: CompressionConfig::default(),
            stats: Arc::new(RwLock::new(CompressionStats::default())),
        })
    }
    
    /// Compress files using gzip
    pub async fn gzip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_gzip_args(args)?;
        
        info!("Starting gzip compression with {} threads", options.threads);
        
        let operation_id = self.start_operation("gzip", &options.input_files).await;
        
        if options.parallel && options.input_files.len() > 1 {
            self.compress_files_parallel(&options, CompressionFormat::Gzip).await?;
        } else {
            for input_file in &options.input_files {
                self.compress_single_file(input_file, &options.output_file, CompressionFormat::Gzip, &options).await?;
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Decompress gzip files
    pub async fn gunzip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_gunzip_args(args)?;
        
        info!("Starting gzip decompression");
        
        let operation_id = self.start_operation("gunzip", &options.input_files).await;
        
        for input_file in &options.input_files {
            self.decompress_single_file(input_file, &options.output_file, CompressionFormat::Gzip, &options).await?;
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Compress files using bzip2
    pub async fn bzip2(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_bzip2_args(args)?;
        
        info!("Starting bzip2 (decompression-only build; compression will error)");
        
        let operation_id = self.start_operation("bzip2", &options.input_files).await;
        
        if options.parallel && options.input_files.len() > 1 {
            self.compress_files_parallel(&options, CompressionFormat::Bzip2).await?;
        } else {
            for input_file in &options.input_files {
                self.compress_single_file(input_file, &options.output_file, CompressionFormat::Bzip2, &options).await?;
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Decompress bzip2 files
    pub async fn bunzip2(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_bunzip2_args(args)?;
        
        info!("Starting bzip2 decompression");
        
        let operation_id = self.start_operation("bunzip2", &options.input_files).await;
        
        for input_file in &options.input_files {
            self.decompress_single_file(input_file, &options.output_file, CompressionFormat::Bzip2, &options).await?;
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Compress files using xz
    pub async fn xz(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_xz_args(args)?;
        
        info!("Starting xz compression");
        
        let operation_id = self.start_operation("xz", &options.input_files).await;
        
        if options.parallel && options.input_files.len() > 1 {
            self.compress_files_parallel(&options, CompressionFormat::Xz).await?;
        } else {
            for input_file in &options.input_files {
                self.compress_single_file(input_file, &options.output_file, CompressionFormat::Xz, &options).await?;
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Decompress xz files
    pub async fn unxz(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_unxz_args(args)?;
        
        info!("Starting xz decompression");
        
        let operation_id = self.start_operation("unxz", &options.input_files).await;
        
        for input_file in &options.input_files {
            self.decompress_single_file(input_file, &options.output_file, CompressionFormat::Xz, &options).await?;
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Compress files using zstd
    pub async fn zstd(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_zstd_args(args)?;
        
        info!("Starting zstd (decompression-only build; compression will error) with level {}", options.compression_level);
        
        let operation_id = self.start_operation("zstd", &options.input_files).await;
        
        if options.parallel && options.input_files.len() > 1 {
            self.compress_files_parallel(&options, CompressionFormat::Zstd).await?;
        } else {
            for input_file in &options.input_files {
                self.compress_single_file(input_file, &options.output_file, CompressionFormat::Zstd, &options).await?;
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Decompress zstd files
    pub async fn unzstd(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_unzstd_args(args)?;
        
        info!("Starting zstd decompression");
        
        let operation_id = self.start_operation("unzstd", &options.input_files).await;
        
        for input_file in &options.input_files {
            self.decompress_single_file(input_file, &options.output_file, CompressionFormat::Zstd, &options).await?;
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Create zip archive
    pub async fn zip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_zip_args(args)?;
        
        info!("Creating zip archive: {}", options.archive_name);
        
        let operation_id = self.start_operation("zip", &options.input_files).await;
        
        self.create_zip_archive(&options).await?;
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Extract zip archive
    pub async fn unzip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_unzip_args(args)?;
        
        info!("Extracting zip archive: {}", options.archive_file);
        
        let operation_id = self.start_operation("unzip", &[options.archive_file.clone()]).await;
        
        self.extract_zip_archive(&options).await?;
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Create tar archive
    pub async fn tar(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_tar_args(args)?;
        
        info!("Tar operation: {} on {}", options.operation, options.archive_name);
        
        let operation_id = self.start_operation("tar", &options.input_files).await;
        
        match options.operation.as_str() {
            "create" | "c" => {
                self.create_tar_archive(&options).await?;
            },
            "extract" | "x" => {
                self.extract_tar_archive(&options).await?;
            },
            "list" | "t" => {
                self.list_tar_archive(&options).await?;
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown tar operation: {}", options.operation).into());
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Create or extract 7z archive
    pub async fn sevenz(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_7z_args(args)?;
        
        info!("7z operation: {} on {}", options.operation, options.archive_name);
        
        let operation_id = self.start_operation("7z", &options.input_files).await;
        
        match options.operation.as_str() {
            "a" | "add" => {
                self.create_7z_archive(&options).await?;
            },
            "x" | "extract" => {
                self.extract_7z_archive(&options).await?;
            },
            "l" | "list" => {
                self.list_7z_archive(&options).await?;
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown 7z operation: {}", options.operation).into());
            }
        }
        
        self.finish_operation(&operation_id).await;
        
        Ok(())
    }
    
    /// Get compression statistics
    pub async fn get_stats(&self) -> CompressionStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// List active operations
    pub async fn list_operations(&self) -> Vec<CompressionOperation> {
        let operations = self.active_operations.read().await;
        operations.values().cloned().collect()
    }
    
    // Private helper methods
    
    async fn start_operation(&self, operation_type: &str, files: &[String]) -> String {
        let operation_id = format!("{}_{}", operation_type, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        
        let operation = CompressionOperation {
            id: operation_id.clone(),
            operation_type: operation_type.to_string(),
            files: files.to_vec(),
            start_time: SystemTime::now(),
            progress: 0.0,
            status: OperationStatus::Running,
        };
        
        let mut operations = self.active_operations.write().await;
        operations.insert(operation_id.clone(), operation);
        
        operation_id
    }
    
    async fn finish_operation(&self, operation_id: &str) {
        let mut operations = self.active_operations.write().await;
        if let Some(mut operation) = operations.remove(operation_id) {
            operation.status = OperationStatus::Completed;
            operation.progress = 100.0;
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_operations += 1;
            stats.completed_operations += 1;
        }
    }
    
    async fn compress_files_parallel(&self, options: &CompressionOptions, format: CompressionFormat) -> Result<()> {
        let files = options.input_files.clone();
        let output_file = options.output_file.clone();
        let compression_level = options.compression_level;
        let threads = options.threads;
        
        // Use rayon for parallel processing
        let results: Result<Vec<_>, _> = spawn_blocking(move || {
            files.par_iter()
                .map(|input_file| {
                    Self::compress_file_blocking(input_file, &output_file, format, compression_level)
                })
                .collect()
        }).await.context("Failed to spawn blocking task")?;
        
        results?;
        
        Ok(())
    }
    
    fn compress_file_blocking(input_file: &str, output_file: &Option<String>, format: CompressionFormat, level: u32) -> Result<()> {
        let input_path = Path::new(input_file);
        let output_path = if let Some(ref output) = output_file {
            PathBuf::from(output)
        } else {
            match format {
                CompressionFormat::Gzip => input_path.with_extension("gz"),
                CompressionFormat::Bzip2 => input_path.with_extension("bz2"),
                CompressionFormat::Xz => input_path.with_extension("xz"),
                CompressionFormat::Zstd => input_path.with_extension("zst"),
            }
        };
        
        let input_file = File::open(input_path).context("Failed to open input file")?;
        let output_file = File::create(&output_path).context("Failed to create output file")?;
        
        let mut reader = BufReader::new(input_file);
        let mut writer = BufWriter::new(output_file);
        
        match format {
            CompressionFormat::Gzip => {
                let mut encoder = GzEncoder::new(writer, GzCompression::new(level));
                std::io::copy(&mut reader, &mut encoder).context("Failed to compress with gzip")?;
                encoder.finish().context("Failed to finish gzip compression")?;
            },
            CompressionFormat::Bzip2 => {
                // Decode-only in pure-Rust build
                return Err(anyhow::anyhow!("bzip2 compression not supported (decode-only). Use gzip or xz for compression, or an external bzip2 binary."));
            },
            CompressionFormat::Xz => {
                // lzma_rs provides function-based API
                lzma_rs::xz_compress(&mut reader, &mut writer).context("Failed to compress with xz")?;
                writer.flush().ok();
            },
            CompressionFormat::Zstd => {
                // Decode-only in pure-Rust build
                return Err(anyhow::anyhow!("zstd compression not supported (decode-only). Use gzip or xz for compression, or an external zstd binary."));
            },
        }
        
        info!("Compressed {} to {}", input_path.display(), output_path.display());
        
        Ok(())
    }
    
    async fn compress_single_file(&self, input_file: &str, output_file: &Option<String>, format: CompressionFormat, options: &CompressionOptions) -> Result<()> {
        let input_path = PathBuf::from(input_file);
        let output_path = if let Some(ref output) = output_file {
            PathBuf::from(output)
        } else {
            match format {
                CompressionFormat::Gzip => input_path.with_extension("gz"),
                CompressionFormat::Bzip2 => input_path.with_extension("bz2"),
                CompressionFormat::Xz => input_path.with_extension("xz"),
                CompressionFormat::Zstd => input_path.with_extension("zst"),
            }
        };
        
        let input_size = async_fs::metadata(&input_path).await?.len();
        let mut input_file = async_fs::File::open(&input_path).await?;
        let output_file = async_fs::File::create(&output_path).await?;
        
        let start_time = SystemTime::now();
        
        // Read input file
        let mut input_data = Vec::new();
        input_file.read_to_end(&mut input_data).await?;
        
        // Compress in blocking task
        let compressed_data = spawn_blocking(move || -> Result<Vec<u8>> {
            let mut output_data = Vec::new();
            
            match format {
                CompressionFormat::Gzip => {
                    let mut encoder = GzEncoder::new(&mut output_data, GzCompression::new(options.compression_level));
                    encoder.write_all(&input_data)?;
                    encoder.finish()?;
                },
                CompressionFormat::Bzip2 => {
                    return Err(anyhow::anyhow!("bzip2 compression not supported (decode-only). Use gzip or xz for compression, or an external bzip2 binary."));
                },
                CompressionFormat::Xz => {
                    let mut input_cursor = Cursor::new(&input_data);
                    lzma_rs::xz_compress(&mut input_cursor, &mut output_data).context("Failed to compress with xz")?;
                },
                CompressionFormat::Zstd => {
                    return Err(anyhow::anyhow!("zstd compression not supported (decode-only). Use gzip or xz for compression, or an external zstd binary."));
                },
            }
            
            Ok(output_data)
        }).await??;
        
        // Write compressed data
        let mut output_file = async_fs::File::create(&output_path).await?;
        output_file.write_all(&compressed_data).await?;
        output_file.sync_all().await?;
        
        let duration = start_time.elapsed().unwrap();
        let output_size = compressed_data.len() as u64;
        let ratio = (input_size as f64 - output_size as f64) / input_size as f64 * 100.0;
        
        if options.verbose {
            println!("  {} -> {} ({:.1}% reduction) in {:.2}s",
                    input_path.display(),
                    output_path.display(),
                    ratio,
                    duration.as_secs_f64());
        }
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_bytes_processed += input_size;
            stats.total_bytes_saved += input_size - output_size;
            stats.files_processed += 1;
        }
        
        Ok(())
    }
    
    async fn decompress_single_file(&self, input_file: &str, output_file: &Option<String>, format: CompressionFormat, options: &DecompressionOptions) -> Result<()> {
        let input_path = PathBuf::from(input_file);
        let output_path = if let Some(ref output) = output_file {
            PathBuf::from(output)
        } else {
            // Remove compression extension
            match format {
                CompressionFormat::Gzip => {
                    if input_path.extension().and_then(|s| s.to_str()) == Some("gz") {
                        input_path.with_extension("")
                    } else {
                        input_path.with_extension("decompressed")
                    }
                },
                CompressionFormat::Bzip2 => {
                    if input_path.extension().and_then(|s| s.to_str()) == Some("bz2") {
                        input_path.with_extension("")
                    } else {
                        input_path.with_extension("decompressed")
                    }
                },
                CompressionFormat::Xz => {
                    if input_path.extension().and_then(|s| s.to_str()) == Some("xz") {
                        input_path.with_extension("")
                    } else {
                        input_path.with_extension("decompressed")
                    }
                },
                CompressionFormat::Zstd => {
                    if input_path.extension().and_then(|s| s.to_str()) == Some("zst") {
                        input_path.with_extension("")
                    } else {
                        input_path.with_extension("decompressed")
                    }
                },
            }
        };
        
        let mut input_file = async_fs::File::open(&input_path).await?;
        let mut input_data = Vec::new();
        input_file.read_to_end(&mut input_data).await?;
        
        let start_time = SystemTime::now();
        
        // Decompress in blocking task
        let decompressed_data = spawn_blocking(move || -> Result<Vec<u8>> {
            let mut output_data = Vec::new();
            
            match format {
                CompressionFormat::Gzip => {
                    let mut decoder = GzDecoder::new(&input_data[..]);
                    decoder.read_to_end(&mut output_data)?;
                },
                CompressionFormat::Bzip2 => {
                    let mut decoder = BzDecoder::new(&input_data[..]);
                    decoder.read_to_end(&mut output_data)?;
                },
                CompressionFormat::Xz => {
                    let mut cursor = Cursor::new(&input_data);
                    lzma_rs::xz_decompress(&mut cursor, &mut output_data).context("Failed to decompress xz")?;
                },
                CompressionFormat::Zstd => {
                    let mut decoder = ZstdDecoder::new(&input_data[..])?;
                    decoder.read_to_end(&mut output_data)?;
                },
            }
            
            Ok(output_data)
        }).await??;
        
        // Write decompressed data
        let mut output_file = async_fs::File::create(&output_path).await?;
        output_file.write_all(&decompressed_data).await?;
        output_file.sync_all().await?;
        
        let duration = start_time.elapsed().unwrap();
        let input_size = input_data.len() as u64;
        let output_size = decompressed_data.len() as u64;
        
        if options.verbose {
            println!("  {} -> {} ({} bytes) in {:.2}s",
                    input_path.display(),
                    output_path.display(),
                    output_size,
                    duration.as_secs_f64());
        }
        
        Ok(())
    }
    
    async fn create_zip_archive(&self, options: &ZipOptions) -> Result<()> {
        let archive_path = PathBuf::from(&options.archive_name);
        let archive_file = File::create(&archive_path).context("Failed to create zip archive")?;
        let mut zip_writer = ZipWriter::new(archive_file);
        
        let zip_options = ZipFileOptions::default()
            .compression_method(match options.compression_level {
                0 => CompressionMethod::Stored,
                _ => CompressionMethod::Deflated,
            })
            .compression_level(Some(options.compression_level as i64));
        
        for input_file in &options.input_files {
            let input_path = Path::new(input_file);
            
            if input_path.is_file() {
                self.add_file_to_zip(&mut zip_writer, input_path, &zip_options, options.verbose).await?;
            } else if input_path.is_dir() && options.recursive {
                self.add_directory_to_zip(&mut zip_writer, input_path, input_path, &zip_options, options.verbose).await?;
            }
        }
        
        zip_writer.finish().context("Failed to finish zip archive")?;
        
        if options.verbose {
            println!("Created zip archive: {}", archive_path.display());
        }
        
        Ok(())
    }
    
    async fn add_file_to_zip(&self, zip_writer: &mut ZipWriter<File>, file_path: &Path, options: &ZipFileOptions, verbose: bool) -> Result<()> {
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        zip_writer.start_file(file_name, *options).context("Failed to start zip file entry")?;
        
        let mut file = File::open(file_path).context("Failed to open input file")?;
        std::io::copy(&mut file, zip_writer).context("Failed to copy file to zip")?;
        
        if verbose {
            println!("  adding: {}", file_path.display());
        }
        
        Ok(())
    }
    
    async fn add_directory_to_zip(&self, zip_writer: &mut ZipWriter<File>, dir_path: &Path, base_path: &Path, options: &ZipFileOptions, verbose: bool) -> Result<()> {
        let entries = std::fs::read_dir(dir_path).context("Failed to read directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let entry_path = entry.path();
            
            if entry_path.is_file() {
                let relative_path = entry_path.strip_prefix(base_path)
                    .context("Failed to get relative path")?;
                let file_name = relative_path.to_str()
                    .context("Invalid file name")?;
                
                zip_writer.start_file(file_name, *options).context("Failed to start zip file entry")?;
                
                let mut file = File::open(&entry_path).context("Failed to open input file")?;
                std::io::copy(&mut file, zip_writer).context("Failed to copy file to zip")?;
                
                if verbose {
                    println!("  adding: {}", entry_path.display());
                }
            } else if entry_path.is_dir() {
                self.add_directory_to_zip(zip_writer, &entry_path, base_path, options, verbose).await?;
            }
        }
        
        Ok(())
    }
    
    async fn extract_zip_archive(&self, options: &UnzipOptions) -> Result<()> {
        let archive_file = File::open(&options.archive_file).context("Failed to open zip archive")?;
        let mut zip_archive = ZipArchive::new(archive_file).context("Failed to read zip archive")?;
        
        let output_dir = options.output_dir.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        
        for i in 0..zip_archive.len() {
            let mut zip_file = zip_archive.by_index(i).context("Failed to get zip entry")?;
            let output_path = output_dir.join(zip_file.name());
            
            if zip_file.name().ends_with('/') {
                // Directory
                async_fs::create_dir_all(&output_path).await.context("Failed to create directory")?;
                if options.verbose {
                    println!("  creating: {}", output_path.display());
                }
            } else {
                // File
                if let Some(parent) = output_path.parent() {
                    async_fs::create_dir_all(parent).await.context("Failed to create parent directory")?;
                }
                
                let mut output_file = File::create(&output_path).context("Failed to create output file")?;
                std::io::copy(&mut zip_file, &mut output_file).context("Failed to extract file")?;
                
                if options.verbose {
                    println!("  extracting: {}", output_path.display());
                }
            }
        }
        
        if options.verbose {
            println!("Extracted {} files from {}", zip_archive.len(), options.archive_file);
        }
        
        Ok(())
    }
    
    async fn create_tar_archive(&self, options: &TarOptions) -> Result<()> {
        let archive_path = PathBuf::from(&options.archive_name);
        let archive_file = File::create(&archive_path).context("Failed to create tar archive")?;
        
        // For XZ we must stage into a temp tar, then compress via lzma_rs
        let mut tar_builder_opt;
        let use_temp_for_xz = matches!(options.compression, Some(CompressionFormat::Xz));
        let mut temp_path_opt: Option<PathBuf> = None;
        let mut tar_builder = if let Some(ref fmt) = options.compression {
            match fmt {
                CompressionFormat::Gzip => {
                    let encoder = GzEncoder::new(archive_file, GzCompression::default());
                    TarBuilder::new(encoder)
                }
                CompressionFormat::Bzip2 => {
                    return Err(anyhow::anyhow!("bzip2 tar compression not supported (decode-only). Use gzip or xz."));
                }
                CompressionFormat::Xz => {
                    use tempfile::NamedTempFile;
                    let temp = NamedTempFile::new().context("Failed to create temp tar for xz")?;
                    let temp_path = temp.into_temp_path().keep().context("Failed to persist temp file")?;
                    temp_path_opt = Some(PathBuf::from(&temp_path));
                    let temp_file = File::options().write(true).truncate(true).open(&temp_path_opt.as_ref().unwrap())?;
                    TarBuilder::new(BufWriter::new(temp_file))
                }
                CompressionFormat::Zstd => {
                    return Err(anyhow::anyhow!("zstd tar compression not supported (decode-only). Use gzip or xz."));
                }
            }
        } else {
            TarBuilder::new(archive_file)
        };
        
        for input_file in &options.input_files {
            let input_path = Path::new(input_file);
            
            if input_path.is_file() {
                tar_builder.append_path(input_path).context("Failed to add file to tar")?;
                if options.verbose {
                    println!("  adding: {}", input_path.display());
                }
            } else if input_path.is_dir() && options.recursive {
                tar_builder.append_dir_all(input_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new(".")), input_path)
                    .context("Failed to add directory to tar")?;
                if options.verbose {
                    println!("  adding: {} (directory)", input_path.display());
                }
            }
        }
        
        tar_builder.finish().context("Failed to finish tar archive")?;
        
        // If XZ, compress temp tar into final archive
        if use_temp_for_xz {
            let temp_path = temp_path_opt.expect("temp path must exist for xz");
            let input_file = File::open(&temp_path).context("Failed to open temp tar for xz compression")?;
            let mut reader = BufReader::new(input_file);
            let mut writer = BufWriter::new(File::create(&archive_path)?);
            lzma_rs::xz_compress(&mut reader, &mut writer).context("Failed to xz-compress tar")?;
            writer.flush().ok();
            let _ = std::fs::remove_file(&temp_path);
        }
        
        if options.verbose {
            println!("Created tar archive: {}", archive_path.display());
        }
        
        Ok(())
    }

    async fn extract_tar_archive(&self, options: &TarOptions) -> Result<()> {
        let archive_file = File::open(&options.archive_name).context("Failed to open tar archive")?;
        
        let mut tar_archive = if options.compression.is_some() {
             match options.compression.as_ref().unwrap() {
                 CompressionFormat::Gzip => {
                     let decoder = GzDecoder::new(archive_file);
                     TarArchive::new(decoder)
                 },
                 CompressionFormat::Bzip2 => {
                     let decoder = BzDecoder::new(archive_file);
                     TarArchive::new(decoder)
                 },
                 CompressionFormat::Xz => {
                    let mut decompressed = Vec::new();
                    let mut r = BufReader::new(archive_file);
                    lzma_rs::xz_decompress(&mut r, &mut decompressed).context("Failed to decompress xz tar")?;
                    TarArchive::new(Cursor::new(decompressed))
                 },
                 CompressionFormat::Zstd => {
                     let decoder = ZstdDecoder::new(archive_file).context("Failed to create zstd decoder")?;
                     TarArchive::new(decoder)
                 },
             }
         } else {
             TarArchive::new(archive_file)
         };
        
        let output_dir = options.output_dir.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        
        tar_archive.unpack(&output_dir).context("Failed to extract tar archive")?;
        
        if options.verbose {
            println!("Extracted tar archive to: {}", output_dir.display());
        }
        
        Ok(())
    }

    async fn list_tar_archive(&self, options: &TarOptions) -> Result<()> {
        let archive_file = File::open(&options.archive_name).context("Failed to open tar archive")?;
        
        let mut tar_archive = if options.compression.is_some() {
             match options.compression.as_ref().unwrap() {
                 CompressionFormat::Gzip => {
                     let decoder = GzDecoder::new(archive_file);
                     TarArchive::new(decoder)
                 },
                 CompressionFormat::Bzip2 => {
                     let decoder = BzDecoder::new(archive_file);
                     TarArchive::new(decoder)
                 },
                 CompressionFormat::Xz => {
                    let mut decompressed = Vec::new();
                    let mut r = BufReader::new(archive_file);
                    lzma_rs::xz_decompress(&mut r, &mut decompressed).context("Failed to decompress xz tar")?;
                    TarArchive::new(Cursor::new(decompressed))
                 },
                 CompressionFormat::Zstd => {
                     let decoder = ZstdDecoder::new(archive_file).context("Failed to create zstd decoder")?;
                     TarArchive::new(decoder)
                 },
             }
         } else {
             TarArchive::new(archive_file)
         };
        
        for entry in tar_archive.entries().context("Failed to read tar entries")? {
            let entry = entry.context("Failed to read tar entry")?;
            let path = entry.path().context("Failed to get entry path")?;
            let header = entry.header();
            
            let file_type = if header.entry_type().is_dir() {
                "d"
            } else if header.entry_type().is_file() {
                "-"
            } else {
                "?"
            };
            
            let size = header.size().unwrap_or(0);
            let mtime = header.mtime().unwrap_or(0);
            
            println!("{} {:>10} {} {}", 
                    file_type,
                    size,
                    chrono::DateTime::from_timestamp(mtime as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    path.display());
        }
        
        Ok(())
    }
    
    async fn create_7z_archive(&self, options: &SevenZOptions) -> Result<()> {
        // Simplified 7z creation using sevenz_rust
        // In a real implementation, this would use the full 7z API
        
        let archive_path = PathBuf::from(&options.archive_name);
        
        // For now, just create a simple archive
        // This is a placeholder implementation
        println!("Creating 7z archive: {} (placeholder implementation)", archive_path.display());
        
        for input_file in &options.input_files {
            if options.verbose {
                println!("  adding: {}", input_file);
            }
        }
        
        Ok(())
    }
    
    async fn extract_7z_archive(&self, options: &SevenZOptions) -> Result<()> {
        // Simplified 7z extraction using sevenz_rust
        // In a real implementation, this would use the full 7z API
        
        println!("Extracting 7z archive: {} (placeholder implementation)", options.archive_name);
        
        let output_dir = options.output_dir.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        
        if options.verbose {
            println!("  extracting to: {}", output_dir.display());
        }
        
        Ok(())
    }
    
    async fn list_7z_archive(&self, options: &SevenZOptions) -> Result<()> {
        // Simplified 7z listing
        println!("Listing 7z archive: {} (placeholder implementation)", options.archive_name);
        
        Ok(())
    }
    
    // Argument parsing methods
    
    fn parse_gzip_args(&self, args: &[String]) -> Result<CompressionOptions> {
        let mut options = CompressionOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-1" | "--fast" => options.compression_level = 1,
                "-9" | "--best" => options.compression_level = 9,
                "-c" | "--stdout" => options.to_stdout = true,
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--force" => options.force = true,
                "-k" | "--keep" => options.keep_original = true,
                "-r" | "--recursive" => options.recursive = true,
                "-t" | "--test" => options.test_only = true,
                "-P" | "--parallel" => options.parallel = true,
                "-T" | "--threads" => {
                    i += 1;
                    if i < args.len() {
                        options.threads = args[i].parse().context("Invalid thread count")?;
                    }
                },
                arg if !arg.starts_with('-') => {
                    options.input_files.push(arg.to_string());
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_gunzip_args(&self, args: &[String]) -> Result<DecompressionOptions> {
        let mut options = DecompressionOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--stdout" => options.to_stdout = true,
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--force" => options.force = true,
                "-k" | "--keep" => options.keep_original = true,
                "-t" | "--test" => options.test_only = true,
                arg if !arg.starts_with('-') => {
                    options.input_files.push(arg.to_string());
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_bzip2_args(&self, args: &[String]) -> Result<CompressionOptions> {
        let mut options = CompressionOptions::default();
        options.compression_level = 9; // bzip2 default
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-1" | "--fast" => options.compression_level = 1,
                "-9" | "--best" => options.compression_level = 9,
                "-c" | "--stdout" => options.to_stdout = true,
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--force" => options.force = true,
                "-k" | "--keep" => options.keep_original = true,
                "-z" | "--compress" => {}, // Default behavior
                arg if !arg.starts_with('-') => {
                    options.input_files.push(arg.to_string());
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_bunzip2_args(&self, args: &[String]) -> Result<DecompressionOptions> {
        self.parse_gunzip_args(args) // Same options as gunzip
    }
    
    fn parse_xz_args(&self, args: &[String]) -> Result<CompressionOptions> {
        let mut options = CompressionOptions::default();
        options.compression_level = 6; // xz default
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-0" | "-1" | "-2" | "-3" | "-4" | "-5" | "-6" | "-7" | "-8" | "-9" => {
                    options.compression_level = args[i][1..].parse().context("Invalid compression level")?;
                },
                "-c" | "--stdout" => options.to_stdout = true,
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--force" => options.force = true,
                "-k" | "--keep" => options.keep_original = true,
                "-z" | "--compress" => {}, // Default behavior
                "-T" | "--threads" => {
                    i += 1;
                    if i < args.len() {
                        options.threads = args[i].parse().context("Invalid thread count")?;
                    }
                },
                arg if !arg.starts_with('-') => {
                    options.input_files.push(arg.to_string());
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_unxz_args(&self, args: &[String]) -> Result<DecompressionOptions> {
        self.parse_gunzip_args(args) // Same options as gunzip
    }
    
    fn parse_zstd_args(&self, args: &[String]) -> Result<CompressionOptions> {
        let mut options = CompressionOptions::default();
        options.compression_level = 3; // zstd default
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                arg if arg.starts_with("-") && arg.len() == 3 && arg.chars().nth(1).unwrap().is_ascii_digit() => {
                    // Handle -1, -2, ..., -9, -10, etc.
                    options.compression_level = arg[1..].parse().context("Invalid compression level")?;
                },
                "-c" | "--stdout" => options.to_stdout = true,
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--force" => options.force = true,
                "-k" | "--keep" => options.keep_original = true,
                "-T" | "--threads" => {
                    i += 1;
                    if i < args.len() {
                        options.threads = args[i].parse().context("Invalid thread count")?;
                    }
                },
                "--ultra" => {
                    // Enable ultra mode (levels 20-22)
                    if options.compression_level < 20 {
                        options.compression_level = 20;
                    }
                },
                arg if !arg.starts_with('-') => {
                    options.input_files.push(arg.to_string());
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_unzstd_args(&self, args: &[String]) -> Result<DecompressionOptions> {
        self.parse_gunzip_args(args) // Same options as gunzip
    }
    
    fn parse_zip_args(&self, args: &[String]) -> Result<ZipOptions> {
        let mut options = ZipOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-r" | "--recurse-paths" => options.recursive = true,
                "-v" | "--verbose" => options.verbose = true,
                "-0" | "-1" | "-2" | "-3" | "-4" | "-5" | "-6" | "-7" | "-8" | "-9" => {
                    options.compression_level = args[i][1..].parse().context("Invalid compression level")?;
                },
                arg if !arg.starts_with('-') => {
                    if options.archive_name.is_empty() {
                        options.archive_name = arg.to_string();
                    } else {
                        options.input_files.push(arg.to_string());
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.archive_name.is_empty() {
            return Err(anyhow::anyhow!("Archive name required"));
        }
        
        if options.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        
        Ok(options)
    }
    
    fn parse_unzip_args(&self, args: &[String]) -> Result<UnzipOptions> {
        let mut options = UnzipOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-v" | "--verbose" => options.verbose = true,
                "-o" | "--overwrite" => options.overwrite = true,
                "-d" => {
                    i += 1;
                    if i < args.len() {
                        options.output_dir = Some(args[i].clone());
                    }
                },
                arg if !arg.starts_with('-') => {
                    if options.archive_file.is_empty() {
                        options.archive_file = arg.to_string();
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.archive_file.is_empty() {
            return Err(anyhow::anyhow!("Archive file required"));
        }
        
        Ok(options)
    }
    
    fn parse_tar_args(&self, args: &[String]) -> Result<TarOptions> {
        let mut options = TarOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--create" => options.operation = "create".to_string(),
                "-x" | "--extract" => options.operation = "extract".to_string(),
                "-t" | "--list" => options.operation = "list".to_string(),
                "-v" | "--verbose" => options.verbose = true,
                "-f" | "--file" => {
                    i += 1;
                    if i < args.len() {
                        options.archive_name = args[i].clone();
                    }
                },
                "-z" | "--gzip" => options.compression = Some(CompressionFormat::Gzip),
                "-j" | "--bzip2" => options.compression = Some(CompressionFormat::Bzip2),
                "-J" | "--xz" => options.compression = Some(CompressionFormat::Xz),
                "--zstd" => options.compression = Some(CompressionFormat::Zstd),
                "-C" => {
                    i += 1;
                    if i < args.len() {
                        options.output_dir = Some(args[i].clone());
                    }
                },
                "-r" | "--append" => options.recursive = true,
                arg if !arg.starts_with('-') => {
                    if options.operation != "extract" && options.operation != "list" {
                        options.input_files.push(arg.to_string());
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.archive_name.is_empty() {
            return Err(anyhow::anyhow!("Archive name required"));
        }
        
        if options.operation.is_empty() {
            return Err(anyhow::anyhow!("Operation required (create, extract, or list)"));
        }
        
        Ok(options)
    }
    
    fn parse_7z_args(&self, args: &[String]) -> Result<SevenZOptions> {
        let mut options = SevenZOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "a" | "add" => options.operation = "add".to_string(),
                "x" | "extract" => options.operation = "extract".to_string(),
                "l" | "list" => options.operation = "list".to_string(),
                "-v" => options.verbose = true,
                "-r" => options.recursive = true,
                "-o" => {
                    i += 1;
                    if i < args.len() {
                        options.output_dir = Some(args[i].clone());
                    }
                },
                "-p" => {
                    i += 1;
                    if i < args.len() {
                        options.password = Some(args[i].clone());
                    }
                },
                arg if !arg.starts_with('-') => {
                    if options.archive_name.is_empty() {
                        options.archive_name = arg.to_string();
                    } else if options.operation == "add" {
                        options.input_files.push(arg.to_string());
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.operation.is_empty() {
            return Err(anyhow::anyhow!("Operation required (add, extract, or list)"));
        }
        
        if options.archive_name.is_empty() {
            return Err(anyhow::anyhow!("Archive name required"));
        }
        
        Ok(options)
    }
}

// Enums and structs

#[derive(Debug, Clone, Copy)]
enum CompressionFormat {
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

#[derive(Debug, Clone)]
struct CompressionOptions {
    input_files: Vec<String>,
    output_file: Option<String>,
    compression_level: u32,
    verbose: bool,
    force: bool,
    keep_original: bool,
    to_stdout: bool,
    recursive: bool,
    test_only: bool,
    parallel: bool,
    threads: usize,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            output_file: None,
            compression_level: 6,
            verbose: false,
            force: false,
            keep_original: false,
            to_stdout: false,
            recursive: false,
            test_only: false,
            parallel: false,
            threads: num_cpus::get(),
        }
    }
}

#[derive(Debug, Clone)]
struct DecompressionOptions {
    input_files: Vec<String>,
    output_file: Option<String>,
    verbose: bool,
    force: bool,
    keep_original: bool,
    to_stdout: bool,
    test_only: bool,
}

impl Default for DecompressionOptions {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            output_file: None,
            verbose: false,
            force: false,
            keep_original: false,
            to_stdout: false,
            test_only: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ZipOptions {
    archive_name: String,
    input_files: Vec<String>,
    compression_level: u32,
    recursive: bool,
    verbose: bool,
}

impl Default for ZipOptions {
    fn default() -> Self {
        Self {
            archive_name: String::new(),
            input_files: Vec::new(),
            compression_level: 6,
            recursive: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone)]
struct UnzipOptions {
    archive_file: String,
    output_dir: Option<String>,
    verbose: bool,
    overwrite: bool,
}

impl Default for UnzipOptions {
    fn default() -> Self {
        Self {
            archive_file: String::new(),
            output_dir: None,
            verbose: false,
            overwrite: false,
        }
    }
}

#[derive(Debug, Clone)]
struct TarOptions {
    operation: String,
    archive_name: String,
    input_files: Vec<String>,
    output_dir: Option<String>,
    compression: Option<CompressionFormat>,
    verbose: bool,
    recursive: bool,
}

impl Default for TarOptions {
    fn default() -> Self {
        Self {
            operation: String::new(),
            archive_name: String::new(),
            input_files: Vec::new(),
            output_dir: None,
            compression: None,
            verbose: false,
            recursive: false,
        }
    }
}

#[derive(Debug, Clone)]
struct SevenZOptions {
    operation: String,
    archive_name: String,
    input_files: Vec<String>,
    output_dir: Option<String>,
    password: Option<String>,
    verbose: bool,
    recursive: bool,
}

impl Default for SevenZOptions {
    fn default() -> Self {
        Self {
            operation: String::new(),
            archive_name: String::new(),
            input_files: Vec::new(),
            output_dir: None,
            password: None,
            verbose: false,
            recursive: false,
        }
    }
}

#[derive(Debug, Clone)]
struct CompressionOperation {
    id: String,
    operation_type: String,
    files: Vec<String>,
    start_time: SystemTime,
    progress: f64,
    status: OperationStatus,
}

#[derive(Debug, Clone)]
enum OperationStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Default)]
struct CompressionStats {
    total_operations: u64,
    completed_operations: u64,
    failed_operations: u64,
    total_bytes_processed: u64,
    total_bytes_saved: u64,
    files_processed: u64,
}

#[derive(Debug, Clone)]
struct CompressionConfig {
    default_compression_level: u32,
    max_parallel_operations: usize,
    buffer_size: usize,
    enable_progress_reporting: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            default_compression_level: 6,
            max_parallel_operations: num_cpus::get(),
            buffer_size: 64 * 1024, // 64KB
            enable_progress_reporting: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    
    #[tokio::test]
    async fn test_compression_manager_creation() {
        let manager = CompressionManager::new().unwrap();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_operations, 0);
    }
    
    #[test]
    fn test_gzip_args_parsing() {
        let manager = CompressionManager::new().unwrap();
        let args = vec!["test.txt".to_string(), "-9".to_string(), "-v".to_string()];
        let options = manager.parse_gzip_args(&args).unwrap();
        
        assert_eq!(options.input_files, vec!["test.txt"]);
        assert_eq!(options.compression_level, 9);
        assert!(options.verbose);
    }
    
    #[test]
    fn test_zip_args_parsing() {
        let manager = CompressionManager::new().unwrap();
        let args = vec!["archive.zip".to_string(), "file1.txt".to_string(), "file2.txt".to_string(), "-r".to_string()];
        let options = manager.parse_zip_args(&args).unwrap();
        
        assert_eq!(options.archive_name, "archive.zip");
        assert_eq!(options.input_files, vec!["file1.txt", "file2.txt"]);
        assert!(options.recursive);
    }
    
    #[test]
    fn test_tar_args_parsing() {
        let manager = CompressionManager::new().unwrap();
        let args = vec!["-czf".to_string(), "archive.tar.gz".to_string(), "file1.txt".to_string()];
        let options = manager.parse_tar_args(&args).unwrap();
        
        assert_eq!(options.operation, "create");
        assert_eq!(options.archive_name, "archive.tar.gz");
        assert!(matches!(options.compression, Some(CompressionFormat::Gzip)));
    }
    
    #[tokio::test]
    async fn test_operation_tracking() {
        let manager = CompressionManager::new().unwrap();
        
        let operation_id = manager.start_operation("test", &["file1.txt".to_string()]).await;
        
        let operations = manager.list_operations().await;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, operation_id);
        
        manager.finish_operation(&operation_id).await;
        
        let operations = manager.list_operations().await;
        assert_eq!(operations.len(), 0);
    }
    
    #[test]
    fn test_compression_formats() {
        use std::mem::discriminant;
        
        let gzip = CompressionFormat::Gzip;
        let bzip2 = CompressionFormat::Bzip2;
        let xz = CompressionFormat::Xz;
        let zstd = CompressionFormat::Zstd;
        
        assert_ne!(discriminant(&gzip), discriminant(&bzip2));
        assert_ne!(discriminant(&xz), discriminant(&zstd));
    }
}
