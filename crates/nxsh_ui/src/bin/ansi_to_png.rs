use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ab_glyph::{FontRef, PxScale};
use image::Rgba;
use nxsh_ui::ansi_render::load_font;

/// Parse a minimal subset of ANSI for colors. This is a lightweight best-effort renderer.
/// It supports SGR codes for foreground colors (30-37,90-97) and reset (0), bold (1).
/// This is not a full terminal emulator; the goal is to render our mockup text to PNG.
#[derive(Clone, Copy, Debug)]
struct AnsiStyle {
    fg: Rgba<u8>,
    bold: bool,
}

impl Default for AnsiStyle {
    fn default() -> Self {
        Self { fg: Rgba([0xEE, 0xEE, 0xEE, 0xFF]), bold: false }
    }
}

fn color_from_code(code: i32) -> Rgba<u8> {
    match code {
        30 => Rgba([0x00, 0x00, 0x00, 0xFF]), // black
        31 => Rgba([0xCC, 0x24, 0x1D, 0xFF]), // red
        32 => Rgba([0x98, 0x97, 0x1A, 0xFF]), // green
        33 => Rgba([0xD7, 0x99, 0x21, 0xFF]), // yellow
        34 => Rgba([0x45, 0x85, 0x88, 0xFF]), // blue
        35 => Rgba([0xB1, 0x62, 0x86, 0xFF]), // magenta
        36 => Rgba([0x68, 0x9D, 0x6A, 0xFF]), // cyan
        37 => Rgba([0xEE, 0xEE, 0xEE, 0xFF]), // white
        90 => Rgba([0x66, 0x66, 0x66, 0xFF]), // bright black
        91 => Rgba([0xFB, 0x49, 0x34, 0xFF]), // bright red
        92 => Rgba([0xB8, 0xBB, 0x26, 0xFF]), // bright green
        93 => Rgba([0xFA, 0xBD, 0x2F, 0xFF]), // bright yellow
        94 => Rgba([0x83, 0xA5, 0x98, 0xFF]), // bright blue
        95 => Rgba([0xD3, 0x86, 0x9B, 0xFF]), // bright magenta
        96 => Rgba([0x8E, 0xC0, 0x7C, 0xFF]), // bright cyan
        97 => Rgba([0xFF, 0xFF, 0xFF, 0xFF]), // bright white
        _ => Rgba([0xEE, 0xEE, 0xEE, 0xFF]),
    }
}

#[allow(dead_code)]
fn parse_ansi_segments(line: &str) -> Vec<(AnsiStyle, String)> {
    // Very simple parser: split by ESC[ ... m sequences and apply style.
    let mut segments = Vec::new();
    let mut style = AnsiStyle::default();
    let mut buf = String::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // flush current buffer
            if !buf.is_empty() {
                segments.push((style, buf.clone()));
                buf.clear();
            }
            // parse until 'm'
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'm' { j += 1; }
            if j < bytes.len() && bytes[j] == b'm' {
                let code_str = &line[i + 2..j];
                for part in code_str.split(';') {
                    if let Ok(code) = part.parse::<i32>() {
                        match code {
                            0 => { style = AnsiStyle::default(); }
                            1 => { style.bold = true; }
                            30..=37 | 90..=97 => { style.fg = color_from_code(code); }
                            _ => {}
                        }
                    }
                }
                i = j + 1;
                continue;
            }
        }
        buf.push(bytes[i] as char);
        i += 1;
    }
    if !buf.is_empty() { segments.push((style, buf)); }
    segments
}

// Kept for backward compatibility: re-export through module use

// Rendering moved to nxsh_ui::ansi_render

fn main() -> anyhow::Result<()> {
    // Arguments: --font <ttf/otf> --size <px> --bg #RRGGBB --in <ans> --out <png> --cols 100 --line-height 1.2
    let mut font_path: Option<PathBuf> = None;
    let mut size: f32 = 18.0;
    let mut bg = Rgba([0x28, 0x28, 0x28, 0xFF]);
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut cols: usize = 100;
    let mut line_height: f32 = 1.2;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--font" => font_path = Some(PathBuf::from(args.next().expect("--font needs a path"))),
            "--size" => size = args.next().unwrap().parse().unwrap_or(18.0),
            "--bg" => {
                let s = args.next().unwrap();
                if let Some(hex) = s.strip_prefix('#') {
                    if hex.len() == 6 {
                        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0x28);
                        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0x28);
                        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0x28);
                        bg = Rgba([r, g, b, 0xFF]);
                    }
                }
            }
            "--in" => input = Some(PathBuf::from(args.next().expect("--in needs a path"))),
            "--out" => output = Some(PathBuf::from(args.next().expect("--out needs a path"))),
            "--cols" => cols = args.next().unwrap().parse().unwrap_or(100),
            "--line-height" => line_height = args.next().unwrap().parse().unwrap_or(1.2),
            _ => {}
        }
    }

    let font_path = font_path.unwrap_or_else(|| PathBuf::from("assets/fonts/JetBrainsMono-Regular.ttf"));
    let input = input.expect("--in is required");
    let output = output.expect("--out is required");

    let font = load_font(&font_path)?;
    let scale = PxScale { x: size, y: size };

    let file = fs::File::open(&input)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Determine image dimensions. Assume monospace approx width per char ~ 0.6 * size.
    let img = nxsh_ui::ansi_render::render_lines_to_image(&font, size, bg, cols, line_height, &lines);
    img.save(&output)?;
    println!("Saved {}", output.display());
    Ok(())
}


