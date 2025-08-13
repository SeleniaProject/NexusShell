use anyhow::{Context, Result};
use serde_json::{self, Value};
use std::fs;
use std::path::Path;

/// Theme validation utilities
pub struct ThemeValidator {
    schema: Value,
}

impl ThemeValidator {
    /// Create a new theme validator with the official schema
    pub fn new() -> Result<Self> {
        let schema_path = "assets/themes/theme-schema.json";
        let schema_content = fs::read_to_string(schema_path)
            .context("Failed to read theme schema file")?;
        let schema: Value = serde_json::from_str(&schema_content)
            .context("Failed to parse theme schema")?;
        
        Ok(Self { schema })
    }
    
    /// Validate a theme file against the schema
    pub fn validate_theme_file<P: AsRef<Path>>(&self, theme_path: P) -> Result<ValidationResult> {
        let theme_content = fs::read_to_string(&theme_path)
            .with_context(|| format!("Failed to read theme file: {}", theme_path.as_ref().display()))?;
        
        let theme_value: Value = serde_json::from_str(&theme_content)
            .context("Failed to parse theme JSON")?;
        
        self.validate_theme_value(&theme_value)
    }
    
    /// Validate a theme JSON value against the schema
    pub fn validate_theme_value(&self, theme: &Value) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // Check required fields
        result.check_required_field(theme, "name");
        result.check_required_field(theme, "version");
        result.check_required_field(theme, "author");
        result.check_required_field(theme, "colors");
        
        // Validate version format
        if let Some(version) = theme.get("version").and_then(|v| v.as_str()) {
            if !Self::is_valid_semantic_version(version) {
                result.errors.push(format!("Invalid version format: '{}'. Expected semantic version (e.g., 1.0.0)", version));
            }
        }
        
        // Validate name format  
        if let Some(name) = theme.get("name").and_then(|v| v.as_str()) {
            if !Self::is_valid_theme_name(name) {
                result.errors.push(format!("Invalid theme name: '{}'. Must contain only alphanumeric characters, underscores, and dashes", name));
            }
        }
        
        // Validate colors and collect map for later lookup
        let mut color_hex_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if let Some(colors) = theme.get("colors").and_then(|v| v.as_object()) {
            let required_colors = ["primary", "secondary", "accent", "background", "foreground", "error", "warning", "success", "info"];
            
            for color_name in &required_colors {
                if let Some(color_value) = colors.get(*color_name).and_then(|v| v.as_str()) {
                    if !Self::is_valid_hex_color(color_value) {
                        result.errors.push(format!("Invalid hex color for '{}': '{}'. Expected format: #RRGGBB", color_name, color_value));
                    }
                    color_hex_map.insert((*color_name).to_string(), color_value.to_string());
                } else {
                    result.errors.push(format!("Missing required color: '{}'", color_name));
                }
            }
            // Keep any additional color keys as well if valid
            for (k, v) in colors.iter() {
                if let Some(hex) = v.as_str() {
                    if Self::is_valid_hex_color(hex) {
                        color_hex_map.entry(k.to_string()).or_insert_with(|| hex.to_string());
                    }
                }
            }
        }

        // Validate styles reference known color names, and check contrast for common pairs
        if let Some(styles) = theme.get("styles").and_then(|v| v.as_object()) {

            for (style_name, style_val) in styles {
                if let Some(style_obj) = style_val.as_object() {
                    let fg_ref = style_obj.get("foreground").and_then(|v| v.as_str());
                    let bg_ref = style_obj.get("background").and_then(|v| v.as_str());

                    // Validate references exist (in colors or named palette) or are hex
                    if let Some(name) = fg_ref {
                        if !name.is_empty() && Self::resolve_color_ref(name, &color_hex_map).is_none() {
                            result.errors.push(format!("Style '{}' references unknown foreground color '{}'.", style_name, name));
                        }
                    }
                    if let Some(name) = bg_ref {
                        if !name.is_empty() && Self::resolve_color_ref(name, &color_hex_map).is_none() {
                            result.errors.push(format!("Style '{}' references unknown background color '{}'.", style_name, name));
                        }
                    }

                    // If both resolvable, compute contrast and warn if < 4.5
                    if let (Some(fg_name), Some(bg_name)) = (fg_ref, bg_ref) {
                        if let (Some(fg_hex), Some(bg_hex)) = (
                            Self::resolve_color_ref(fg_name, &color_hex_map),
                            Self::resolve_color_ref(bg_name, &color_hex_map),
                        ) {
                            let ratio = Self::contrast_ratio_from_hex(&fg_hex, &bg_hex);
                            if ratio < 4.5 {
                                result.warnings.push(format!(
                                    "Style '{}' foreground '{}' on background '{}' has low contrast ({:.2}).",
                                    style_name, fg_name, bg_name, ratio
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Check if a string is a valid semantic version
    fn is_valid_semantic_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
    
    /// Check if a theme name is valid
    fn is_valid_theme_name(name: &str) -> bool {
        !name.is_empty() && 
        name.len() <= 50 &&
        name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
    
    /// Check if a string is a valid hex color
    fn is_valid_hex_color(color: &str) -> bool {
        if color.len() != 7 || !color.starts_with('#') {
            return false;
        }
        
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    // Resolve a color reference, accepting #RRGGBB, named basic colors, or keys from colors map
    fn resolve_color_ref(name: &str, color_hex_map: &std::collections::HashMap<String, String>) -> Option<String> {
        if Self::is_valid_hex_color(name) { return Some(name.to_string()); }
        if let Some(hex) = color_hex_map.get(name) { return Some(hex.clone()); }
        // Basic 8-color names mapping
        match name {
            "Black" => Some("#000000".to_string()),
            "Red" => Some("#FF0000".to_string()),
            "Green" => Some("#00FF00".to_string()),
            "Yellow" => Some("#FFFF00".to_string()),
            "Blue" => Some("#0000FF".to_string()),
            "Magenta" => Some("#FF00FF".to_string()),
            "Cyan" => Some("#00FFFF".to_string()),
            "White" => Some("#FFFFFF".to_string()),
            _ => None,
        }
    }

    // Convert #RRGGBB to linearized relative luminance and compute contrast ratio
    fn contrast_ratio_from_hex(fg: &str, bg: &str) -> f64 {
        fn hex_to_rgb(c: &str) -> (f64, f64, f64) {
            let r = u8::from_str_radix(&c[1..3], 16).unwrap_or(0) as f64 / 255.0;
            let g = u8::from_str_radix(&c[3..5], 16).unwrap_or(0) as f64 / 255.0;
            let b = u8::from_str_radix(&c[5..7], 16).unwrap_or(0) as f64 / 255.0;
            (r, g, b)
        }
        fn srgb_to_linear(u: f64) -> f64 {
            if u <= 0.03928 { u / 12.92 } else { ((u + 0.055) / 1.055).powf(2.4) }
        }
        fn luminance(r: f64, g: f64, b: f64) -> f64 {
            let (r, g, b) = (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b));
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }
        let (fr, fg_, fb) = hex_to_rgb(fg);
        let (br, bg_, bb) = hex_to_rgb(bg);
        let l1 = luminance(fr, fg_, fb);
        let l2 = luminance(br, bg_, bb);
        let (light, dark) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (light + 0.05) / (dark + 0.05)
    }
    
    /// List all available themes in the themes directory
    pub fn list_available_themes() -> Result<Vec<String>> {
        let themes_dir = "assets/themes";
        let mut themes = Vec::new();
        
        if let Ok(entries) = fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        if filename != "theme-schema" {
                            themes.push(filename.to_string());
                        }
                    }
                }
            }
        }
        
        themes.sort();
        Ok(themes)
    }
}

/// Theme validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    fn check_required_field(&mut self, theme: &Value, field: &str) {
        if theme.get(field).is_none() {
            self.errors.push(format!("Missing required field: '{}'", field));
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_semantic_version_validation() {
        assert!(ThemeValidator::is_valid_semantic_version("1.0.0"));
        assert!(ThemeValidator::is_valid_semantic_version("0.1.2"));
        assert!(ThemeValidator::is_valid_semantic_version("10.20.30"));
        
        assert!(!ThemeValidator::is_valid_semantic_version("1.0"));
        assert!(!ThemeValidator::is_valid_semantic_version("1.0.0.0"));
        assert!(!ThemeValidator::is_valid_semantic_version("v1.0.0"));
    }
    
    #[test]
    fn test_hex_color_validation() {
        assert!(ThemeValidator::is_valid_hex_color("#ffffff"));
        assert!(ThemeValidator::is_valid_hex_color("#000000"));
        assert!(ThemeValidator::is_valid_hex_color("#12abCD"));
        
        assert!(!ThemeValidator::is_valid_hex_color("ffffff"));
        assert!(!ThemeValidator::is_valid_hex_color("#fff"));
        assert!(!ThemeValidator::is_valid_hex_color("#gggggg"));
        assert!(!ThemeValidator::is_valid_hex_color("#12345"));
    }
    
    #[test]
    fn test_theme_name_validation() {
        assert!(ThemeValidator::is_valid_theme_name("nxsh-dark"));
        assert!(ThemeValidator::is_valid_theme_name("theme_123"));
        assert!(ThemeValidator::is_valid_theme_name("MyTheme"));
        
        assert!(!ThemeValidator::is_valid_theme_name(""));
        assert!(!ThemeValidator::is_valid_theme_name("theme with spaces"));
        assert!(!ThemeValidator::is_valid_theme_name("theme@special"));
    }
}
