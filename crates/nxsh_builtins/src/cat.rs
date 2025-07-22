//! `cat` command – concatenate files and print on the standard output.
//! Usage: cat [FILE...]
//! If no FILE or FILE is '-', read from stdin.

use anyhow::Result;
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, fs::File};
use tokio::task;

pub async fn cat_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        copy_stream(&mut io::stdin(), &mut io::stdout()).await?;
        return Ok(());
    }
    for f in args {
        if f == "-" {
            copy_stream(&mut io::stdin(), &mut io::stdout()).await?;
        } else {
            let mut file = File::open(f).await?;
            copy_stream(&mut file, &mut io::stdout()).await?;
        }
    }
    Ok(())
}

async fn copy_stream<R, W>(reader: &mut R, writer: &mut W) -> Result<()> where R: AsyncReadExt + Unpin, W: AsyncWriteExt + Unpin {
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }
        writer.write_all(&buf[..n]).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; use tempfile::NamedTempFile; use std::io::Write;
#[tokio::test]
async fn cat_file(){ let mut f=NamedTempFile::new().unwrap(); writeln!(f,"hi").unwrap(); cat_cli(&[f.path().to_string_lossy().into()]).await.unwrap(); }} 