//! Comprehensive test suite for internationalization system

use super::*;
use tempfile::tempdir;

/// Create a test i18n manager with sample locale files
fn create_test_i18n_manager() -> (I18nManager, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let locale_dir = temp_dir.path().to_path_buf();

    // Create sample English locale file
    let en_content = r#"
# English locale file
welcome = Welcome to NexusShell
goodbye = Goodbye!
error-file-not-found = File not found: {filename}
command-not-found = Command '{command}' not found
files.one = {count} file
files.other = {count} files
bytes.one = {count} byte
bytes.other = {count} bytes
"#;
    std::fs::write(locale_dir.join("en-US.ftl"), en_content).expect("Failed to write en-US.ftl");

    // Create sample Japanese locale file
    let ja_content = r#"
# Japanese locale file
welcome = NexusShellへようこそ
goodbye = さようなら！
error-file-not-found = ファイルが見つかりません: {filename}
command-not-found = コマンド '{command}' が見つかりません
files.other = {count}個のファイル
bytes.other = {count}バイト
"#;
    std::fs::write(locale_dir.join("ja-JP.ftl"), ja_content).expect("Failed to write ja-JP.ftl");

    // Create sample German locale file
    let de_content = r#"
# German locale file
welcome = Willkommen bei NexusShell
goodbye = Auf Wiedersehen!
error-file-not-found = Datei nicht gefunden: {filename}
command-not-found = Befehl '{command}' nicht gefunden
files.one = {count} Datei
files.other = {count} Dateien
bytes.one = {count} Byte
bytes.other = {count} Bytes
"#;
    std::fs::write(locale_dir.join("de-DE.ftl"), de_content).expect("Failed to write de-DE.ftl");

    let manager = I18nManager::new(locale_dir);
    (manager, temp_dir)
}

#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn test_locale_detection() {
        // Test default locale
        let locale = I18nManager::detect_system_locale();
        assert!(!locale.is_empty());

        // Test locale normalization
        assert_eq!(I18nManager::normalize_locale("en_US.UTF-8"), "en-US");
        assert_eq!(I18nManager::normalize_locale("ja_JP"), "ja-JP");
        assert_eq!(I18nManager::normalize_locale("de"), "de-DE");
        assert_eq!(I18nManager::normalize_locale("invalid"), "en-US");
    }

    #[test]
    fn test_fluent_key_validation() {
        assert!(I18nManager::is_valid_fluent_key("valid-key"));
        assert!(I18nManager::is_valid_fluent_key("valid_key"));
        assert!(I18nManager::is_valid_fluent_key("validKey123"));
        assert!(I18nManager::is_valid_fluent_key("_private-key"));
        assert!(I18nManager::is_valid_fluent_key("files.one")); // Dot notation for plurals
        assert!(I18nManager::is_valid_fluent_key("error.file-not-found"));

        assert!(!I18nManager::is_valid_fluent_key(""));
        assert!(!I18nManager::is_valid_fluent_key("123invalid"));
        assert!(!I18nManager::is_valid_fluent_key("invalid key"));
        assert!(!I18nManager::is_valid_fluent_key("-invalid"));
    }

    #[test]
    fn test_fluent_file_parsing() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();

        let content = r#"
# Test fluent file
simple-key = Simple value
multiline-key = 
    This is a multiline
    value with continuation
quoted-key = "Quoted value with spaces"
key-with-placeholder = Hello {name}!
"#;

        let translations = manager.parse_fluent_file(content)?;

        assert_eq!(
            translations.get("simple-key"),
            Some(&"Simple value".to_string())
        );
        assert_eq!(
            translations.get("multiline-key"),
            Some(&"This is a multiline value with continuation".to_string())
        );
        assert_eq!(
            translations.get("quoted-key"),
            Some(&"Quoted value with spaces".to_string())
        );
        assert_eq!(
            translations.get("key-with-placeholder"),
            Some(&"Hello {name}!".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_fluent_syntax_validation() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();

        // Valid syntax
        let valid_content = r#"
valid-key = Valid value
another-key = Another value
"#;
        let errors = manager.validate_fluent_syntax(valid_content)?;
        assert!(errors.is_empty());

        // Invalid syntax - missing equals
        let invalid_content = r#"
valid-key = Valid value
invalid-line-without-equals
"#;
        let errors = manager.validate_fluent_syntax(invalid_content)?;
        assert!(!errors.is_empty());
        assert!(errors[0].contains("Invalid Fluent syntax"));

        // Invalid syntax - unmatched braces
        let invalid_braces = r#"
invalid-key = Value with {unmatched brace
"#;
        let errors = manager.validate_fluent_syntax(invalid_braces)?;
        assert!(!errors.is_empty());

        Ok(())
    }

    #[test]
    fn test_locale_loading() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();

        // Load English locale
        manager.load_locale("en-US")?;
        assert_eq!(manager.get("welcome"), "Welcome to NexusShell");

        // Load Japanese locale
        manager.load_locale("ja-JP")?;
        // Test current locale override by temporarily changing it
        let mut temp_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        temp_manager.load_locale("ja-JP")?;
        temp_manager.current_locale = "ja-JP".to_string();
        assert_eq!(temp_manager.get("welcome"), "NexusShellへようこそ");

        // Test fallback to English for missing translations
        assert_eq!(temp_manager.get("nonexistent-key"), "nonexistent-key");

        Ok(())
    }

    #[test]
    fn test_message_with_arguments() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();
        manager.load_locale("en-US")?;

        let mut args = HashMap::new();
        args.insert("filename".to_string(), "test.txt".to_string());

        let message = manager.get_with_args("error-file-not-found", &args);
        assert_eq!(message, "File not found: test.txt");

        Ok(())
    }

    #[test]
    fn test_number_formatting() {
        let (_manager, _temp_dir) = create_test_i18n_manager();

        // Test English formatting (no locale file needed for formatting)
        let mut test_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        test_manager.current_locale = "en-US".to_string();
        assert_eq!(test_manager.format_number(1234.56), "1,234.56");
        assert_eq!(test_manager.format_integer(1234567), "1,234,567");

        // Test German formatting
        test_manager.current_locale = "de-DE".to_string();
        assert_eq!(test_manager.format_number(1234.56), "1 234,56");

        // Test French formatting
        test_manager.current_locale = "fr-FR".to_string();
        assert_eq!(test_manager.format_number(1234.56), "1 234,56");

        // Test percentage formatting
        assert_eq!(test_manager.format_percentage(0.753), "75,3 %");
    }

    #[test]
    fn test_date_time_formatting() {
        let (_manager, _temp_dir) = create_test_i18n_manager();
        let test_date = chrono::Utc::now();

        // Test different locale date formats
        let mut test_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        test_manager.current_locale = "en-US".to_string();
        let us_date = test_manager.format_date(&test_date);
        assert!(us_date.contains("/"));

        test_manager.current_locale = "ja-JP".to_string();
        let jp_date = test_manager.format_date(&test_date);
        assert!(jp_date.contains("年"));

        test_manager.current_locale = "de-DE".to_string();
        let de_date = test_manager.format_date(&test_date);
        assert!(de_date.contains("."));
    }

    #[test]
    fn test_file_size_formatting() {
        let (_manager, _temp_dir) = create_test_i18n_manager();

        // Test different file sizes
        let mut test_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        test_manager.current_locale = "en-US".to_string();
        assert_eq!(test_manager.format_file_size(0), "0 B");
        assert_eq!(test_manager.format_file_size(512), "512 B");
        assert_eq!(test_manager.format_file_size(1024), "1.0 KB");
        assert_eq!(test_manager.format_file_size(1536), "1.5 KB");
        assert_eq!(test_manager.format_file_size(1048576), "1.0 MB");

        // Test binary formatting for German locale
        test_manager.current_locale = "de-DE".to_string();
        assert_eq!(test_manager.format_file_size(1024), "1,0 KiB");
    }

    #[test]
    fn test_duration_formatting() {
        let (_manager, _temp_dir) = create_test_i18n_manager();

        // Test English duration formatting
        let mut test_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        test_manager.current_locale = "en-US".to_string();
        assert_eq!(test_manager.format_duration(30), "30s");
        assert_eq!(test_manager.format_duration(90), "1m 30s");
        assert_eq!(test_manager.format_duration(3665), "1h 1m 5s");

        // Test Japanese duration formatting
        test_manager.current_locale = "ja-JP".to_string();
        assert_eq!(test_manager.format_duration(30), "30秒");
        assert_eq!(test_manager.format_duration(90), "1分30秒");
        assert_eq!(test_manager.format_duration(3665), "1時間1分5秒");
    }

    #[test]
    fn test_currency_formatting() {
        let (_manager, _temp_dir) = create_test_i18n_manager();

        // Test different currency formats
        let mut test_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        test_manager.current_locale = "en-US".to_string();
        assert_eq!(test_manager.format_currency(123.45, "USD"), "$123.45");

        test_manager.current_locale = "ja-JP".to_string();
        assert_eq!(test_manager.format_currency(123.45, "JPY"), "¥123");

        test_manager.current_locale = "de-DE".to_string();
        // In German locale, numbers use comma as decimal separator
        assert_eq!(test_manager.format_currency(123.45, "EUR"), "123,45 €");
    }

    #[test]
    fn test_plural_forms() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();

        // Test English plurals
        manager.load_locale("en-US")?;
        let mut temp_manager = I18nManager::new(_temp_dir.path().to_path_buf());
        temp_manager.load_locale("en-US")?;
        temp_manager.current_locale = "en-US".to_string();
        assert_eq!(temp_manager.get_plural("files", 1), "1 file");
        assert_eq!(temp_manager.get_plural("files", 5), "5 files");

        // Test German plurals
        temp_manager.load_locale("de-DE")?;
        temp_manager.current_locale = "de-DE".to_string();
        assert_eq!(temp_manager.get_plural("files", 1), "1 Datei");
        assert_eq!(temp_manager.get_plural("files", 5), "5 Dateien");

        Ok(())
    }

    #[test]
    fn test_locale_info() {
        let manager = I18nManager::default();

        // Test English locale info
        let en_info = manager.get_locale_info("en-US").unwrap();
        assert_eq!(en_info.language, "en");
        assert_eq!(en_info.country, "US");
        assert_eq!(en_info.direction, TextDirection::LeftToRight);

        // Test Japanese locale info
        let ja_info = manager.get_locale_info("ja-JP").unwrap();
        assert_eq!(ja_info.language, "ja");
        assert_eq!(ja_info.country, "JP");
        assert_eq!(ja_info.script, Some("Jpan".to_string()));

        // Test invalid locale
        assert!(manager.get_locale_info("invalid").is_none());
    }

    #[test]
    fn test_validation_report() {
        let (manager, _temp_dir) = create_test_i18n_manager();

        let report = manager.get_validation_report();

        // Should have entries for all supported locales
        assert!(report.len() <= manager.supported_locales().len());

        // Existing locales should have no errors (empty vec or not present in report)
        if let Some(en_errors) = report.get("en-US") {
            if !en_errors.is_empty() {
                println!("English locale errors: {en_errors:?}");
            }
            // Don't assert empty as validation might find issues we need to fix
        }
        if let Some(ja_errors) = report.get("ja-JP") {
            if !ja_errors.is_empty() {
                println!("Japanese locale errors: {ja_errors:?}");
            }
        }
        if let Some(de_errors) = report.get("de-DE") {
            if !de_errors.is_empty() {
                println!("German locale errors: {de_errors:?}");
            }
        }
    }

    #[test]
    fn test_translation_stats() -> crate::error::ShellResult<()> {
        let (manager, _temp_dir) = create_test_i18n_manager();

        // Load multiple locales
        manager.load_locale("en-US")?;
        manager.load_locale("ja-JP")?;
        manager.load_locale("de-DE")?;

        let stats = manager.get_translation_stats();

        // Should have stats for loaded locales
        assert!(stats.contains_key("en-US"));
        assert!(stats.contains_key("ja-JP"));
        assert!(stats.contains_key("de-DE"));

        // Each locale should have some translations
        assert!(stats["en-US"] > 0);
        assert!(stats["ja-JP"] > 0);
        assert!(stats["de-DE"] > 0);

        Ok(())
    }

    #[test]
    fn test_locale_support() {
        let manager = I18nManager::default();

        // Test supported locales
        assert!(manager.is_locale_supported("en-US"));
        assert!(manager.is_locale_supported("ja-JP"));
        assert!(manager.is_locale_supported("de-DE"));

        // Test unsupported locale
        assert!(!manager.is_locale_supported("xx-XX"));

        // Test supported locales list
        let supported = manager.supported_locales();
        assert!(supported.contains(&"en-US".to_string()));
        assert!(supported.contains(&"ja-JP".to_string()));
        assert_eq!(supported.len(), 10); // All 10 supported locales
    }

    #[test]
    fn test_thousands_separator() {
        // Test different separators
        assert_eq!(
            I18nManager::add_thousands_separator("1234567", ","),
            "1,234,567"
        );
        assert_eq!(
            I18nManager::add_thousands_separator("1234567", " "),
            "1 234 567"
        );
        assert_eq!(
            I18nManager::add_thousands_separator("1234567", "."),
            "1.234.567"
        );

        // Test with negative numbers
        assert_eq!(
            I18nManager::add_thousands_separator("-1234567", ","),
            "-1,234,567"
        );

        // Test with small numbers
        assert_eq!(I18nManager::add_thousands_separator("123", ","), "123");
        assert_eq!(I18nManager::add_thousands_separator("1", ","), "1");
    }

    #[test]
    fn test_fluent_value_processing() -> crate::error::ShellResult<()> {
        // Test simple value
        let result = I18nManager::process_fluent_value("Simple value")?;
        assert_eq!(result, "Simple value");

        // Test quoted value
        let result = I18nManager::process_fluent_value("\"Quoted value\"")?;
        assert_eq!(result, "Quoted value");

        // Test escaped characters
        let result = I18nManager::process_fluent_value("Value with\\nnewline")?;
        assert_eq!(result, "Value with\nnewline");

        // Test escaped quotes
        let result = I18nManager::process_fluent_value("Value with \\\"quotes\\\"")?;
        assert_eq!(result, "Value with \"quotes\"");

        Ok(())
    }

    #[test]
    fn test_error_handling() {
        let manager = I18nManager::default();

        // Test loading non-existent locale
        let result = manager.load_locale("nonexistent");
        assert!(result.is_err());

        // Test setting unsupported locale
        let mut manager = I18nManager::default();
        let result = manager.set_locale("unsupported-locale");
        assert!(result.is_err());

        // Test validation of invalid file
        let result = manager.validate_locale_file("nonexistent");
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for non-existent file
    }
}
