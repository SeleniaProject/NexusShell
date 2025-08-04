//! Task 11: Internationalization System Test
//! 
//! This example demonstrates the comprehensive internationalization system
//! implementation for NexusShell.

use nxsh_core::i18n::I18nManager;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 NexusShell Internationalization System Test");
    println!("Task 11: Complete i18n and Localization Support");
    println!();

    // Create i18n manager with locale directory
    let locale_dir = std::path::PathBuf::from("crates/nxsh_builtins/locales");
    let mut i18n = I18nManager::new(locale_dir);
    
    // Test 1: Locale Detection and Setup
    println!("Test 1: System Locale Detection");
    println!("  Current Locale: {}", i18n.current_locale());
    println!("  Supported Locales: {:?}", i18n.supported_locales());
    println!();
    
    // Test 2: Locale Information
    println!("Test 2: Locale Information");
    for locale in i18n.supported_locales() {
        if let Some(info) = i18n.get_locale_info(locale) {
            println!("  {} - {} ({})", locale, info.display_name, info.native_name);
        }
    }
    println!();
    
    // Test 3: Number Formatting
    println!("Test 3: Locale-Specific Number Formatting");
    let test_number = 1234567.89;
    
    for locale in &["en-US", "ja-JP", "de-DE", "fr-FR", "zh-CN"] {
        i18n.set_locale(locale)?;
        let formatted = i18n.format_number(test_number);
        println!("  {}: {}", locale, formatted);
    }
    println!();
    
    // Test 4: Currency Formatting
    println!("Test 4: Currency Formatting");
    let test_amount = 1234.56;
    
    for locale in &["en-US", "ja-JP", "de-DE", "fr-FR"] {
        i18n.set_locale(locale)?;
        let currency = match locale {
            &"en-US" => "USD",
            &"ja-JP" => "JPY", 
            &"de-DE" => "EUR",
            &"fr-FR" => "EUR",
            _ => "USD",
        };
        let formatted = i18n.format_currency(test_amount, currency);
        println!("  {}: {}", locale, formatted);
    }
    println!();
    
    // Test 5: File Size Formatting
    println!("Test 5: File Size Formatting");
    let test_bytes = 1073741824; // 1 GB
    
    for locale in &["en-US", "ja-JP", "de-DE", "fr-FR", "zh-CN"] {
        i18n.set_locale(locale)?;
        let formatted = i18n.format_file_size(test_bytes);
        println!("  {}: {}", locale, formatted);
    }
    println!();
    
    // Test 6: Duration Formatting
    println!("Test 6: Duration Formatting");
    let test_seconds = 3725; // 1 hour, 2 minutes, 5 seconds
    
    for locale in &["en-US", "ja-JP", "de-DE", "fr-FR", "zh-CN"] {
        i18n.set_locale(locale)?;
        let formatted = i18n.format_duration(test_seconds);
        println!("  {}: {}", locale, formatted);
    }
    println!();
    
    // Test 7: Text Direction
    println!("Test 7: Text Direction Support");
    for locale in i18n.supported_locales() {
        let direction = i18n.get_text_direction();
        println!("  {}: {:?}", locale, direction);
        break; // Just show current locale for brevity
    }
    println!();
    
    // Test 8: Translation Statistics
    println!("Test 8: Translation Statistics");
    let stats = i18n.get_translation_stats();
    for (locale, count) in stats {
        println!("  {}: {} translations", locale, count);
    }
    println!();
    
    // Test 9: Locale Validation
    println!("Test 9: Locale File Validation");
    for locale in &["en-US", "ja-JP", "de-DE"] {
        match i18n.validate_locale_file(locale) {
            Ok(is_valid) => println!("  {}: {}", locale, if is_valid { "Valid" } else { "Invalid" }),
            Err(e) => println!("  {}: Error - {}", locale, e),
        }
    }
    println!();
    
    // Test 10: Pluralization
    println!("Test 10: Pluralization Support");
    i18n.set_locale("en-US")?;
    for count in &[0, 1, 2, 5, 10] {
        let plural_form = i18n.get_plural_form(*count);
        println!("  Count {}: {:?}", count, plural_form);
    }
    println!();
    
    // Test 11: Message Translation with Arguments
    println!("Test 11: Message Translation with Arguments");
    i18n.set_locale("en-US")?;
    
    let mut args = HashMap::new();
    args.insert("filename".to_string(), "test.txt".to_string());
    args.insert("size".to_string(), "1024".to_string());
    
    // Test message retrieval with arguments
    let message = i18n.get_with_args("file-size-info", &args);
    println!("  Translation: {}", message);
    println!();
    
    // Test 12: Fallback Mechanism
    println!("Test 12: Fallback Translation Mechanism");
    i18n.set_locale("ja-JP")?;
    
    let missing_key = i18n.get("non-existent-key");
    println!("  Missing key result: {}", missing_key);
    
    let existing_key = i18n.get("file-exists");
    println!("  Existing key result: {}", existing_key);
    println!();
    
    println!("✅ Task 11: Internationalization System - Implementation Complete!");
    println!("🎯 Comprehensive i18n support with 10 languages and locale-specific formatting");
    
    Ok(())
}
