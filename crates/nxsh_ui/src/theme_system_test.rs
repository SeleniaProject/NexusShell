//! Theme System Test Suite
//! 
//! Comprehensive testing suite for NexusShell's advanced theming system.
//! Tests theme loading, switching, customization, and syntax highlighting integration.

use anyhow::Result;
use serde_json;
use tempfile::TempDir;

// Import from the nxsh_ui crate
use nxsh_ui::themes::{NexusTheme, UiColors, ThemeConfig, SyntaxElement, ThemeManager, ThemeFormat, RgbColor};
use nxsh_ui::highlighting::SyntaxHighlighter;

/// Comprehensive theme system test
pub fn test_theme_system() -> Result<()> {
    println!("🎨 NexusShell Theme System Test Suite");
    println!("=====================================");

    // Test 1: Theme Creation and Configuration
    test_theme_creation()?;
    
    // Test 2: Theme Manager Operations
    test_theme_manager()?;
    
    // Test 3: Syntax Highlighting Integration
    test_syntax_highlighting_integration()?;
    
    // Test 4: Theme Persistence
    test_theme_persistence()?;
    
    // Test 5: Pure Rust Theme Configuration
    test_pure_rust_themes()?;
    
    // Test 6: Color System
    test_color_system()?;
    
    // Test 7: Custom Theme Creation
    test_custom_theme_creation()?;

    println!("\n✅ All theme system tests completed successfully!");
    println!("🎨 Theme System: Production Ready");
    
    Ok(())
}

/// Test theme creation and basic configuration
fn test_theme_creation() -> Result<()> {
    println!("\n🎯 Test 1: Theme Creation and Configuration");
    
    // Test default theme creation
    let default_theme = NexusTheme::default();
    assert_eq!(default_theme.name, "Dark");
    assert!(!default_theme.description.is_empty());
    assert!(!default_theme.author.is_empty());
    
    // Test UI colors
    let ui_colors = UiColors::dark();
    assert!(ui_colors.background.r < 128, "Dark theme should have dark background");
    
    let light_colors = UiColors::light();
    assert!(light_colors.background.r > 128, "Light theme should have light background");
    
    // Test theme config creation
    let theme_config = ThemeConfig::base16_ocean_dark();
    assert_eq!(theme_config.name, "Base16 Ocean Dark");
    
    // Test syntax element colors
    let keyword_color = theme_config.get_syntax_color(SyntaxElement::Keyword);
    let string_color = theme_config.get_syntax_color(SyntaxElement::String);
    assert_ne!(keyword_color.r, string_color.r, "Keywords and strings should have different colors");
    
    println!("  ✅ Theme creation and configuration working correctly");
    
    Ok(())
}

/// Test theme manager operations
fn test_theme_manager() -> Result<()> {
    println!("\n🎯 Test 2: Theme Manager Operations");
    
    tokio::runtime::Runtime::new()?.block_on(async {
        // Create theme manager
        let manager = ThemeManager::new()?;
        
        // Test current theme
        let current = manager.current_theme();
        assert!(!current.name.is_empty());
        
        // Test available themes
        let themes = manager.available_theme_names().await;
        assert!(!themes.is_empty());
        assert!(themes.contains(&"Dark".to_string()));
        assert!(themes.contains(&"Light".to_string()));
        assert!(themes.contains(&"Monokai".to_string()));
        assert!(themes.contains(&"Solarized Dark".to_string()));
        
        println!("  📚 Available themes: {:?}", themes);
        
        // Test theme switching
        if themes.contains(&"Light".to_string()) {
            manager.switch_theme("Light").await?;
            let current = manager.current_theme();
            assert_eq!(current.name, "Light");
            
            // Switch back to dark
            manager.switch_theme("Dark").await?;
            let current = manager.current_theme();
            assert_eq!(current.name, "Dark");
        }
        
        println!("  ✅ Theme manager operations working correctly");
        
        Ok::<(), anyhow::Error>(())
    })?;
    
    Ok(())
}

/// Test syntax highlighting integration
fn test_syntax_highlighting_integration() -> Result<()> {
    println!("\n🎯 Test 3: Syntax Highlighting Integration");
    
    // Create theme configurations
    let dark_config = ThemeConfig::base16_ocean_dark();
    let light_config = ThemeConfig::base16_ocean_light();
    let monokai_config = ThemeConfig::base16_mocha_dark();
    let solarized_config = ThemeConfig::solarized_dark();
    let github_config = ThemeConfig::inspired_github();
    
    // Test theme configs
    assert_eq!(dark_config.name, "Base16 Ocean Dark");
    assert_eq!(light_config.name, "Base16 Ocean Light");
    assert_eq!(monokai_config.name, "Base16 Mocha Dark");
    assert_eq!(solarized_config.name, "Solarized Dark");
    assert_eq!(github_config.name, "Inspired GitHub");
    
    // Test syntax highlighter with themes
    let mut highlighter = SyntaxHighlighter::new()?;
    
    // Test with different themes
    highlighter.set_theme(&dark_config)?;
    let dark_result = highlighter.highlight_line("echo 'hello world'");
    assert!(!dark_result.is_empty());
    
    highlighter.set_theme(&light_config)?;
    let light_result = highlighter.highlight_line("if [ -f file.txt ]; then");
    assert!(!light_result.is_empty());
    
    // Test theme retrieval
    let theme = NexusTheme::default();
    let retrieved_config = theme.get_theme_config("Dark")?;
    assert_eq!(retrieved_config.name, "Base16 Ocean Dark");
    
    let monokai_retrieved = theme.get_theme_config("Monokai")?;
    assert_eq!(monokai_retrieved.name, "Base16 Mocha Dark");
    
    println!("  ✅ Syntax highlighting integration working correctly");
    
    Ok(())
}

/// Test theme persistence
fn test_theme_persistence() -> Result<()> {
    println!("\n🎯 Test 4: Theme Persistence");
    
    let temp_dir = TempDir::new()?;
    
    // Create and save a theme
    let theme = NexusTheme::default();
    let json_path = temp_dir.path().join("test_theme.json");
    let yaml_path = temp_dir.path().join("test_theme.yaml");
    
    // Test JSON format
    theme.save_to_file(&json_path, ThemeFormat::Json)?;
    assert!(json_path.exists());
    
    let loaded_json = NexusTheme::load_from_file(&json_path)?;
    assert_eq!(theme.name, loaded_json.name);
    assert_eq!(theme.description, loaded_json.description);
    
    // Test YAML format
    theme.save_to_file(&yaml_path, ThemeFormat::Yaml)?;
    assert!(yaml_path.exists());
    
    let loaded_yaml = NexusTheme::load_from_file(&yaml_path)?;
    assert_eq!(theme.name, loaded_yaml.name);
    assert_eq!(theme.description, loaded_yaml.description);
    
    // Test serialization/deserialization
    let json_str = serde_json::to_string(&theme)?;
    let deserialized: NexusTheme = serde_json::from_str(&json_str)?;
    assert_eq!(theme.name, deserialized.name);
    
    println!("  ✅ Theme persistence working correctly");
    
    Ok(())
}

/// Test pure Rust theme configuration
fn test_pure_rust_themes() -> Result<()> {
    println!("\n🎯 Test 5: Pure Rust Theme Configuration");
    
    let theme = NexusTheme::default();
    
    // Test all built-in theme configurations
    let test_themes = vec![
        ("Dark", "Base16 Ocean Dark"),
        ("Light", "Base16 Ocean Light"),
        ("base16-ocean.dark", "Base16 Ocean Dark"),
        ("base16-ocean.light", "Base16 Ocean Light"),
        ("base16-eighties.dark", "Base16 Eighties Dark"),
        ("base16-mocha.dark", "Base16 Mocha Dark"),
        ("Monokai", "Base16 Mocha Dark"),
        ("InspiredGitHub", "Inspired GitHub"),
        ("Solarized (dark)", "Solarized Dark"),
        ("Solarized Dark", "Solarized Dark"),
        ("Solarized (light)", "Solarized Light"),
    ];
    
    for (theme_name, expected_config_name) in test_themes {
        let config = theme.get_theme_config(theme_name)?;
        assert_eq!(config.name, expected_config_name);
        
        // Verify colors are different from default
        assert!(config.background.r != 0 || config.background.g != 0 || config.background.b != 0);
        assert!(config.foreground.r != 0 || config.foreground.g != 0 || config.foreground.b != 0);
        
        println!("    ✓ Theme '{}' -> Config '{}'", theme_name, config.name);
    }
    
    // Test error case
    assert!(theme.get_theme_config("NonExistentTheme").is_err());
    
    // Test available theme names
    let available = NexusTheme::available_theme_names();
    assert!(!available.is_empty());
    assert!(available.len() >= 6);
    
    println!("  ✅ Pure Rust theme configuration working correctly");
    
    Ok(())
}

/// Test color system
fn test_color_system() -> Result<()> {
    println!("\n🎯 Test 6: Color System");
    
    // Test RGB color creation
    let red = RgbColor { r: 255, g: 0, b: 0 };
    let _green = RgbColor { r: 0, g: 255, b: 0 };
    let _blue = RgbColor { r: 0, g: 0, b: 255 };
    
    // Test color conversion  
    let ansi_red = ansi_term::Colour::RGB(red.r, red.g, red.b);
    let converted_back: RgbColor = ansi_red.into();
    assert_eq!(red.r, converted_back.r);
    assert_eq!(red.g, converted_back.g);
    assert_eq!(red.b, converted_back.b);
    
    // Test UI color schemes
    let dark_scheme = UiColors::dark();
    let light_scheme = UiColors::light();
    let monokai_scheme = UiColors::monokai();
    let solarized_scheme = UiColors::solarized_dark();
    
    // Verify different schemes have different colors
    assert_ne!(dark_scheme.background.r, light_scheme.background.r);
    assert_ne!(monokai_scheme.primary.g, solarized_scheme.primary.g);
    
    // Test syntax element colors
    let theme_config = ThemeConfig::base16_ocean_dark();
    let all_elements = vec![
        SyntaxElement::Background,
        SyntaxElement::Foreground,
        SyntaxElement::Keyword,
        SyntaxElement::String,
        SyntaxElement::Number,
        SyntaxElement::Comment,
        SyntaxElement::Function,
        SyntaxElement::Variable,
        SyntaxElement::Operator,
        SyntaxElement::Punctuation,
        SyntaxElement::Type,
        SyntaxElement::Constant,
        SyntaxElement::Preprocessor,
        SyntaxElement::Escape,
        SyntaxElement::Error,
        SyntaxElement::Warning,
        SyntaxElement::Selection,
        SyntaxElement::LineHighlight,
    ];
    
    for element in all_elements {
        let color = theme_config.get_syntax_color(element);
        // Verify color is not black (0,0,0) unless intended
        assert!(color.r > 0 || color.g > 0 || color.b > 0 || 
               element == SyntaxElement::Background);
    }
    
    println!("  ✅ Color system working correctly");
    
    Ok(())
}

/// Test custom theme creation
fn test_custom_theme_creation() -> Result<()> {
    println!("\n🎯 Test 7: Custom Theme Creation");
    
    tokio::runtime::Runtime::new()?.block_on(async {
        let manager = ThemeManager::new()?;
        
        // Test creating custom theme based on existing one
        let custom_theme = manager.create_custom_theme(
            "MyCustomTheme".to_string(),
            "Dark"
        ).await?;
        
        assert_eq!(custom_theme.name, "MyCustomTheme");
        assert!(custom_theme.description.contains("Custom theme based on Dark"));
        assert_eq!(custom_theme.author, "User");
        
        // Test with different base themes
        let custom_light = manager.create_custom_theme(
            "MyLightTheme".to_string(),
            "Light"
        ).await?;
        
        assert_eq!(custom_light.name, "MyLightTheme");
        assert!(custom_light.description.contains("Custom theme based on Light"));
        
        // Test error case
        let result = manager.create_custom_theme(
            "ErrorTheme".to_string(),
            "NonExistentBase"
        ).await;
        assert!(result.is_err());
        
        println!("  ✅ Custom theme creation working correctly");
        
        Ok::<(), anyhow::Error>(())
    })?;
    
    Ok(())
}

// Test main function for cargo run
fn main() -> Result<()> {
    test_theme_system()
}
