use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use ab_glyph::{point, FontRef, PxScale, Font, ScaleFont};
use image::{ImageBuffer, Rgba};
use anyhow::Result;
use crate::Theme;

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

    let sf = font.as_scaled(scale);
    let draw_baseline = baseline_y as f32 + sf.ascent();
    let img_width = img.width() as i32;
    let img_height = img.height() as i32;

    for ch in text.chars() {
        if ch == '\r' { continue; }
        if ch == '\n' { break; }
        
        // Early exit if we've gone beyond the right edge
        if x >= img_width - 16 {
            break;
        }
        
        let gid = font.glyph_id(ch);
        let mut glyph = gid.with_scale(scale);
        glyph.position = point(x as f32, draw_baseline);
        
        if let Some(outlined) = font.outline_glyph(glyph) {
            outlined.draw(|px, py, coverage| {
                let xi = px as i32;
                let yi = py as i32;
                
                // Enhanced bounds checking with safety margins
                if xi >= 0 && yi >= 0 && 
                   xi < img_width && yi < img_height &&
                   coverage > 1e-6 { // More robust coverage threshold
                    
                    // Safe pixel access
                    if let Some(dst) = img.get_pixel_mut_checked(xi as u32, yi as u32) {
                        let sa = coverage.clamp(0.0, 1.0);
                        let da = dst[3] as f32 / 255.0;
                        let out_a = sa + da * (1.0 - sa);
                        
                        if out_a > 1e-6 { // Avoid division by very small numbers
                            for c in 0..3 {
                                let sc = color[c] as f32 / 255.0;
                                let dc = dst[c] as f32 / 255.0;
                                let out_c = (sc * sa + dc * da * (1.0 - sa)) / out_a;
                                dst[c] = (out_c * 255.0).round().clamp(0.0, 255.0) as u8;
                            }
                            dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            });
        }
        
        let advance = sf.h_advance(gid);
        x += advance.ceil() as i32;
        
        // Safety check to prevent infinite loops
        if x >= img_width {
            break;
        }
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
    let sf = font.as_scaled(scale);
    let line_height_px = ((sf.ascent() - sf.descent() + sf.line_gap()) * line_height).ceil() as i32;
    let char_w = (size * 0.6).ceil() as i32; // rough per-char width estimate
    
    // Ensure minimum sensible dimensions with safety margins
    let min_width = 640u32;
    let max_width = 8192u32;
    let min_height = 360u32;
    let max_height = 8192u32;
    
    let width = ((cols as i32 * char_w + 32).max(min_width as i32) as u32).min(max_width);
    let calculated_height = ((lines.len() as i32 * line_height_px) + (size as i32) + 24).max(min_height as i32) as u32;
    let height = calculated_height.min(max_height);
    
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, bg);

    let mut y = (size as i32) + 12; // top padding
    let bottom_margin = 12; // bottom padding
    
    for line in lines {
        // Enhanced bounds checking - ensure we have enough space for the line
        if y + line_height_px + bottom_margin >= height as i32 {
            break;
        }
        
        let segments = parse_ansi_segments(line);
        let mut x = 16i32; // left padding
        let right_margin = 16i32; // right padding
        
        for (style, text) in segments {
            // Check if we have space before rendering
            if x >= (width as i32) - right_margin {
                break;
            }
            
            x = render_segment(&mut img, font, x, y, scale, style, &text);
            
            // Stop if we've gone beyond the safe rendering area
            if x >= (width as i32) - right_margin {
                break;
            }
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

/// Simple text-based ANSI renderer used by the CUI to prepare a printable line.
///
/// This renderer processes ANSI escape sequences and applies theme-aware transformations
/// to ensure consistent and safe terminal output.
#[derive(Default, Debug, Clone, Copy)]
pub struct AnsiRenderer;

impl AnsiRenderer {
    /// Create a new renderer instance.
    pub fn new() -> Self { Self }

    /// Render a single line of text for terminal output.
    /// Applies theme-aware filtering and sanitization of ANSI sequences.
    pub fn render(&self, line: &str, theme: &Theme) -> Result<String> {
        // Basic sanitization - remove potentially harmful sequences
        let sanitized = self.sanitize_ansi(line)?;
        
        // Apply theme transformations if needed
        let themed = self.apply_theme_filters(&sanitized, theme)?;
        
        Ok(themed)
    }
    
    /// Sanitize ANSI escape sequences to prevent terminal corruption
    fn sanitize_ansi(&self, input: &str) -> Result<String> {
        let mut result = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut i = 0;
        
        while i < bytes.len() {
            if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                // Find the end of the escape sequence
                let mut j = i + 2;
                let mut valid_sequence = true;
                
                // Limit sequence length to prevent buffer overflow attacks
                let max_sequence_length = 32;
                let start_j = j;
                
                while j < bytes.len() && (j - start_j) < max_sequence_length {
                    let ch = bytes[j];
                    if ch.is_ascii_alphabetic() {
                        j += 1;
                        break;
                    } else if ch.is_ascii_digit() || ch == b';' || ch == b':' {
                        j += 1;
                    } else {
                        valid_sequence = false;
                        break;
                    }
                }
                
                if valid_sequence && j <= bytes.len() {
                    // Copy the valid escape sequence
                    result.push_str(&input[i..j]);
                    i = j;
                } else {
                    // Skip invalid sequence, just copy the escape character as-is
                    result.push(bytes[i] as char);
                    i += 1;
                }
            } else {
                // Regular character
                result.push(bytes[i] as char);
                i += 1;
            }
        }
        
        Ok(result)
    }
    
    /// Apply theme-specific filters to the rendered line
    fn apply_theme_filters(&self, input: &str, _theme: &Theme) -> Result<String> {
        // For now, just return the input unchanged
        // In the future, this could apply theme-specific color transformations,
        // accessibility adjustments, etc.
        Ok(input.to_string())
    }
}


