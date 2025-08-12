use anyhow::{Context, Result};
use crossterm::style::{Color, ContentStyle, Stylize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// Color scheme definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String, 
    pub accent: String,
    pub background: String,
    pub foreground: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub info: String,
    pub muted: String,
    pub highlight: String,
    pub border: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: "#3b82f6".to_string(),
            secondary: "#6b7280".to_string(),
            accent: "#f59e0b".to_string(),
            background: "#1f2937".to_string(),
            foreground: "#f9fafb".to_string(),
            error: "#ef4444".to_string(),
            warning: "#f59e0b".to_string(),
            success: "#10b981".to_string(),
            info: "#3b82f6".to_string(),
            muted: "#6b7280".to_string(),
            highlight: "#fbbf24".to_string(),
            border: "#374151".to_string(),
        }
    }
}

// Theme format definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeFormat {
    Json,
    Toml,
}

// Main theme structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusTheme {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub colors: ColorScheme,
    pub styles: HashMap<String, ContentStyle>,
}

impl Default for NexusTheme {
    fn default() -> Self {
        let mut styles = HashMap::new();
        
        // Default styles
        styles.insert("prompt".to_string(), ContentStyle::new().with(Color::Blue));
        styles.insert("command".to_string(), ContentStyle::new().with(Color::White));
        styles.insert("error".to_string(), ContentStyle::new().with(Color::Red));
        styles.insert("warning".to_string(), ContentStyle::new().with(Color::Yellow));
        styles.insert("success".to_string(), ContentStyle::new().with(Color::Green));
        styles.insert("info".to_string(), ContentStyle::new().with(Color::Cyan));
        styles.insert("muted".to_string(), ContentStyle::new().with(Color::DarkGrey));
        
        Self {
            name: "Dark".to_string(),
            version: "1.0.0".to_string(),
            author: "NexusShell".to_string(),
            description: "Default dark theme".to_string(),
            colors: ColorScheme::default(),
            styles,
        }
    }
}

impl NexusTheme {
    /// Load theme from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file: {:?}", path))?;
            
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap_or("") {
                "json" => {
                    serde_json::from_str(&content)
                        .with_context(|| "Failed to parse JSON theme")
                },
                "toml" => {
                    toml::from_str(&content)
                        .with_context(|| "Failed to parse TOML theme")
                },
                _ => Err(anyhow::anyhow!("Unsupported theme file format")),
            }
        } else {
            Err(anyhow::anyhow!("Theme file has no extension"))
        }
    }
    
    /// Save theme to file
    pub fn save_to_file(&self, path: &PathBuf, format: ThemeFormat) -> Result<()> {
        let content = match format {
            ThemeFormat::Json => serde_json::to_string_pretty(self)?,
            ThemeFormat::Toml => toml::to_string_pretty(self)?,
        };
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write theme file: {:?}", path))
    }
    
    /// Get a specific style
    pub fn get_style(&self, name: &str) -> Option<&ContentStyle> {
        self.styles.get(name)
    }
    
    /// Set a specific style
    pub fn set_style(&mut self, name: String, style: ContentStyle) {
        self.styles.insert(name, style);
    }
}

// Theme manager
#[derive(Debug)]
pub struct ThemeManager {
    current_theme: Arc<RwLock<NexusTheme>>,
    available_themes: Arc<RwLock<HashMap<String, NexusTheme>>>,
    theme_directory: PathBuf,
}

impl ThemeManager {
    /// Get theme directory path
    pub fn get_theme_directory() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("nexusshell").join("themes"))
    }
    
    /// Create new theme manager with complete functionality
    pub fn new() -> Result<Self> {
        let theme_directory = Self::get_theme_directory()
            .unwrap_or_else(|_| PathBuf::from("themes"));
        
        let manager = Self {
            current_theme: Arc::new(RwLock::new(NexusTheme::default())),
            available_themes: Arc::new(RwLock::new(HashMap::new())),
            theme_directory,
        };
        
        // Complete theme discovery and loading
        manager.discover_themes()?;
        Ok(manager)
    }
    
    /// Discover themes in the theme directory
    pub fn discover_themes(&self) -> Result<()> {
        // Create theme directory if it doesn't exist
        if !self.theme_directory.exists() {
            std::fs::create_dir_all(&self.theme_directory)?;
        }
        
        // Scan for theme files
        if let Ok(entries) = std::fs::read_dir(&self.theme_directory) {
            let mut themes = self.available_themes.write().unwrap();
            
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "toml" || ext == "json" {
                        if let Some(name) = entry.path().file_stem()
                            .and_then(|s| s.to_str()) {
                            // Load theme from file
                            if let Ok(theme) = NexusTheme::load_from_file(&entry.path()) {
                                themes.insert(name.to_string(), theme);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Result<NexusTheme> {
        let themes = self.available_themes.read().unwrap();
        themes.get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Theme '{}' not found", name))
    }

    /// Set the current theme
    pub fn set_theme(&self, theme_name: &str) -> Result<()> {
        let themes = self.available_themes.read().unwrap();
        if let Some(theme) = themes.get(theme_name) {
            let mut current = self.current_theme.write().unwrap();
            *current = theme.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
        }
    }

    /// Get the current theme
    pub fn get_current_theme(&self) -> NexusTheme {
        self.current_theme.read().unwrap().clone()
    }

    /// List available themes
    pub fn list_themes(&self) -> Vec<String> {
        self.available_themes.read().unwrap().keys().cloned().collect()
    }
}

// Display theme trait for UI components
#[derive(Debug, Clone)]
pub struct DisplayTheme {
    pub style_cache: HashMap<String, ContentStyle>,
    pub color_scheme: ColorScheme,
}

impl DisplayTheme {
    pub fn load_from_config() -> Result<Self> {
        Ok(Self {
            style_cache: HashMap::new(),
            color_scheme: ColorScheme::default(),
        })
    }
}
