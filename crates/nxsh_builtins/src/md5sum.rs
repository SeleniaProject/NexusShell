use anyhow::{anyhow, Context, Result};
use md5::{Context as Md5Context, Digest};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Default, Debug)]
struct Opts {
    binary: bool,
    check: bool,
    quiet: bool,
    status: bool,
    help: bool,
    files: Vec<String>,
}

/// md5sum: compute and check MD5 message digests (subset)
pub fn md5sum_cli(args: &[String]) -> Result<()> {
    let opts = parse_args(args)?;
    if opts.help { print_help(); return Ok(()); }
    if opts.check { run_check_mode(&opts)?; } else { run_hash_mode(&opts)?; }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Opts> {
    let mut opts = Opts::default();
    for arg in args {
        match arg.as_str() {
            "-b" | "--binary" => opts.binary = true,
            "-c" | "--check" => opts.check = true,
            "--quiet" => opts.quiet = true,
            "--status" => opts.status = true,
            "-h" | "--help" => opts.help = true,
            s if !s.starts_with('-') => opts.files.push(s.to_string()),
            other => return Err(anyhow!("md5sum: unrecognized option '{other}'")),
        }
    }
    Ok(opts)
}

fn print_help() {
    println!("md5sum - compute and check MD5 message digest");
    println!("Usage: md5sum [OPTION]... [FILE]...");
    println!("       md5sum -c [OPTION]... [FILE]...");
    println!("Options:");
    println!("  -b, --binary        read files in binary mode (marker only)");
    println!("  -c, --check         read MD5 sums from the FILEs and check them");
    println!("      --quiet         don't print OK for each successfully verified file");
    println!("      --status        don't output anything, status code shows success");
    println!("  -h, --help          display this help and exit");
}

fn run_hash_mode(opts: &Opts) -> Result<()> {
    if opts.files.is_empty() {
        let hash = hash_reader_to_hex(&mut io::stdin().lock())?;
        let marker = if opts.binary { '*' } else { ' ' };
        println!("{hash}{marker}-");
        return Ok(());
    }
    for name in &opts.files {
        if name == "-" {
            let hash = hash_reader_to_hex(&mut io::stdin().lock())?;
            let marker = if opts.binary { '*' } else { ' ' };
            println!("{hash}{marker}-");
            continue;
        }
        match File::open(name) {
            Ok(mut f) => {
                let hash = hash_reader_to_hex(&mut f)?;
                let marker = if opts.binary { '*' } else { ' ' };
                println!("{hash}{marker}{name}");
            }
            Err(e) => eprintln!("md5sum: {name}: {e}"),
        }
    }
    Ok(())
}

fn run_check_mode(opts: &Opts) -> Result<()> {
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut open_failed = 0usize;

    if opts.files.is_empty() {
        verify_checksum_stream(&mut io::stdin().lock(), opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
    } else {
        for list_file in &opts.files {
            if list_file == "-" {
                verify_checksum_stream(&mut io::stdin().lock(), opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
                continue;
            }
            match File::open(list_file) {
                Ok(f) => {
                    let mut reader = BufReader::new(f);
                    verify_checksum_stream(&mut reader, opts, &mut total, &mut ok, &mut failed, &mut open_failed)?;
                }
                Err(e) => {
                    eprintln!("md5sum: {list_file}: {e}");
                    return Err(anyhow!("failed to open list file"));
                }
            }
        }
    }

    if !opts.status {
        if failed == 0 && open_failed == 0 {
            eprintln!("md5sum: OK");
        } else if failed > 0 || open_failed > 0 {
            eprintln!("md5sum: WARNING: {failed} mismatches, {open_failed} unreadable");
        }
    }

    if failed == 0 && open_failed == 0 { Ok(()) } else { Err(anyhow!("checksum verification failed")) }
}

fn verify_checksum_stream<R: BufRead>(reader: &mut R, opts: &Opts, total: &mut usize, ok: &mut usize, failed: &mut usize, open_failed: &mut usize) -> Result<()> {
    let mut line_buf = String::new();
    while {
        line_buf.clear();
        reader.read_line(&mut line_buf)? > 0
    } {
        let line = line_buf.trim_end_matches(['\n', '\r']);
        if line.is_empty() || line.starts_with('#') { continue; }
        if line.len() < 34 { continue; }
        let (hash_part, rest) = line.split_at(32); // MD5 hex length
        if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        let rest = rest.trim_start();
        if rest.is_empty() { continue; }
        let (mode_char, filename) = match rest.chars().next().unwrap() {
            '*' | ' ' => (rest.chars().next().unwrap(), &rest[1..]),
            _ => (' ', rest),
        };
        *total += 1;
        let fname_trim = filename.trim_start_matches([' ', '\t']);
        match File::open(fname_trim) {
            Ok(mut f) => match hash_reader_to_hex(&mut f) {
                Ok(actual) => {
                    if actual.eq_ignore_ascii_case(hash_part) {
                        *ok += 1;
                        if !opts.quiet && !opts.status { println!("{fname_trim}: OK"); }
                    } else {
                        *failed += 1;
                        if !opts.status { println!("{fname_trim}: FAILED"); }
                    }
                }
                Err(e) => {
                    *failed += 1;
                    if !opts.status { println!("{fname_trim}: FAILED ({e})"); }
                }
            },
            Err(e) => {
                *open_failed += 1;
                if !opts.status { println!("{fname_trim}: FAILED open ({e})"); }
            }
        }
        let _ = mode_char;
    }
    Ok(())
}

fn hash_reader_to_hex<R: Read>(reader: &mut R) -> Result<String> {
    let mut ctx = Md5Context::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).context("failed to read input")?;
        if n == 0 { break; }
        ctx.consume(&buf[..n]);
    }
    let digest: Digest = ctx.compute();
    // Clippy uninlined_format_args 対忁E
    Ok(format!("{digest:x}"))
}

