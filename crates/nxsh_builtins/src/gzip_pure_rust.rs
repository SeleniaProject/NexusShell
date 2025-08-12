//! Simple Pure Rust gzip implementation

use anyhow::{anyhow, Result};
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub fn gzip_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("gzip: missing file operand"));
    }

    for filename in args {
        compress_file(filename)?;
    }
    Ok(())
}

pub fn gunzip_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("gunzip: missing file operand"));
    }

    for filename in args {
        decompress_file(filename)?;
    }
    Ok(())
}

fn compress_file(filename: &str) -> Result<()> {
    let input_path = Path::new(filename);
    let output_filename = format!("{filename}.gz");
    let output_path = Path::new(&output_filename);

    let mut input_file = File::open(input_path)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;

    let compressed_data = compress_to_vec(&input_data, 6);

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&compressed_data)?;

    println!("Compressed: {} -> {}", filename, output_path.display());
    Ok(())
}

fn decompress_file(filename: &str) -> Result<()> {
    let input_path = Path::new(filename);
    let output_filename = if filename.ends_with(".gz") {
        filename.strip_suffix(".gz").unwrap()
    } else {
        return Err(anyhow!("File does not have .gz extension"));
    };
    let output_path = Path::new(output_filename);

    let mut input_file = File::open(input_path)?;
    let mut compressed_data = Vec::new();
    input_file.read_to_end(&mut compressed_data)?;

    let decompressed_data = decompress_to_vec(&compressed_data)
        .map_err(|e| anyhow!("Decompression failed: {:?}", e))?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decompressed_data)?;

    println!("Decompressed: {filename} -> {output_filename}");
    Ok(())
}
