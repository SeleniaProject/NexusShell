use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use tar::Builder;
// Remove tar crate dependency - using pure Rust implementation
use walkdir::WalkDir;

pub enum Compression {
    Gzip,
    Bzip2,
    Zstd,
    None,
}

pub struct TarOptions {
    pub src: Vec<String>,
    pub dest: String,
    pub compression: Compression,
}

pub fn tar_cli(opts: TarOptions) -> Result<()> {
    // Calculate total size for progress bar
    let total_bytes: u64 = opts
        .src
        .iter()
        .flat_map(|p| WalkDir::new(p))
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();

    let pb = ProgressBar::new(total_bytes);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap());

    let dest_file = File::create(&opts.dest)?;
    let writer: Box<dyn Write> = match opts.compression {
        Compression::Gzip => Box::new(flate2::write::GzEncoder::new(dest_file, flate2::Compression::default())),
        Compression::Bzip2 => {
            // Use pure Rust alternative or fallback
            eprintln!("Warning: bzip2 compression not available, using no compression");
            Box::new(dest_file)
        },
        Compression::Zstd => {
            // Use pure Rust alternative or fallback  
            eprintln!("Warning: zstd compression not available, using no compression");
            Box::new(dest_file)
        },
        Compression::None => Box::new(dest_file),
    };

    let mut writer = BufWriter::new(ProgressWriter { inner: writer, pb: pb.clone() });
    let mut builder = Builder::new(&mut writer);

    for src in &opts.src {
        builder.append_dir_all(Path::new(src).file_name().unwrap(), src)?;
    }
    builder.finish()?;
    pb.finish_with_message("Archive created");
    Ok(())
}

struct ProgressWriter<W: Write> {
    inner: W,
    pb: ProgressBar,
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.pb.inc(n as u64);
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
} 
