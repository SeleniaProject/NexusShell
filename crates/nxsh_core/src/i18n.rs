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
    fn parse_fluent_file(&self, _content: &str) -> crate::error::ShellResult<HashMap<String, String>> {
        // TODO: Implement proper Fluent file parsing
        // For now, return empty map
        Ok(HashMap::new())
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
        
        // Simple placeholder replacement
        for (arg_key, arg_value) in args {
            let placeholder = format!("{{{}}}", arg_key);
            message = message.replace(&placeholder, arg_value);
        }
        
        message
    }

    /// Format number according to locale
    pub fn format_number(&self, number: f64) -> String {
        // TODO: Implement locale-specific number formatting
        match self.current_locale.as_str() {
            "de-DE" | "fr-FR" | "es-ES" | "it-IT" => {
                // European format: 1.234,56
                format!("{:.2}", number).replace('.', ",")
            }
            _ => {
                // Default format: 1,234.56
                format!("{:.2}", number)
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
            "en-US" => format!("${:.2}", amount),
            "ja-JP" => format!("¥{:.0}", amount),
            "de-DE" => format!("{:.2} €", amount),
            "fr-FR" => format!("{:.2} €", amount),
            "es-ES" => format!("{:.2} €", amount),
            "it-IT" => format!("{:.2} €", amount),
            "pt-BR" => format!("R$ {:.2}", amount),
            "ru-RU" => format!("{:.2} ₽", amount),
            "zh-CN" => format!("¥{:.2}", amount),
            "ko-KR" => format!("₩{:.0}", amount),
            _ => format!("{:.2} {}", amount, currency),
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

        let _content = std::fs::read_to_string(&locale_file)
            .map_err(|e| crate::error::ShellError::new(
                crate::error::ErrorKind::IoError(crate::error::IoErrorKind::FileReadError),
                format!("Failed to read locale file: {}", e)
            ))?;

        // TODO: Validate Fluent file syntax
        Ok(true)
    }
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("locales"))
    }
} 