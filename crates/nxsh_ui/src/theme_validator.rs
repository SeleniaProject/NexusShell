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
        
        // Validate colors
        if let Some(colors) = theme.get("colors").and_then(|v| v.as_object()) {
            let required_colors = ["primary", "secondary", "accent", "background", "foreground", "error", "warning", "success", "info"];
            
            for color_name in &required_colors {
                if let Some(color_value) = colors.get(*color_name).and_then(|v| v.as_str()) {
                    if !Self::is_valid_hex_color(color_value) {
                        result.errors.push(format!("Invalid hex color for '{}': '{}'. Expected format: #RRGGBB", color_name, color_value));
                    }
                } else {
                    result.errors.push(format!("Missing required color: '{}'", color_name));
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
