use std::fs;
use std::path::PathBuf;

use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
struct JobEntry {
    #[serde(rename = "in")] in_path: String,
    out: String,
}

#[derive(Deserialize)]
struct BatchConfig {
    font: Option<String>,
    size: Option<f32>,
    bg: Option<String>,
    cols: Option<usize>,
    #[serde(rename = "line_height")] line_height: Option<f32>,
    inputs: Vec<JobEntry>,
}

fn parse_hex_color(s: &str) -> Rgba<u8> {
    let s = s.trim();
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        return Rgba([r, g, b, 0xFF]);
    }
    Rgba([0x28, 0x28, 0x28, 0xFF])
}

fn main() -> anyhow::Result<()> {
    // Usage: cargo run -p nxsh_ui --bin ansi_to_png_batch -- --config assets/mockups/generate_pngs.json
    let mut config_path: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--config" { config_path = Some(PathBuf::from(args.next().expect("--config requires a path"))); }
    }
    let config_path = config_path.expect("--config is required");
    let data = fs::read_to_string(&config_path)?;
    let cfg: BatchConfig = serde_json::from_str(&data)?;

    let font_path = PathBuf::from(cfg.font.unwrap_or_else(|| "assets/fonts/JetBrainsMono-Regular.ttf".to_string()));
    let size = cfg.size.unwrap_or(18.0);
    let bg = cfg.bg.map(|s| parse_hex_color(&s)).unwrap_or(Rgba([0x28, 0x28, 0x28, 0xFF]));
    let cols = cfg.cols.unwrap_or(100);
    let line_height = cfg.line_height.unwrap_or(1.2);

    for job in cfg.inputs {
        let input = PathBuf::from(job.in_path);
        let output = PathBuf::from(job.out);
        nxsh_ui::ansi_render::render_ansi_file_to_png(
            &font_path, size, bg, cols, line_height, &input, &output,
        )?;
        println!("Saved {}", output.display());
    }
    Ok(())
}


