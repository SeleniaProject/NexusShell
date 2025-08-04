//! Internationalization and localization support for NexusShell
//!
//! This module provides comprehensive i18n support including message translation,
//! locale detection, and culture-specific formatting.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use chrono::{DateTime, Utc};
use tracing::{debug, error};

#[cfg(test)]
mod tests;

/// Internationalization manager for NexusShell
pub struct I18nManager {
    /// Current locale
    current_locale: String,
    /// Loaded translations
    translations: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    /// Fallback locale
    fallback_locale: String,
    /// Locale directory
    locale_dir: std::path::PathBuf,
    /// Supported locales
    supported_locales: Vec<String>,
}

impl I18nManager {
    /// Create a new i18n manager
    pub fn new(locale_dir: std::path::PathBuf) -> Self {
        Self {
            current_locale: Self::detect_system_locale(),
            translations: Arc::new(RwLock::new(HashMap::new())),
            fallback_locale: "en-US".to_string(),
            locale_dir,
            supported_locales: vec![
                "en-US".to_string(),
                "ja-JP".to_string(),
                "de-DE".to_string(),
                "fr-FR".to_string(),
                "es-ES".to_string(),
                "it-IT".to_string(),
                "pt-BR".to_string(),
                "ru-RU".to_string(),
                "zh-CN".to_string(),
                "ko-KR".to_string(),
            ],
        }
    }

    /// Detect system locale
    fn detect_system_locale() -> String {
        // Try environment variables
        if let Ok(locale) = std::env::var("LC_ALL") {
            return Self::normalize_locale(&locale);
        }
        if let Ok(locale) = std::env::var("LC_MESSAGES") {
            return Self::normalize_locale(&locale);
        }
        if let Ok(locale) = std::env::var("LANG") {
            return Self::normalize_locale(&locale);
        }

        // Default to English
        "en-US".to_string()
    }

    /// Normalize locale string
    fn normalize_locale(locale: &str) -> String {
        // Extract language and country from locale string
        let parts: Vec<&str> = locale.split(['_', '.', '@']).collect();
        if parts.len() >= 2 {
            format!("{}-{}", parts[0].to_lowercase(), parts[1].to_uppercase())
        } else {
            match parts[0].to_lowercase().as_str() {
                "en" => "en-US".to_string(),
                "ja" => "ja-JP".to_string(),
                "de" => "de-DE".to_string(),
                "fr" => "fr-FR".to_string(),
                "es" => "es-ES".to_string(),
                "it" => "it-IT".to_string(),
                "pt" => "pt-BR".to_string(),
                "ru" => "ru-RU".to_string(),
                "zh" => "zh-CN".to_string(),
                "ko" => "ko-KR".to_string(),
                _ => "en-US".to_string(),
            }
        }
    }

    /// Load translations for a locale
    pub fn load_locale(&self, locale: &str) -> crate::error::ShellResult<()> {
        let locale_file = self.locale_dir.join(format!("{}.ftl", locale));
        
        if !locale_file.exists() {
            return Err(crate::error::ShellError::new(
                crate::error::ErrorKind::IoError(crate::error::IoErrorKind::NotFound),
                format!("Locale file not found: {:?}", locale_file)
            ));
        }

        let content = std::fs::read_to_string(&locale_file)
            .map_err(|e| crate::error::ShellError::new(
                crate::error::ErrorKind::IoError(crate::error::IoErrorKind::FileReadError),
                format!("Failed to read locale file: {}", e)
            ))?;

        let translations = self.parse_fluent_file(&content)?;
        
        if let Ok(mut trans) = self.translations.write() {
            trans.insert(locale.to_string(), translations);
        }

        debug!("Loaded translations for locale: {}", locale);
        Ok(())
    }

    /// Parse Fluent (.ftl) file format
    fn parse_fluent_file(&self, content: &str) -> crate::error::ShellResult<HashMap<String, String>> {
        let mut translations = HashMap::new();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_multiline = false;
        
        for (line_num, original_line) in content.lines().enumerate() {
            let line = original_line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Handle multiline values
            if in_multiline {
                if original_line.starts_with(' ') || original_line.starts_with('\t') {
                    // Continuation of multiline value
                    if !current_value.is_empty() {
                        current_value.push(' '); // Use space instead of newline for proper formatting
                    }
                    current_value.push_str(line);
                    continue;
                } else {
                    // End of multiline value
                    if !current_key.is_empty() && !current_value.is_empty() {
                        translations.insert(current_key.clone(), current_value.clone());
                    }
                    current_key.clear();
                    current_value.clear();
                    in_multiline = false;
                    // Fall through to process this line as a potential new key-value pair
                }
            }
            
            // Parse key-value pairs
            if let Some(eq_pos) = line.find('=') {
                current_key = line[..eq_pos].trim().to_string();
                let value_part = line[eq_pos + 1..].trim();
                
                // Validate key format
                if !Self::is_valid_fluent_key(&current_key) {
                    return Err(crate::error::ShellError::new(
                        crate::error::ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                        format!("Invalid Fluent key '{}' at line {}", current_key, line_num + 1)
                    ));
                }
                
                if value_part.is_empty() {
                    // Start of multiline value
                    in_multiline = true;
                    current_value.clear();
                } else {
                    // Single line value
                    current_value = Self::process_fluent_value(value_part)?;
                    translations.insert(current_key.clone(), current_value.clone());
                    current_key.clear();
                    current_value.clear();
                }
            } else if !original_line.starts_with(' ') && !original_line.starts_with('\t') && !in_multiline {
                // Invalid syntax - line without '=' that's not indented and not in multiline
                return Err(crate::error::ShellError::new(
                    crate::error::ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                    format!("Invalid Fluent syntax at line {}: {}", line_num + 1, line)
                ));
            }
        }
        
        // Handle final multiline value
        if in_multiline && !current_key.is_empty() && !current_value.is_empty() {
            translations.insert(current_key, current_value);
        }
        
        debug!("Parsed {} Fluent translations", translations.len());
        Ok(translations)
    }
    
    /// Validate Fluent key format
    fn is_valid_fluent_key(key: &str) -> bool {
        if key.is_empty() {
            return false;
        }
        
        // Fluent keys must start with letter or underscore
        let first_char = key.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
        
        // Fluent keys can contain letters, numbers, hyphens, underscores, and dots
        key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    }
    
    /// Process Fluent value with escape sequences and placeholders
    fn process_fluent_value(value: &str) -> crate::error::ShellResult<String> {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    // Handle escape sequences
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            '\\' => result.push('\\'),
                            '"' => result.push('"'),
                            '\'' => result.push('\''),
                            '{' => result.push('{'),
                            '}' => result.push('}'),
                            _ => {
                                result.push('\\');
                                result.push(next_ch);
                            }
                        }
                    } else {
                        result.push('\\');
                    }
                }
                '"' => {
                    // Handle quoted strings
                    while let Some(quoted_ch) = chars.next() {
                        if quoted_ch == '"' {
                            break;
                        } else if quoted_ch == '\\' {
                            if let Some(escaped_ch) = chars.next() {
                                match escaped_ch {
                                    'n' => result.push('\n'),
                                    'r' => result.push('\r'),
                                    't' => result.push('\t'),
                                    '\\' => result.push('\\'),
                                    '"' => result.push('"'),
                                    _ => {
                                        result.push('\\');
                                        result.push(escaped_ch);
                                    }
                                }
                            }
                        } else {
                            result.push(quoted_ch);
                        }
                    }
                }
                _ => result.push(ch),
            }
        }
        
        Ok(result.trim().to_string())
    }

    /// Set current locale
    pub fn set_locale(&mut self, locale: &str) -> crate::error::ShellResult<()> {
        if !self.supported_locales.contains(&locale.to_string()) {
            return Err(crate::error::ShellError::new(
                crate::error::ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("Unsupported locale: {}", locale)
            ));
        }

        // Load locale if not already loaded
        if let Ok(trans) = self.translations.read() {
            if !trans.contains_key(locale) {
                drop(trans);
                self.load_locale(locale)?;
            }
        }

        self.current_locale = locale.to_string();
        debug!("Set current locale to: {}", locale);
        Ok(())
    }

    /// Get current locale
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Get translated message
    pub fn get(&self, key: &str) -> String {
        self.get_with_locale(key, &self.current_locale)
    }

    /// Get translated message with specific locale
    pub fn get_with_locale(&self, key: &str, locale: &str) -> String {
        if let Ok(trans) = self.translations.read() {
            if let Some(locale_trans) = trans.get(locale) {
                if let Some(message) = locale_trans.get(key) {
                    return message.clone();
                }
            }

            // Try fallback locale
            if locale != self.fallback_locale {
                if let Some(fallback_trans) = trans.get(&self.fallback_locale) {
                    if let Some(message) = fallback_trans.get(key) {
                        return message.clone();
                    }
                }
            }
        }

        // Return key if no translation found
        key.to_string()
    }

    /// Get translated message with arguments
    pub fn get_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        let mut message = self.get(key);
        
        // Advanced placeholder replacement with support for:
        // - Positional arguments: {0}, {1}, etc.
        // - Named arguments: {name}, {value}, etc.
        // - Pluralization: {count} with |one|other syntax
        // - Date/time formatting: {date:format}
        
        for (arg_key, arg_value) in args {
            // Handle different placeholder formats
            let placeholders = vec![
                format!("{{{}}}", arg_key),           // {name}
                format!("{{{}:.*}}", arg_key),        // {name:format}
                format!("{{{}|.*|.*}}", arg_key),     // {name|singular|plural}
            ];
            
            for placeholder_pattern in placeholders {
                if placeholder_pattern.contains(".*") {
                    // Handle format specifiers and pluralization
                    message = self.replace_advanced_placeholder(&message, arg_key, arg_value);
                } else {
                    // Simple replacement
                    message = message.replace(&placeholder_pattern, arg_value);
                }
            }
        }
        
        message
    }

    /// Handle advanced placeholder replacement with formatting and pluralization
    fn replace_advanced_placeholder(&self, message: &str, key: &str, value: &str) -> String {
        let mut result = message.to_string();
        
        // Handle format specifiers like {date:yyyy-MM-dd}
        let format_pattern = format!("{{{}:", key);
        if let Some(start) = result.find(&format_pattern) {
            if let Some(end) = result[start..].find('}') {
                let full_placeholder = &result[start..start + end + 1];
                let format_spec = &full_placeholder[format_pattern.len()..full_placeholder.len()-1];
                
                // Apply formatting based on type
                let formatted_value = self.apply_format(value, format_spec);
                result = result.replace(full_placeholder, &formatted_value);
            }
        }
        
        // Handle pluralization like {count|item|items}
        let plural_pattern = format!("{{{}|", key);
        if let Some(start) = result.find(&plural_pattern) {
            if let Some(end) = result[start..].find('}') {
                let full_placeholder = &result[start..start + end + 1];
                let plural_spec = &full_placeholder[plural_pattern.len()..full_placeholder.len()-1];
                
                if let Some(pipe_pos) = plural_spec.find('|') {
                    let singular = &plural_spec[..pipe_pos];
                    let plural = &plural_spec[pipe_pos + 1..];
                    
                    // Determine if singular or plural based on value
                    let count: i32 = value.parse().unwrap_or(0);
                    let chosen_form = if count == 1 { singular } else { plural };
                    
                    result = result.replace(full_placeholder, &format!("{} {}", value, chosen_form));
                }
            }
        }
        
        result
    }

    /// Apply formatting to a value based on format specifier
    fn apply_format(&self, value: &str, format_spec: &str) -> String {
        match format_spec {
            "upper" => value.to_uppercase(),
            "lower" => value.to_lowercase(),
            "capitalize" => {
                let mut chars = value.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            }
            "number" => {
                // Format as number with thousand separators
                if let Ok(num) = value.parse::<i64>() {
                    // Simple number formatting without thousand separators for now
                    num.to_string()
                } else {
                    value.to_string()
                }
            }
            spec if spec.starts_with("date:") => {
                // Basic date formatting (would need chrono for full implementation)
                let date_format = &spec[5..];
                match date_format {
                    "short" => value.to_string(), // Simplified - would use actual date parsing
                    "long" => value.to_string(),
                    "iso" => value.to_string(),
                    _ => value.to_string(),
                }
            }
            _ => value.to_string(),
        }
    }

    /// Format number according to locale
    pub fn format_number(&self, number: f64) -> String {
        self.format_number_with_precision(number, 2)
    }
    
    /// Format number with specified decimal precision according to locale
    pub fn format_number_with_precision(&self, number: f64, precision: usize) -> String {
        match self.current_locale.as_str() {
            // German locale: 1.234,56 (space as thousands separator, comma as decimal)
            "de-DE" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                // Add thousands separators
                let integer_with_sep = Self::add_thousands_separator(integer, " ");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{},{}", integer_with_sep, decimal)
                }
            }
            // French locale: 1 234,56 (space as thousands separator, comma as decimal)
            "fr-FR" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, " ");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{},{}", integer_with_sep, decimal)
                }
            }
            // Spanish/Italian locale: 1.234,56 (dot as thousands separator, comma as decimal)
            "es-ES" | "it-IT" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ".");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{},{}", integer_with_sep, decimal)
                }
            }
            // Portuguese (Brazil): 1.234,56 
            "pt-BR" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ".");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{},{}", integer_with_sep, decimal)
                }
            }
            // Russian locale: 1 234,56 (space as thousands separator, comma as decimal)
            "ru-RU" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, " ");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{},{}", integer_with_sep, decimal)
                }
            }
            // Japanese locale: 1,234.56 (comma as thousands separator, dot as decimal)
            "ja-JP" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ",");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{}.{}", integer_with_sep, decimal)
                }
            }
            // Chinese locale: 1,234.56 
            "zh-CN" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ",");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{}.{}", integer_with_sep, decimal)
                }
            }
            // Korean locale: 1,234.56
            "ko-KR" => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ",");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{}.{}", integer_with_sep, decimal)
                }
            }
            // Default (US/English): 1,234.56 (comma as thousands separator, dot as decimal)
            _ => {
                let formatted = format!("{:.precision$}", number, precision = precision);
                let (integer, decimal) = if formatted.contains('.') {
                    let parts: Vec<&str> = formatted.split('.').collect();
                    (parts[0], parts.get(1).map_or("", |v| v))
                } else {
                    (formatted.as_str(), "")
                };
                
                let integer_with_sep = Self::add_thousands_separator(integer, ",");
                if decimal.is_empty() || precision == 0 {
                    integer_with_sep
                } else {
                    format!("{}.{}", integer_with_sep, decimal)
                }
            }
        }
    }
    
    /// Add thousands separator to integer part
    fn add_thousands_separator(integer: &str, separator: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = integer.chars().collect();
        
        // Handle negative numbers
        let (sign, digits) = if chars.get(0) == Some(&'-') {
            ("-", &chars[1..])
        } else {
            ("", &chars[..])
        };
        
        result.push_str(sign);
        
        for (i, &ch) in digits.iter().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                result.push_str(separator);
            }
            result.push(ch);
        }
        
        result
    }
    
    /// Format integer according to locale
    pub fn format_integer(&self, number: i64) -> String {
        self.format_number_with_precision(number as f64, 0)
    }
    
    /// Format percentage according to locale
    pub fn format_percentage(&self, ratio: f64) -> String {
        let percentage = ratio * 100.0;
        match self.current_locale.as_str() {
            "de-DE" | "fr-FR" | "es-ES" | "it-IT" | "pt-BR" | "ru-RU" => {
                format!("{} %", self.format_number_with_precision(percentage, 1))
            }
            _ => {
                format!("{}%", self.format_number_with_precision(percentage, 1))
            }
        }
    }

    /// Format date according to locale
    pub fn format_date(&self, date: &DateTime<Utc>) -> String {
        match self.current_locale.as_str() {
            "en-US" => date.format("%m/%d/%Y").to_string(),
            "ja-JP" => date.format("%Y年%m月%d日").to_string(),
            "de-DE" => date.format("%d.%m.%Y").to_string(),
            "fr-FR" => date.format("%d/%m/%Y").to_string(),
            "es-ES" => date.format("%d/%m/%Y").to_string(),
            "it-IT" => date.format("%d/%m/%Y").to_string(),
            "pt-BR" => date.format("%d/%m/%Y").to_string(),
            "ru-RU" => date.format("%d.%m.%Y").to_string(),
            "zh-CN" => date.format("%Y年%m月%d日").to_string(),
            "ko-KR" => date.format("%Y년 %m월 %d일").to_string(),
            _ => date.format("%Y-%m-%d").to_string(),
        }
    }

    /// Format time according to locale
    pub fn format_time(&self, date: &DateTime<Utc>) -> String {
        match self.current_locale.as_str() {
            "en-US" => date.format("%I:%M:%S %p").to_string(),
            _ => date.format("%H:%M:%S").to_string(),
        }
    }

    /// Format currency according to locale
    pub fn format_currency(&self, amount: f64, currency: &str) -> String {
        match self.current_locale.as_str() {
            "en-US" => format!("${}", self.format_number_with_precision(amount, 2)),
            "ja-JP" => format!("¥{}", self.format_number_with_precision(amount, 0)),
            "de-DE" => format!("{} €", self.format_number_with_precision(amount, 2)),
            "fr-FR" => format!("{} €", self.format_number_with_precision(amount, 2)),
            "es-ES" => format!("{} €", self.format_number_with_precision(amount, 2)),
            "it-IT" => format!("{} €", self.format_number_with_precision(amount, 2)),
            "pt-BR" => format!("R$ {}", self.format_number_with_precision(amount, 2)),
            "ru-RU" => format!("{} ₽", self.format_number_with_precision(amount, 2)),
            "zh-CN" => format!("¥{}", self.format_number_with_precision(amount, 2)),
            "ko-KR" => format!("₩{}", self.format_number_with_precision(amount, 0)),
            _ => format!("{} {}", self.format_number_with_precision(amount, 2), currency),
        }
    }

    /// Get supported locales
    pub fn supported_locales(&self) -> &[String] {
        &self.supported_locales
    }

    /// Check if locale is supported
    pub fn is_locale_supported(&self, locale: &str) -> bool {
        self.supported_locales.contains(&locale.to_string())
    }

    /// Get locale display name
    pub fn get_locale_display_name(&self, locale: &str) -> String {
        match locale {
            "en-US" => "English (United States)".to_string(),
            "ja-JP" => "日本語 (日本)".to_string(),
            "de-DE" => "Deutsch (Deutschland)".to_string(),
            "fr-FR" => "Français (France)".to_string(),
            "es-ES" => "Español (España)".to_string(),
            "it-IT" => "Italiano (Italia)".to_string(),
            "pt-BR" => "Português (Brasil)".to_string(),
            "ru-RU" => "Русский (Россия)".to_string(),
            "zh-CN" => "中文 (中国)".to_string(),
            "ko-KR" => "한국어 (대한민국)".to_string(),
            _ => locale.to_string(),
        }
    }

    /// Load all supported locales
    pub fn load_all_locales(&self) -> crate::error::ShellResult<()> {
        for locale in &self.supported_locales {
            if let Err(e) = self.load_locale(locale) {
                error!("Failed to load locale {}: {}", locale, e);
                // Continue loading other locales
            }
        }
        Ok(())
    }

    /// Reload current locale
    pub fn reload_current_locale(&self) -> crate::error::ShellResult<()> {
        self.load_locale(&self.current_locale.clone())
    }

    /// Get translation statistics
    pub fn get_translation_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        if let Ok(trans) = self.translations.read() {
            for (locale, translations) in trans.iter() {
                stats.insert(locale.clone(), translations.len());
            }
        }
        
        stats
    }

    /// Validate locale file
    pub fn validate_locale_file(&self, locale: &str) -> crate::error::ShellResult<bool> {
        let locale_file = self.locale_dir.join(format!("{}.ftl", locale));
        
        if !locale_file.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&locale_file)
            .map_err(|e| crate::error::ShellError::new(
                crate::error::ErrorKind::IoError(crate::error::IoErrorKind::FileReadError),
                format!("Failed to read locale file: {}", e)
            ))?;

        // Validate Fluent file syntax
        self.validate_fluent_syntax(&content)?;
        Ok(true)
    }
    
    /// Validate Fluent file syntax
    pub fn validate_fluent_syntax(&self, content: &str) -> crate::error::ShellResult<Vec<String>> {
        let mut errors = Vec::new();
        let mut current_key = String::new();
        let mut in_multiline = false;
        let mut brace_depth = 0;
        let mut in_select_expression = false;
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Handle multiline values
            if in_multiline {
                if line.starts_with(' ') || line.starts_with('\t') {
                    // Validate multiline continuation
                    if let Err(error) = Self::validate_fluent_value_line(trimmed, &mut brace_depth, &mut in_select_expression) {
                        errors.push(format!("Line {}: {}", line_num + 1, error));
                    }
                    continue;
                } else {
                    // End of multiline value
                    if brace_depth != 0 {
                        errors.push(format!("Line {}: Unmatched braces in multiline value for key '{}'", line_num + 1, current_key));
                    }
                    current_key.clear();
                    in_multiline = false;
                    brace_depth = 0;
                    in_select_expression = false;
                }
            }
            
            // Parse key-value pairs
            if let Some(eq_pos) = trimmed.find('=') {
                current_key = trimmed[..eq_pos].trim().to_string();
                let value_part = trimmed[eq_pos + 1..].trim();
                
                // Validate key format
                if !Self::is_valid_fluent_key(&current_key) {
                    errors.push(format!("Line {}: Invalid Fluent key '{}'", line_num + 1, current_key));
                }
                
                // Check for duplicate keys
                if let Ok(trans) = self.translations.read() {
                    if let Some(locale_trans) = trans.get(&self.current_locale) {
                        if locale_trans.contains_key(&current_key) {
                            errors.push(format!("Line {}: Duplicate key '{}'", line_num + 1, current_key));
                        }
                    }
                }
                
                if value_part.is_empty() {
                    // Start of multiline value
                    in_multiline = true;
                    brace_depth = 0;
                    in_select_expression = false;
                } else {
                    // Single line value
                    if let Err(error) = Self::validate_fluent_value_line(value_part, &mut brace_depth, &mut in_select_expression) {
                        errors.push(format!("Line {}: {}", line_num + 1, error));
                    }
                    if brace_depth != 0 {
                        errors.push(format!("Line {}: Unmatched braces in value", line_num + 1));
                    }
                    current_key.clear();
                    brace_depth = 0;
                    in_select_expression = false;
                }
            } else if !trimmed.starts_with(' ') && !trimmed.starts_with('\t') {
                // Invalid syntax - line without '=' that's not indented
                errors.push(format!("Line {}: Invalid Fluent syntax - missing '=' or incorrect indentation", line_num + 1));
            }
        }
        
        // Check final state
        if in_multiline && brace_depth != 0 {
            errors.push(format!("End of file: Unmatched braces in multiline value for key '{}'", current_key));
        }
        
        if errors.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(errors)
        }
    }
    
    /// Validate a single Fluent value line
    fn validate_fluent_value_line(value: &str, brace_depth: &mut i32, in_select: &mut bool) -> crate::error::ShellResult<()> {
        let mut chars = value.chars().peekable();
        let mut in_string = false;
        let mut escape_next = false;
        
        while let Some(ch) = chars.next() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    *brace_depth += 1;
                    // Check for select expressions
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_alphabetic() || next_ch == '$' {
                            // Look ahead for select keyword
                            let remaining: String = chars.clone().collect();
                            if remaining.trim_start().starts_with("$") || remaining.contains(" ->") {
                                *in_select = true;
                            }
                        }
                    }
                }
                '}' if !in_string => {
                    *brace_depth -= 1;
                    if *brace_depth < 0 {
                        return Err(crate::error::ShellError::new(
                            crate::error::ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                            "Unmatched closing brace '}'".to_string()
                        ));
                    }
                    if *brace_depth == 0 {
                        *in_select = false;
                    }
                }
                '-' if !in_string && *in_select => {
                    if let Some(&'>') = chars.peek() {
                        chars.next(); // consume '>'
                        // This is a select branch separator, which is valid
                    }
                }
                _ => {}
            }
        }
        
        if in_string {
            return Err(crate::error::ShellError::new(
                crate::error::ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                "Unterminated string literal".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get validation report for all loaded locales
    pub fn get_validation_report(&self) -> HashMap<String, Vec<String>> {
        let mut report = HashMap::new();
        
        for locale in &self.supported_locales {
            let locale_file = self.locale_dir.join(format!("{}.ftl", locale));
            
            if locale_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&locale_file) {
                    match self.validate_fluent_syntax(&content) {
                        Ok(errors) => {
                            if !errors.is_empty() {
                                report.insert(locale.clone(), errors);
                            }
                        }
                        Err(e) => {
                            report.insert(locale.clone(), vec![format!("Failed to validate: {}", e)]);
                        }
                    }
                } else {
                    report.insert(locale.clone(), vec!["Failed to read locale file".to_string()]);
                }
            } else {
                report.insert(locale.clone(), vec!["Locale file not found".to_string()]);
            }
        }
        
        report
    }
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("locales"))
    }
}

/// Fluent message with metadata
#[derive(Debug, Clone)]
pub struct FluentMessage {
    pub key: String,
    pub value: String,
    pub attributes: HashMap<String, String>,
    pub comment: Option<String>,
}

/// Locale information
#[derive(Debug, Clone)]
pub struct LocaleInfo {
    pub code: String,
    pub display_name: String,
    pub native_name: String,
    pub language: String,
    pub country: String,
    pub script: Option<String>,
    pub direction: TextDirection,
}

/// Text direction for locale
#[derive(Debug, Clone, PartialEq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

impl I18nManager {
    /// Get comprehensive locale information
    pub fn get_locale_info(&self, locale: &str) -> Option<LocaleInfo> {
        match locale {
            "en-US" => Some(LocaleInfo {
                code: "en-US".to_string(),
                display_name: "English (United States)".to_string(),
                native_name: "English (United States)".to_string(),
                language: "en".to_string(),
                country: "US".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "ja-JP" => Some(LocaleInfo {
                code: "ja-JP".to_string(),
                display_name: "Japanese (Japan)".to_string(),
                native_name: "日本語 (日本)".to_string(),
                language: "ja".to_string(),
                country: "JP".to_string(),
                script: Some("Jpan".to_string()),
                direction: TextDirection::LeftToRight,
            }),
            "de-DE" => Some(LocaleInfo {
                code: "de-DE".to_string(),
                display_name: "German (Germany)".to_string(),
                native_name: "Deutsch (Deutschland)".to_string(),
                language: "de".to_string(),
                country: "DE".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "fr-FR" => Some(LocaleInfo {
                code: "fr-FR".to_string(),
                display_name: "French (France)".to_string(),
                native_name: "Français (France)".to_string(),
                language: "fr".to_string(),
                country: "FR".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "es-ES" => Some(LocaleInfo {
                code: "es-ES".to_string(),
                display_name: "Spanish (Spain)".to_string(),
                native_name: "Español (España)".to_string(),
                language: "es".to_string(),
                country: "ES".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "it-IT" => Some(LocaleInfo {
                code: "it-IT".to_string(),
                display_name: "Italian (Italy)".to_string(),
                native_name: "Italiano (Italia)".to_string(),
                language: "it".to_string(),
                country: "IT".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "pt-BR" => Some(LocaleInfo {
                code: "pt-BR".to_string(),
                display_name: "Portuguese (Brazil)".to_string(),
                native_name: "Português (Brasil)".to_string(),
                language: "pt".to_string(),
                country: "BR".to_string(),
                script: None,
                direction: TextDirection::LeftToRight,
            }),
            "ru-RU" => Some(LocaleInfo {
                code: "ru-RU".to_string(),
                display_name: "Russian (Russia)".to_string(),
                native_name: "Русский (Россия)".to_string(),
                language: "ru".to_string(),
                country: "RU".to_string(),
                script: Some("Cyrl".to_string()),
                direction: TextDirection::LeftToRight,
            }),
            "zh-CN" => Some(LocaleInfo {
                code: "zh-CN".to_string(),
                display_name: "Chinese (China)".to_string(),
                native_name: "中文 (中国)".to_string(),
                language: "zh".to_string(),
                country: "CN".to_string(),
                script: Some("Hans".to_string()),
                direction: TextDirection::LeftToRight,
            }),
            "ko-KR" => Some(LocaleInfo {
                code: "ko-KR".to_string(),
                display_name: "Korean (Korea)".to_string(),
                native_name: "한국어 (대한민국)".to_string(),
                language: "ko".to_string(),
                country: "KR".to_string(),
                script: Some("Kore".to_string()),
                direction: TextDirection::LeftToRight,
            }),
            _ => None,
        }
    }
    
    /// Get text direction for current locale
    pub fn get_text_direction(&self) -> TextDirection {
        self.get_locale_info(&self.current_locale)
            .map(|info| info.direction)
            .unwrap_or(TextDirection::LeftToRight)
    }
    
    /// Format file size according to locale
    pub fn format_file_size(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        const BINARY_UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        
        if bytes == 0 {
            return format!("0 {}", UNITS[0]);
        }
        
        let use_binary = match self.current_locale.as_str() {
            "de-DE" | "fr-FR" => true, // Some European locales prefer binary
            _ => false,
        };
        
        let (units, divisor) = if use_binary {
            (BINARY_UNITS, 1024.0)
        } else {
            (UNITS, 1000.0)
        };
        
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= divisor && unit_index < units.len() - 1 {
            size /= divisor;
            unit_index += 1;
        }
        
        let formatted_size = if unit_index == 0 {
            format!("{}", size as u64)
        } else {
            self.format_number_with_precision(size, 1)
        };
        
        format!("{} {}", formatted_size, units[unit_index])
    }
    
    /// Format duration according to locale
    pub fn format_duration(&self, seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        match self.current_locale.as_str() {
            "ja-JP" => {
                if hours > 0 {
                    format!("{}時間{}分{}秒", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}分{}秒", minutes, secs)
                } else {
                    format!("{}秒", secs)
                }
            }
            "zh-CN" => {
                if hours > 0 {
                    format!("{}小时{}分{}秒", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}分{}秒", minutes, secs)
                } else {
                    format!("{}秒", secs)
                }
            }
            "ko-KR" => {
                if hours > 0 {
                    format!("{}시간 {}분 {}초", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}분 {}초", minutes, secs)
                } else {
                    format!("{}초", secs)
                }
            }
            "de-DE" => {
                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
            "fr-FR" => {
                if hours > 0 {
                    format!("{}h {}min {}s", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}min {}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
            _ => {
                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
        }
    }
    
    /// Get pluralization rule for current locale
    pub fn get_plural_form(&self, count: i64) -> PluralForm {
        match self.current_locale.as_str() {
            "en-US" => {
                if count == 1 { PluralForm::One } else { PluralForm::Other }
            }
            "ja-JP" | "zh-CN" | "ko-KR" => {
                // These languages don't have plural forms
                PluralForm::Other
            }
            "ru-RU" => {
                // Russian has complex plural rules
                let n = count % 100;
                let n10 = count % 10;
                
                if n10 == 1 && n != 11 {
                    PluralForm::One
                } else if (2..=4).contains(&n10) && !(12..=14).contains(&n) {
                    PluralForm::Few
                } else {
                    PluralForm::Many
                }
            }
            "de-DE" | "fr-FR" | "es-ES" | "it-IT" | "pt-BR" => {
                if count == 1 { PluralForm::One } else { PluralForm::Other }
            }
            _ => {
                if count == 1 { PluralForm::One } else { PluralForm::Other }
            }
        }
    }
    
    /// Get localized message with plural support
    pub fn get_plural(&self, key: &str, count: i64) -> String {
        let plural_form = self.get_plural_form(count);
        let plural_key = match plural_form {
            PluralForm::Zero => format!("{}.zero", key),
            PluralForm::One => format!("{}.one", key),
            PluralForm::Two => format!("{}.two", key),
            PluralForm::Few => format!("{}.few", key),
            PluralForm::Many => format!("{}.many", key),
            PluralForm::Other => format!("{}.other", key),
        };
        
        let message = self.get(&plural_key);
        if message == plural_key {
            // Fallback to base key if plural form not found
            self.get(key)
        } else {
            // Replace count placeholder
            message.replace("{count}", &count.to_string())
        }
    }
}

/// Plural forms for different languages
#[derive(Debug, Clone, PartialEq)]
pub enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
} 