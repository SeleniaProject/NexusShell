use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ab_glyph::{point, FontRef, PxScale};
use image::{ImageBuffer, Rgba};

/// Represents a minimal ANSI style for foreground color and bold attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnsiStyle {
    pub fg: Rgba<u8>,
    pub bold: bool,
}

impl Default for AnsiStyle {
    fn default() -> Self {
        Self { fg: Rgba([0xEE, 0xEE, 0xEE, 0xFF]), bold: false }
    }
}

/// Map a limited set of SGR color codes (30-37, 90-97) to RGBA colors.
pub fn color_from_code(code: i32) -> Rgba<u8> {
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

/// Parse a line containing SGR sequences to styled text segments.
/// Supports reset (0), bold (1), and 16 foreground colors.
pub fn parse_ansi_segments(line: &str) -> Vec<(AnsiStyle, String)> {
    let mut segments = Vec::new();
    let mut style = AnsiStyle::default();
    let mut buf = String::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if !buf.is_empty() {
                segments.push((style, buf.clone()));
                buf.clear();
            }
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

/// Load a font file into ab_glyph FontRef.
pub fn load_font(font_path: &Path) -> anyhow::Result<FontRef<'static>> {
    let data = fs::read(font_path)?;
    let owned: &'static [u8] = Box::leak(data.into_boxed_slice());
    let font = FontRef::try_from_slice(owned)?;
    Ok(font)
}

/// Render one styled text segment and return the new x position.
fn render_segment(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &FontRef<'_>,
    mut x: i32,
    baseline_y: i32,
    scale: PxScale,
    style: AnsiStyle,
    text: &str,
) -> i32 {
    let color = if style.bold {
        Rgba([
            style.fg[0].saturating_add(16),
            style.fg[1].saturating_add(16),
            style.fg[2].saturating_add(16),
            0xFF,
        ])
    } else {
        style.fg
    };

    let v = font.v_metrics(scale);
    let draw_baseline = baseline_y as f32 + v.ascent;

    for ch in text.chars() {
        if ch == '\r' { continue; }
        if ch == '\n' { break; }
        let gid = font.glyph_id(ch);
        let glyph = gid.with_scale(scale).with_position(point(x as f32, draw_baseline));
        if let Some(outlined) = font.outline_glyph(glyph) {
            outlined.draw(|px, py, coverage| {
                let xi = px as i32;
                let yi = py as i32;
                if xi >= 0 && yi >= 0 && (xi as u32) < img.width() && (yi as u32) < img.height() {
                    let dst = img.get_pixel_mut(xi as u32, yi as u32);
                    let sa = coverage as f32; // 0.0..1.0
                    let da = dst[3] as f32 / 255.0;
                    let out_a = sa + da * (1.0 - sa);
                    for c in 0..3 {
                        let sc = color[c] as f32 / 255.0;
                        let dc = dst[c] as f32 / 255.0;
                        let out_c = (sc * sa + dc * da * (1.0 - sa)) / out_a.max(1e-6);
                        dst[c] = (out_c * 255.0).min(255.0).max(0.0) as u8;
                    }
                    dst[3] = (out_a * 255.0).min(255.0) as u8;
                }
            });
        }
        let advance = font.h_advance(gid) * scale.x;
        x += advance.ceil() as i32;
    }
    x
}

/// Render multiple lines (already read) to an RGBA image with specified layout.
pub fn render_lines_to_image(
    font: &FontRef<'_>,
    size: f32,
    bg: Rgba<u8>,
    cols: usize,
    line_height: f32,
    lines: &[String],
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let scale = PxScale { x: size, y: size };
    let v = font.v_metrics(scale);
    let line_height_px = ((v.ascent - v.descent + v.line_gap) * line_height).ceil() as i32;
    let char_w = (size * 0.6).ceil() as i32; // rough per-char width estimate
    let width = (cols as i32 * char_w + 32).max(640) as u32;
    let height = ((lines.len() as i32 * line_height_px) + (size as i32) + 24).max(360) as u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, bg);

    let mut y = (size as i32) + 12; // top padding
    for line in lines {
        let segments = parse_ansi_segments(line);
        let mut x = 16i32; // left padding
        for (style, text) in segments {
            x = render_segment(&mut img, font, x, y, scale, style, &text);
        }
        y += line_height_px;
    }
    img
}

/// Convenience: read from an .ans file and save a PNG.
pub fn render_ansi_file_to_png(
    font_path: &Path,
    size: f32,
    bg: Rgba<u8>,
    cols: usize,
    line_height: f32,
    input: &Path,
    output: &Path,
) -> anyhow::Result<()> {
    let font = load_font(font_path)?;
    let file = fs::File::open(input)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let img = render_lines_to_image(&font, size, bg, cols, line_height, &lines);
    img.save(output)?;
    Ok(())
}


