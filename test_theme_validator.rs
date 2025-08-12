// Theme validation test program
use std::env;
use std::path::Path;

#[path = "crates/nxsh_ui/src/theme_validator.rs"]
mod theme_validator;

use theme_validator::{ThemeValidator, ValidationResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NexusShell Theme Validator Test");
    println!("=================================");
    
    // Test 1: List available themes
    match ThemeValidator::list_available_themes() {
        Ok(themes) => {
            println!("Found {} themes:", themes.len());
            for (i, theme) in themes.iter().enumerate() {
                println!("  {}. {}", i + 1, theme);
            }
            println!();
        },
        Err(e) => println!("Error listing themes: {}", e),
    }
    
    // Test 2: Validate a few themes
    let validator = ThemeValidator::new()?;
    let test_themes = [
        "assets/themes/nxsh-dark-default.json",
        "assets/themes/nxsh-dracula.json",
        "assets/themes/nxsh-matrix.json",
    ];
    
    for theme_path in &test_themes {
        println!("Validating theme: {}", theme_path);
        match validator.validate_theme_file(theme_path) {
            Ok(result) => {
                if result.is_valid() {
                    println!("  ✅ Theme is valid!");
                    if result.has_warnings() {
                        println!("  ⚠️  Warnings:");
                        for warning in &result.warnings {
                            println!("     - {}", warning);
                        }
                    }
                } else {
                    println!("  ❌ Theme validation failed:");
                    for error in &result.errors {
                        println!("     - {}", error);
                    }
                }
            },
            Err(e) => println!("  ❌ Error validating theme: {}", e),
        }
        println!();
    }
    
    println!("Theme validation test completed!");
    Ok(())
}
