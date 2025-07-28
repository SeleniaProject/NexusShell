//! Advanced theming system for NexusShell
//! 
//! This module provides comprehensive theming support with JSON/YAML configuration,
//! multiple built-in themes, runtime theme switching, and customizable color schemes.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use ansi_term::Colour;
use syntect::highlighting::{Theme, ThemeSet};

/// Main theme configuration for NexusShell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusTheme {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub ui_colors: UiColors,
    pub syntax_theme: String, // Name of the syntect theme
    pub custom_syntax_themes: Vec<PathBuf>,
}

impl Default for NexusTheme {
    fn default() -> Self {
        Self {
            name: "Dark".to_string(),
            description: "Default dark theme for NexusShell".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            ui_colors: UiColors::default(),
            syntax_theme: "base16-ocean.dark".to_string(),
            custom_syntax_themes: vec![],
        }
    }
}

impl NexusTheme {
    /// Load theme from a JSON/YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file: {}", path.display()))?;
        
        let theme = if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
                       path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML theme file: {}", path.display()))?
        } else {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON theme file: {}", path.display()))?
        };
        
        Ok(theme)
    }
    
    /// Save theme to a JSON/YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P, format: ThemeFormat) -> Result<()> {
        let path = path.as_ref();
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create theme directory: {}", parent.display()))?;
        }
        
        let content = match format {
            ThemeFormat::Json => serde_json::to_string_pretty(self)
                .context("Failed to serialize theme to JSON")?,
            ThemeFormat::Yaml => serde_yaml::to_string(self)
                .context("Failed to serialize theme to YAML")?,
        };
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write theme file: {}", path.display()))?;
        
        Ok(())
    }
    
    /// Get default theme file path
    pub fn default_theme_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("nexusshell").join("theme.json"))
    }
    
    /// Load default theme
    pub fn load_default() -> Result<Self> {
        if let Some(path) = Self::default_theme_path() {
            if path.exists() {
                return Self::load_from_file(path);
            }
        }
        Ok(Self::default())
    }
    
    /// Save default theme
    pub fn save_default(&self) -> Result<()> {
        if let Some(path) = Self::default_theme_path() {
            self.save_to_file(path, ThemeFormat::Json)?;
        }
        Ok(())
    }
    
    /// Get a syntect theme by name
    pub fn get_syntect_theme(&self, theme_name: &str) -> Result<Theme> {
        let theme_set = ThemeSet::load_defaults();
        
        // Try to find the theme by name
        if let Some(theme) = theme_set.themes.get(theme_name) {
            return Ok(theme.clone());
        }
        
        // Fallback to default theme
        if let Some(theme) = theme_set.themes.get("base16-ocean.dark") {
            return Ok(theme.clone());
        }
        
        // If all else fails, use the first available theme
        if let Some((_, theme)) = theme_set.themes.iter().next() {
            return Ok(theme.clone());
        }
        
        Err(anyhow::anyhow!("No syntect themes available"))
    }
    
    /// Get available syntect themes
    pub fn available_syntect_themes() -> Vec<String> {
        let theme_set = ThemeSet::load_defaults();
        theme_set.themes.keys().cloned().collect()
    }
}

/// Theme manager for handling multiple themes and theme switching
pub struct ThemeManager {
    current_theme: Arc<RwLock<NexusTheme>>,
    available_themes: Arc<RwLock<HashMap<String, NexusTheme>>>,
    theme_directory: PathBuf,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Result<Self> {
        let theme_directory = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexusshell")
            .join("themes");
        
        let mut manager = Self {
            current_theme: Arc::new(RwLock::new(NexusTheme::default())),
            available_themes: Arc::new(RwLock::new(HashMap::new())),
            theme_directory,
        };
        
        // Load default theme
        let default_theme = NexusTheme::load_default()?;
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *manager.current_theme.write().await = default_theme;
            })
        });
        
        // Discover available themes
        manager.discover_themes()?;
        
        Ok(manager)
    }
    
    /// Get the current theme
    pub fn current_theme(&self) -> NexusTheme {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.current_theme.read().await.clone()
            })
        })
    }
    
    /// Switch to a different theme
    pub async fn switch_theme(&self, theme_name: &str) -> Result<()> {
        let themes = self.available_themes.read().await;
        
        if let Some(theme) = themes.get(theme_name) {
            let mut current = self.current_theme.write().await;
            *current = theme.clone();
            
            // Save as default
            current.save_default()
                .context("Failed to save theme as default")?;
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
        }
    }
    
    /// Get list of available theme names
    pub async fn available_theme_names(&self) -> Vec<String> {
        let themes = self.available_themes.read().await;
        themes.keys().cloned().collect()
    }
    
    /// Install a new theme from file
    pub async fn install_theme<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let theme = NexusTheme::load_from_file(&path)?;
        let theme_name = theme.name.clone();
        
        // Copy theme to themes directory
        let theme_path = self.theme_directory.join(format!("{}.json", theme_name));
        theme.save_to_file(theme_path, ThemeFormat::Json)?;
        
        // Add to available themes
        let mut themes = self.available_themes.write().await;
        themes.insert(theme_name.clone(), theme);
        
        Ok(theme_name)
    }
    
    /// Remove a theme
    pub async fn remove_theme(&self, theme_name: &str) -> Result<()> {
        if theme_name == "Dark" || theme_name == "Light" {
            return Err(anyhow::anyhow!("Cannot remove built-in theme"));
        }
        
        let mut themes = self.available_themes.write().await;
        themes.remove(theme_name);
        
        // Remove theme file
        let theme_path = self.theme_directory.join(format!("{}.json", theme_name));
        if theme_path.exists() {
            fs::remove_file(theme_path)
                .context("Failed to remove theme file")?;
        }
        
        Ok(())
    }
    
    /// Create a custom theme
    pub async fn create_custom_theme(&self, name: String, base_theme: &str) -> Result<NexusTheme> {
        let themes = self.available_themes.read().await;
        let base = themes.get(base_theme)
            .ok_or_else(|| anyhow::anyhow!("Base theme '{}' not found", base_theme))?;
        
        let mut custom_theme = base.clone();
        custom_theme.name = name;
        custom_theme.description = format!("Custom theme based on {}", base_theme);
        custom_theme.author = "User".to_string();
        
        Ok(custom_theme)
    }
    
    /// Discover themes in the themes directory
    fn discover_themes(&mut self) -> Result<()> {
        // Add built-in themes
        let mut themes = HashMap::new();
        
        // Dark theme (default)
        themes.insert("Dark".to_string(), NexusTheme::default());
        
        // Light theme
        let light_theme = NexusTheme {
            name: "Light".to_string(),
            description: "Light theme for NexusShell".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            ui_colors: UiColors::light(),
            syntax_theme: "base16-ocean.light".to_string(),
            custom_syntax_themes: vec![],
        };
        themes.insert("Light".to_string(), light_theme);
        
        // Monokai theme
        let monokai_theme = NexusTheme {
            name: "Monokai".to_string(),
            description: "Monokai theme for NexusShell".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            ui_colors: UiColors::monokai(),
            syntax_theme: "Monokai".to_string(),
            custom_syntax_themes: vec![],
        };
        themes.insert("Monokai".to_string(), monokai_theme);
        
        // Solarized Dark theme
        let solarized_dark_theme = NexusTheme {
            name: "Solarized Dark".to_string(),
            description: "Solarized Dark theme for NexusShell".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            ui_colors: UiColors::solarized_dark(),
            syntax_theme: "Solarized (dark)".to_string(),
            custom_syntax_themes: vec![],
        };
        themes.insert("Solarized Dark".to_string(), solarized_dark_theme);
        
        // Create themes directory if it doesn't exist
        if !self.theme_directory.exists() {
            fs::create_dir_all(&self.theme_directory)
                .context("Failed to create themes directory")?;
        }
        
        // Discover user themes
        if self.theme_directory.exists() {
            for entry in fs::read_dir(&self.theme_directory)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") ||
                   path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
                   path.extension().and_then(|s| s.to_str()) == Some("yml") {
                    
                    match NexusTheme::load_from_file(&path) {
                        Ok(theme) => {
                            themes.insert(theme.name.clone(), theme);
                        }
                        Err(e) => {
                            eprintln!("Failed to load theme from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        // Update available themes
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *self.available_themes.write().await = themes;
            })
        });
        
        Ok(())
    }
}

/// UI specific color settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub background: RgbColor,
    pub foreground: RgbColor,
    pub primary: RgbColor,
    pub secondary: RgbColor,
    pub accent: RgbColor,
    pub error: RgbColor,
    pub warning: RgbColor,
    pub info: RgbColor,
    pub success: RgbColor,
    pub border: RgbColor,
    pub input_text: RgbColor,
    pub prompt: RgbColor,
    pub cursor: RgbColor,
    pub selection: RgbColor,
    pub scrollbar: RgbColor,
    pub status_bar_bg: RgbColor,
    pub status_bar_fg: RgbColor,
    pub side_panel_bg: RgbColor,
    pub side_panel_fg: RgbColor,
    pub suggestion_bg: RgbColor,
    pub suggestion_fg: RgbColor,
    pub highlight_bg: RgbColor,
    pub highlight_fg: RgbColor,
}

impl Default for UiColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl UiColors {
    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            background: RgbColor { r: 40, g: 44, b: 52 },
            foreground: RgbColor { r: 171, g: 178, b: 191 },
            primary: RgbColor { r: 97, g: 175, b: 239 },
            secondary: RgbColor { r: 198, g: 120, b: 221 },
            accent: RgbColor { r: 86, g: 182, b: 194 },
            error: RgbColor { r: 224, g: 108, b: 117 },
            warning: RgbColor { r: 229, g: 192, b: 123 },
            info: RgbColor { r: 97, g: 175, b: 239 },
            success: RgbColor { r: 152, g: 195, b: 121 },
            border: RgbColor { r: 76, g: 82, b: 99 },
            input_text: RgbColor { r: 171, g: 178, b: 191 },
            prompt: RgbColor { r: 97, g: 175, b: 239 },
            cursor: RgbColor { r: 97, g: 175, b: 239 },
            selection: RgbColor { r: 76, g: 82, b: 99 },
            scrollbar: RgbColor { r: 76, g: 82, b: 99 },
            status_bar_bg: RgbColor { r: 33, g: 37, b: 43 },
            status_bar_fg: RgbColor { r: 171, g: 178, b: 191 },
            side_panel_bg: RgbColor { r: 33, g: 37, b: 43 },
            side_panel_fg: RgbColor { r: 171, g: 178, b: 191 },
            suggestion_bg: RgbColor { r: 76, g: 82, b: 99 },
            suggestion_fg: RgbColor { r: 171, g: 178, b: 191 },
            highlight_bg: RgbColor { r: 97, g: 175, b: 239 },
            highlight_fg: RgbColor { r: 40, g: 44, b: 52 },
        }
    }
    
    /// Light theme colors
    pub fn light() -> Self {
        Self {
            background: RgbColor { r: 250, g: 250, b: 250 },
            foreground: RgbColor { r: 56, g: 58, b: 66 },
            primary: RgbColor { r: 64, g: 120, b: 242 },
            secondary: RgbColor { r: 166, g: 38, b: 164 },
            accent: RgbColor { r: 1, g: 132, b: 188 },
            error: RgbColor { r: 202, g: 18, b: 67 },
            warning: RgbColor { r: 152, g: 104, b: 1 },
            info: RgbColor { r: 64, g: 120, b: 242 },
            success: RgbColor { r: 80, g: 161, b: 79 },
            border: RgbColor { r: 200, g: 200, b: 200 },
            input_text: RgbColor { r: 56, g: 58, b: 66 },
            prompt: RgbColor { r: 64, g: 120, b: 242 },
            cursor: RgbColor { r: 64, g: 120, b: 242 },
            selection: RgbColor { r: 200, g: 200, b: 200 },
            scrollbar: RgbColor { r: 200, g: 200, b: 200 },
            status_bar_bg: RgbColor { r: 240, g: 240, b: 240 },
            status_bar_fg: RgbColor { r: 56, g: 58, b: 66 },
            side_panel_bg: RgbColor { r: 240, g: 240, b: 240 },
            side_panel_fg: RgbColor { r: 56, g: 58, b: 66 },
            suggestion_bg: RgbColor { r: 200, g: 200, b: 200 },
            suggestion_fg: RgbColor { r: 56, g: 58, b: 66 },
            highlight_bg: RgbColor { r: 64, g: 120, b: 242 },
            highlight_fg: RgbColor { r: 250, g: 250, b: 250 },
        }
    }
    
    /// Monokai theme colors
    pub fn monokai() -> Self {
        Self {
            background: RgbColor { r: 39, g: 40, b: 34 },
            foreground: RgbColor { r: 248, g: 248, b: 242 },
            primary: RgbColor { r: 102, g: 217, b: 239 },
            secondary: RgbColor { r: 166, g: 226, b: 46 },
            accent: RgbColor { r: 249, g: 38, b: 114 },
            error: RgbColor { r: 249, g: 38, b: 114 },
            warning: RgbColor { r: 230, g: 219, b: 116 },
            info: RgbColor { r: 102, g: 217, b: 239 },
            success: RgbColor { r: 166, g: 226, b: 46 },
            border: RgbColor { r: 73, g: 72, b: 62 },
            input_text: RgbColor { r: 248, g: 248, b: 242 },
            prompt: RgbColor { r: 102, g: 217, b: 239 },
            cursor: RgbColor { r: 102, g: 217, b: 239 },
            selection: RgbColor { r: 73, g: 72, b: 62 },
            scrollbar: RgbColor { r: 73, g: 72, b: 62 },
            status_bar_bg: RgbColor { r: 30, g: 31, b: 26 },
            status_bar_fg: RgbColor { r: 248, g: 248, b: 242 },
            side_panel_bg: RgbColor { r: 30, g: 31, b: 26 },
            side_panel_fg: RgbColor { r: 248, g: 248, b: 242 },
            suggestion_bg: RgbColor { r: 73, g: 72, b: 62 },
            suggestion_fg: RgbColor { r: 248, g: 248, b: 242 },
            highlight_bg: RgbColor { r: 102, g: 217, b: 239 },
            highlight_fg: RgbColor { r: 39, g: 40, b: 34 },
        }
    }
    
    /// Solarized Dark theme colors
    pub fn solarized_dark() -> Self {
        Self {
            background: RgbColor { r: 0, g: 43, b: 54 },
            foreground: RgbColor { r: 131, g: 148, b: 150 },
            primary: RgbColor { r: 38, g: 139, b: 210 },
            secondary: RgbColor { r: 211, g: 54, b: 130 },
            accent: RgbColor { r: 42, g: 161, b: 152 },
            error: RgbColor { r: 220, g: 50, b: 47 },
            warning: RgbColor { r: 181, g: 137, b: 0 },
            info: RgbColor { r: 38, g: 139, b: 210 },
            success: RgbColor { r: 133, g: 153, b: 0 },
            border: RgbColor { r: 7, g: 54, b: 66 },
            input_text: RgbColor { r: 131, g: 148, b: 150 },
            prompt: RgbColor { r: 38, g: 139, b: 210 },
            cursor: RgbColor { r: 38, g: 139, b: 210 },
            selection: RgbColor { r: 7, g: 54, b: 66 },
            scrollbar: RgbColor { r: 7, g: 54, b: 66 },
            status_bar_bg: RgbColor { r: 7, g: 54, b: 66 },
            status_bar_fg: RgbColor { r: 131, g: 148, b: 150 },
            side_panel_bg: RgbColor { r: 7, g: 54, b: 66 },
            side_panel_fg: RgbColor { r: 131, g: 148, b: 150 },
            suggestion_bg: RgbColor { r: 7, g: 54, b: 66 },
            suggestion_fg: RgbColor { r: 131, g: 148, b: 150 },
            highlight_bg: RgbColor { r: 38, g: 139, b: 210 },
            highlight_fg: RgbColor { r: 0, g: 43, b: 54 },
        }
    }
}

/// RGB color struct
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<RgbColor> for Colour {
    fn from(color: RgbColor) -> Self {
        Colour::RGB(color.r, color.g, color.b)
    }
}

impl From<Colour> for RgbColor {
    fn from(color: Colour) -> Self {
        match color {
            Colour::RGB(r, g, b) => Self { r, g, b },
            _ => Self { r: 255, g: 255, b: 255 }, // Default to white
        }
    }
}

/// Theme file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFormat {
    Json,
    Yaml,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_theme_creation() {
        let theme = NexusTheme::default();
        assert_eq!(theme.name, "Dark");
        assert!(!theme.ui_colors.background.r == 0 || !theme.ui_colors.background.g == 0 || !theme.ui_colors.background.b == 0);
    }
    
    #[test]
    fn test_theme_serialization() {
        let theme = NexusTheme::default();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: NexusTheme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }
    
    #[tokio::test]
    async fn test_theme_manager_creation() {
        let manager = ThemeManager::new().unwrap();
        let current = manager.current_theme();
        assert!(!current.name.is_empty());
    }
    
    #[tokio::test]
    async fn test_theme_switching() {
        let manager = ThemeManager::new().unwrap();
        let themes = manager.available_theme_names().await;
        assert!(!themes.is_empty());
        
        if themes.contains(&"Light".to_string()) {
            manager.switch_theme("Light").await.unwrap();
            let current = manager.current_theme();
            assert_eq!(current.name, "Light");
        }
    }
    
    #[test]
    fn test_ui_colors() {
        let dark_colors = UiColors::dark();
        let light_colors = UiColors::light();
        
        // Dark theme should have dark background
        assert!(dark_colors.background.r < 128);
        assert!(dark_colors.background.g < 128);
        assert!(dark_colors.background.b < 128);
        
        // Light theme should have light background
        assert!(light_colors.background.r > 128);
        assert!(light_colors.background.g > 128);
        assert!(light_colors.background.b > 128);
    }
    
    #[test]
    fn test_color_conversion() {
        let rgb_color = RgbColor { r: 255, g: 128, b: 64 };
        let ansi_color: Colour = rgb_color.into();
        let converted_back: RgbColor = ansi_color.into();
        
        assert_eq!(rgb_color.r, converted_back.r);
        assert_eq!(rgb_color.g, converted_back.g);
        assert_eq!(rgb_color.b, converted_back.b);
    }
} 