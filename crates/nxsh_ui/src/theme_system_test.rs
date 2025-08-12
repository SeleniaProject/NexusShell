//! Theme System Test Suite
//! 
//! Comprehensive testing suite for NexusShell's advanced theming system.
//! Tests theme loading, switching, customization, and syntax highlighting integration.

use anyhow::Result;
use nxsh_ui::themes::{NexusTheme, ThemeFormat};
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("🎨 NexusShell Theme Quick Check");

    // 1) デフォルトテーマの基本検証
    let theme = NexusTheme::default();
    assert_eq!(theme.name, "Dark");
    assert!(!theme.description.is_empty());
    assert!(theme.get_style("prompt").is_some(), "default theme should provide 'prompt' style");

    // 2) JSON/TOML での保存・読込の往復
    let tmp = TempDir::new()?;
    let json_path = tmp.path().join("theme.json");
    let toml_path = tmp.path().join("theme.toml");

    // JSON
    theme.save_to_file(&json_path, ThemeFormat::Json)?;
    let loaded_json = NexusTheme::load_from_file(&json_path)?;
    assert_eq!(loaded_json.name, theme.name);
    assert_eq!(loaded_json.description, theme.description);

    // TOML
    theme.save_to_file(&toml_path, ThemeFormat::Toml)?;
    let loaded_toml = NexusTheme::load_from_file(&toml_path)?;
    assert_eq!(loaded_toml.name, theme.name);
    assert_eq!(loaded_toml.description, theme.description);

    println!("✅ Theme example OK");
    Ok(())
}
