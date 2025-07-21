use anyhow::Result;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;

pub fn download_plugin(id: &str, dest: &str) -> Result<()> {
    let url = format!("https://example.com/plugins/v1/download/{}", id);
    let resp = Client::new().get(url).send()?;
    let bytes = resp.bytes()?;
    let mut f = File::create(dest)?;
    f.write_all(&bytes)?;
    Ok(())
} 