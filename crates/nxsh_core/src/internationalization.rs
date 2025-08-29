type I18nHook = std::sync::Arc<dyn Fn(&LanguagePack) -> Result<()> + Send + Sync>;
use crate::compat::Result; // Removed unused Context import
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf, // Re-added PathBuf needed by multiple function signatures
    sync::{Arc, Mutex},
    time::SystemTime,
};

/// Comprehensive internationalization and localization system  
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InternationalizationSystem {
    language_packs: HashMap<String, LanguagePack>,
    current_locale: String,
    fallback_locale: String,
    message_cache: Arc<Mutex<HashMap<String, String>>>,
    date_formatters: HashMap<String, DateFormatter>,
    number_formatters: HashMap<String, NumberFormatter>,
    currency_formatters: HashMap<String, CurrencyFormatter>,
    pluralization_rules: HashMap<String, PluralizationRule>,
    text_direction: HashMap<String, TextDirection>,
    collation_rules: HashMap<String, CollationRule>,
    validators: Vec<TranslationValidator>,
    context_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl InternationalizationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            language_packs: HashMap::new(),
            current_locale: "en-US".to_string(),
            fallback_locale: "en-US".to_string(),
            message_cache: Arc::new(Mutex::new(HashMap::new())),
            date_formatters: HashMap::new(),
            number_formatters: HashMap::new(),
            currency_formatters: HashMap::new(),
            pluralization_rules: HashMap::new(),
            text_direction: HashMap::new(),
            collation_rules: HashMap::new(),
            validators: Vec::new(),
            context_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        system.initialize_default_language_packs();
        system.register_default_formatters();
        system.register_pluralization_rules();
        system.register_collation_rules();
        system.register_translation_validators();
        system
    }

    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) -> Result<()> {
        if !self.language_packs.contains_key(locale) {
            return Err(crate::anyhow!(
                "Language pack for locale '{}' not found",
                locale
            ));
        }

        // Clear message cache when locale changes
        {
            let mut cache = self.message_cache.lock().unwrap();
            cache.clear();
        }

        self.current_locale = locale.to_string();
        Ok(())
    }

    /// Get localized message with parameters
    pub fn get_message(&self, key: &str, params: Option<HashMap<String, String>>) -> String {
        // Check cache first
        {
            let cache = self.message_cache.lock().unwrap();
            if let Some(cached) = cache.get(key) {
                return if let Some(ref p) = params {
                    self.interpolate_message(cached, p)
                } else {
                    cached.clone()
                };
            }
        }

        // Try current locale first
        if let Some(language_pack) = self.language_packs.get(&self.current_locale) {
            if let Some(message) = language_pack.messages.get(key) {
                let result = if let Some(ref p) = params {
                    self.interpolate_message(message, p)
                } else {
                    message.clone()
                };

                // Cache the result
                {
                    let mut cache = self.message_cache.lock().unwrap();
                    cache.insert(key.to_string(), result.clone());
                }

                return result;
            }
        }

        // Fall back to fallback locale
        if let Some(language_pack) = self.language_packs.get(&self.fallback_locale) {
            if let Some(message) = language_pack.messages.get(key) {
                let result = if let Some(ref p) = params {
                    self.interpolate_message(message, p)
                } else {
                    message.clone()
                };

                return result;
            }
        }

        // If no message found, return the key itself
        format!("[{key}]")
    }

    /// Interpolate parameters into message template
    fn interpolate_message(&self, template: &str, params: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in params {
            result = result.replace(&format!("{{{key}}}"), value);
        }

        result
    }

    /// Get pluralized message
    pub fn get_plural_message(
        &self,
        key: &str,
        count: i64,
        params: Option<HashMap<String, String>>,
    ) -> String {
        // Determine plural form based on current locale
        let plural_form = if let Some(rule) = self.pluralization_rules.get(&self.current_locale) {
            (rule.rule_function)(count)
        } else {
            // Default English pluralization
            if count == 1 {
                PluralForm::One
            } else {
                PluralForm::Other
            }
        };

        let plural_key = format!("{}_{}", key, plural_form.to_key());

        let mut final_params = params.unwrap_or_default();
        final_params.insert("count".to_string(), count.to_string());

        self.get_message(&plural_key, Some(final_params))
    }

    /// Format date according to current locale
    pub fn format_date(&self, date: SystemTime, format: DateFormat) -> String {
        if let Some(formatter) = self.date_formatters.get(&self.current_locale) {
            formatter.format(date, format)
        } else {
            // Fallback formatting
            format!("{date:?}")
        }
    }

    /// Format number according to current locale
    pub fn format_number(&self, number: f64, format: NumberFormat) -> String {
        if let Some(formatter) = self.number_formatters.get(&self.current_locale) {
            formatter.format(number, format)
        } else {
            // Fallback formatting
            format!("{number}")
        }
    }

    /// Format currency according to current locale
    pub fn format_currency(&self, amount: f64, currency_code: &str) -> String {
        if let Some(formatter) = self.currency_formatters.get(&self.current_locale) {
            formatter.format(amount, currency_code)
        } else {
            // Fallback formatting
            format!("{currency_code} {amount}")
        }
    }

    /// Get text direction for current locale
    pub fn get_text_direction(&self) -> TextDirection {
        self.text_direction
            .get(&self.current_locale)
            .cloned()
            .unwrap_or(TextDirection::LeftToRight)
    }

    /// Load language pack from file
    pub fn load_language_pack(&mut self, locale: &str, file_path: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let language_pack: LanguagePack = serde_json::from_str(&content)?;

        self.language_packs
            .insert(locale.to_string(), language_pack);
        Ok(())
    }

    /// Export language pack to file
    pub fn export_language_pack(&self, locale: &str, file_path: &PathBuf) -> Result<()> {
        if let Some(language_pack) = self.language_packs.get(locale) {
            let content = serde_json::to_string_pretty(language_pack)?;
            fs::write(file_path, content)?;
            Ok(())
        } else {
            Err(crate::anyhow!(
                "Language pack for locale '{}' not found",
                locale
            ))
        }
    }

    /// Validate all language packs
    pub fn validate_language_packs(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            total_locales: self.language_packs.len(),
            validated_locales: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            missing_keys: HashMap::new(),
            extra_keys: HashMap::new(),
        };

        let base_keys: std::collections::HashSet<String> =
            if let Some(base_pack) = self.language_packs.get(&self.fallback_locale) {
                base_pack.messages.keys().cloned().collect()
            } else {
                return Err(crate::anyhow!(
                    "Base language pack '{}' not found",
                    self.fallback_locale
                ));
            };

        for (locale, language_pack) in &self.language_packs {
            // Run custom validators
            for validator in &self.validators {
                if let Err(e) = (validator.validate_function)(language_pack) {
                    report
                        .errors
                        .push(format!("Validation error in '{locale}': {e}"));
                    continue;
                }
            }

            let pack_keys: std::collections::HashSet<String> =
                language_pack.messages.keys().cloned().collect();

            // Check for missing keys
            let missing: Vec<String> = base_keys.difference(&pack_keys).cloned().collect();
            if !missing.is_empty() {
                report.missing_keys.insert(locale.clone(), missing);
            }

            // Check for extra keys
            let extra: Vec<String> = pack_keys.difference(&base_keys).cloned().collect();
            if !extra.is_empty() {
                report.extra_keys.insert(locale.clone(), extra);
            }

            report.validated_locales += 1;
        }

        Ok(report)
    }

    /// Extract all translatable strings from source code
    pub fn extract_strings(&self, source_dirs: Vec<PathBuf>) -> Result<ExtractionReport> {
        let mut report = ExtractionReport {
            extracted_strings: Vec::new(),
            files_processed: 0,
            total_strings: 0,
        };

        // This is a simplified implementation
        // In practice, you'd use a proper parser to extract strings
        for dir in source_dirs {
            self.extract_from_directory(&dir, &mut report)?;
        }

        Ok(report)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_from_directory(&self, dir: &PathBuf, report: &mut ExtractionReport) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.extract_from_directory(&path, report)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Extract strings from Rust files
                let content = fs::read_to_string(&path)?;

                // Simple regex-based extraction (in practice, use a proper parser)
                for line in content.lines() {
                    if line.contains("get_message(") || line.contains("t!(") {
                        // Extract string key
                        // This is simplified - real implementation would be more sophisticated
                        if let Some(start) = line.find("\"") {
                            if let Some(end) = line[start + 1..].find("\"") {
                                let string_key = &line[start + 1..start + 1 + end];
                                report.extracted_strings.push(ExtractedString {
                                    key: string_key.to_string(),
                                    file: path.clone(),
                                    line_number: 0, // Simplified
                                    context: line.to_string(),
                                });
                                report.total_strings += 1;
                            }
                        }
                    }
                }

                report.files_processed += 1;
            }
        }

        Ok(())
    }

    // Default initialization methods
    fn initialize_default_language_packs(&mut self) {
        // English (US)
        let mut en_us = LanguagePack {
            locale: "en-US".to_string(),
            name: "English (United States)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "NexusShell Team".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 100.0,
            },
        };

        en_us
            .messages
            .insert("welcome".to_string(), "Welcome to NexusShell".to_string());
        en_us
            .messages
            .insert("goodbye".to_string(), "Goodbye!".to_string());
        en_us.messages.insert(
            "error_command_not_found".to_string(),
            "Command not found: {command}".to_string(),
        );
        en_us.messages.insert(
            "file_not_found".to_string(),
            "File not found: {filename}".to_string(),
        );
        en_us.messages.insert(
            "permission_denied".to_string(),
            "Permission denied".to_string(),
        );

        // Plural forms
        en_us
            .messages
            .insert("files_count_one".to_string(), "{count} file".to_string());
        en_us
            .messages
            .insert("files_count_other".to_string(), "{count} files".to_string());

        self.language_packs.insert("en-US".to_string(), en_us);

        // Spanish
        let mut es_es = LanguagePack {
            locale: "es-ES".to_string(),
            name: "Español (España)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "NexusShell Team".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 90.0,
            },
        };

        es_es
            .messages
            .insert("welcome".to_string(), "Bienvenido a NexusShell".to_string());
        es_es
            .messages
            .insert("goodbye".to_string(), "¡Adiós!".to_string());
        es_es.messages.insert(
            "error_command_not_found".to_string(),
            "Comando no encontrado: {command}".to_string(),
        );
        es_es.messages.insert(
            "file_not_found".to_string(),
            "Archivo no encontrado: {filename}".to_string(),
        );
        es_es.messages.insert(
            "permission_denied".to_string(),
            "Permiso denegado".to_string(),
        );

        self.language_packs.insert("es-ES".to_string(), es_es);

        // French
        let mut fr_fr = LanguagePack {
            locale: "fr-FR".to_string(),
            name: "Français (France)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "NexusShell Team".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 85.0,
            },
        };

        fr_fr.messages.insert(
            "welcome".to_string(),
            "Bienvenue dans NexusShell".to_string(),
        );
        fr_fr
            .messages
            .insert("goodbye".to_string(), "Au revoir!".to_string());
        fr_fr.messages.insert(
            "error_command_not_found".to_string(),
            "Commande introuvable: {command}".to_string(),
        );
        fr_fr.messages.insert(
            "file_not_found".to_string(),
            "Fichier introuvable: {filename}".to_string(),
        );
        fr_fr.messages.insert(
            "permission_denied".to_string(),
            "Permission refusée".to_string(),
        );

        self.language_packs.insert("fr-FR".to_string(), fr_fr);

        // German
        let mut de_de = LanguagePack {
            locale: "de-DE".to_string(),
            name: "Deutsch (Deutschland)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "NexusShell Team".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 80.0,
            },
        };

        de_de.messages.insert(
            "welcome".to_string(),
            "Willkommen bei NexusShell".to_string(),
        );
        de_de
            .messages
            .insert("goodbye".to_string(), "Auf Wiedersehen!".to_string());
        de_de.messages.insert(
            "error_command_not_found".to_string(),
            "Befehl nicht gefunden: {command}".to_string(),
        );
        de_de.messages.insert(
            "file_not_found".to_string(),
            "Datei nicht gefunden: {filename}".to_string(),
        );
        de_de.messages.insert(
            "permission_denied".to_string(),
            "Zugriff verweigert".to_string(),
        );

        self.language_packs.insert("de-DE".to_string(), de_de);

        // Japanese
        let mut ja_jp = LanguagePack {
            locale: "ja-JP".to_string(),
            name: "日本語 (日本)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "NexusShell Team".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 75.0,
            },
        };

        ja_jp
            .messages
            .insert("welcome".to_string(), "NexusShellへようこそ".to_string());
        ja_jp
            .messages
            .insert("goodbye".to_string(), "さようなら！".to_string());
        ja_jp.messages.insert(
            "error_command_not_found".to_string(),
            "コマンドが見つかりません: {command}".to_string(),
        );
        ja_jp.messages.insert(
            "file_not_found".to_string(),
            "ファイルが見つかりません: {filename}".to_string(),
        );
        ja_jp.messages.insert(
            "permission_denied".to_string(),
            "アクセスが拒否されました".to_string(),
        );

        self.language_packs.insert("ja-JP".to_string(), ja_jp);
    }

    fn register_default_formatters(&mut self) {
        // English formatters
        self.date_formatters.insert(
            "en-US".to_string(),
            DateFormatter {
                short_format: "%m/%d/%Y".to_string(),
                medium_format: "%b %d, %Y".to_string(),
                long_format: "%B %d, %Y".to_string(),
                full_format: "%A, %B %d, %Y".to_string(),
            },
        );

        self.number_formatters.insert(
            "en-US".to_string(),
            NumberFormatter {
                decimal_separator: ".".to_string(),
                group_separator: ",".to_string(),
                group_size: 3,
            },
        );

        self.currency_formatters.insert(
            "en-US".to_string(),
            CurrencyFormatter {
                symbol_position: CurrencySymbolPosition::Before,
                decimal_places: 2,
                group_separator: ",".to_string(),
                decimal_separator: ".".to_string(),
            },
        );

        // German formatters
        self.date_formatters.insert(
            "de-DE".to_string(),
            DateFormatter {
                short_format: "%d.%m.%Y".to_string(),
                medium_format: "%d. %b %Y".to_string(),
                long_format: "%d. %B %Y".to_string(),
                full_format: "%A, %d. %B %Y".to_string(),
            },
        );

        self.number_formatters.insert(
            "de-DE".to_string(),
            NumberFormatter {
                decimal_separator: ",".to_string(),
                group_separator: ".".to_string(),
                group_size: 3,
            },
        );

        // Add more formatters for other locales...
    }

    fn register_pluralization_rules(&mut self) {
        // English pluralization
        let en_rule = PluralizationRule {
            locale: "en-US".to_string(),
            rule_function: std::sync::Arc::new(|n| {
                if n == 1 {
                    PluralForm::One
                } else {
                    PluralForm::Other
                }
            }),
        };

        // French pluralization
        let fr_rule = PluralizationRule {
            locale: "fr-FR".to_string(),
            rule_function: std::sync::Arc::new(|n| {
                if n <= 1 {
                    PluralForm::One
                } else {
                    PluralForm::Other
                }
            }),
        };

        // German pluralization (similar to English)
        let de_rule = PluralizationRule {
            locale: "de-DE".to_string(),
            rule_function: std::sync::Arc::new(|n| {
                if n == 1 {
                    PluralForm::One
                } else {
                    PluralForm::Other
                }
            }),
        };

        self.pluralization_rules
            .insert("en-US".to_string(), en_rule);
        self.pluralization_rules
            .insert("fr-FR".to_string(), fr_rule);
        self.pluralization_rules
            .insert("de-DE".to_string(), de_rule);
    }

    fn register_collation_rules(&mut self) {
        // This is a simplified implementation
        // Real collation rules are much more complex

        for locale in ["en-US", "es-ES", "fr-FR", "de-DE", "ja-JP"] {
            let rule = CollationRule {
                locale: locale.to_string(),
                sort_order: CollationSortOrder::Dictionary,
                case_sensitive: false,
                accent_sensitive: true,
            };

            self.collation_rules.insert(locale.to_string(), rule);
        }

        // Japanese uses different sorting
        if let Some(ja_rule) = self.collation_rules.get_mut("ja-JP") {
            ja_rule.sort_order = CollationSortOrder::Phonetic;
        }
    }

    fn register_translation_validators(&mut self) {
        // Validator for checking message completeness
        let completeness_validator = TranslationValidator {
            name: "Completeness Validator".to_string(),
            validate_function: std::sync::Arc::new(|language_pack: &LanguagePack| -> Result<()> {
                if language_pack.messages.is_empty() {
                    return Err(crate::anyhow!("Language pack has no messages"));
                }

                for (key, message) in &language_pack.messages {
                    if message.trim().is_empty() {
                        return Err(crate::anyhow!("Empty message for key: {}", key));
                    }
                }

                Ok(())
            }),
        };

        // Validator for checking parameter consistency
        let parameter_validator = TranslationValidator {
            name: "Parameter Validator".to_string(),
            validate_function: std::sync::Arc::new(|language_pack: &LanguagePack| -> Result<()> {
                for (key, message) in &language_pack.messages {
                    // Check for unclosed parameter placeholders
                    let open_braces = message.matches('{').count();
                    let close_braces = message.matches('}').count();

                    if open_braces != close_braces {
                        return Err(crate::anyhow!(
                            "Mismatched parameter braces in key: {}",
                            key
                        ));
                    }
                }

                Ok(())
            }),
        };

        self.validators.push(completeness_validator);
        self.validators.push(parameter_validator);
    }

    /// Set text direction for specific locales
    pub fn set_text_directions(&mut self) {
        // Left-to-right languages
        for locale in [
            "en-US", "es-ES", "fr-FR", "de-DE", "ja-JP", "ko-KR", "zh-CN", "pt-BR", "it-IT",
            "ru-RU",
        ] {
            self.text_direction
                .insert(locale.to_string(), TextDirection::LeftToRight);
        }

        // Right-to-left languages
        for locale in ["ar-SA", "he-IL", "fa-IR", "ur-PK"] {
            self.text_direction
                .insert(locale.to_string(), TextDirection::RightToLeft);
        }
    }
}

// Supporting types and structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePack {
    pub locale: String,
    pub name: String,
    pub messages: HashMap<String, String>,
    pub metadata: LanguagePackMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePackMetadata {
    pub version: String,
    pub author: String,
    pub last_updated: SystemTime,
    pub completion_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct DateFormatter {
    pub short_format: String,
    pub medium_format: String,
    pub long_format: String,
    pub full_format: String,
}

impl DateFormatter {
    pub fn format(&self, _date: SystemTime, format: DateFormat) -> String {
        // This is a simplified implementation
        // Real implementation would use proper date formatting libraries
        match format {
            DateFormat::Short => "1/1/2024".to_string(),
            DateFormat::Medium => "Jan 1, 2024".to_string(),
            DateFormat::Long => "January 1, 2024".to_string(),
            DateFormat::Full => "Monday, January 1, 2024".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberFormatter {
    pub decimal_separator: String,
    pub group_separator: String,
    pub group_size: usize,
}

impl NumberFormatter {
    pub fn format(&self, number: f64, _format: NumberFormat) -> String {
        // Simplified implementation
        format!("{number}")
    }
}

#[derive(Debug, Clone)]
pub struct CurrencyFormatter {
    pub symbol_position: CurrencySymbolPosition,
    pub decimal_places: usize,
    pub group_separator: String,
    pub decimal_separator: String,
}

impl CurrencyFormatter {
    pub fn format(&self, amount: f64, currency_code: &str) -> String {
        // Simplified implementation
        match self.symbol_position {
            CurrencySymbolPosition::Before => format!("{currency_code}{amount:.2}"),
            CurrencySymbolPosition::After => format!("{amount:.2}{currency_code}"),
        }
    }
}

// Pluralization

#[derive(Clone)]
pub struct PluralizationRule {
    pub locale: String,
    #[doc = "Function stored as Arc for cloneability"]
    pub rule_function: std::sync::Arc<dyn Fn(i64) -> PluralForm + Send + Sync>,
}

impl std::fmt::Debug for PluralizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluralizationRule")
            .field("locale", &self.locale)
            .field("rule_function", &"<function>")
            .finish()
    }
}

impl PluralizationRule {
    pub fn get_plural_form(&self, count: i64) -> PluralForm {
        (self.rule_function)(count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralForm {
    pub fn to_key(&self) -> &'static str {
        match self {
            PluralForm::Zero => "zero",
            PluralForm::One => "one",
            PluralForm::Two => "two",
            PluralForm::Few => "few",
            PluralForm::Many => "many",
            PluralForm::Other => "other",
        }
    }
}

// Validation

#[derive(Clone)]
pub struct TranslationValidator {
    pub name: String,
    #[doc = "Function stored as Arc for cloneability"]
    pub validate_function: I18nHook,
}

impl std::fmt::Debug for TranslationValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranslationValidator")
            .field("name", &self.name)
            .field("validate_function", &"<function>")
            .finish()
    }
}

impl TranslationValidator {
    pub fn validate(&self, language_pack: &LanguagePack) -> Result<()> {
        (self.validate_function)(language_pack)
    }
}

// Collation and sorting

#[derive(Debug, Clone)]
pub struct CollationRule {
    pub locale: String,
    pub sort_order: CollationSortOrder,
    pub case_sensitive: bool,
    pub accent_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CollationSortOrder {
    Dictionary,
    Phonetic,
    Numeric,
    Custom(String),
}

// Enums and supporting types

#[derive(Debug, Clone, PartialEq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateFormat {
    Short,
    Medium,
    Long,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormat {
    Decimal,
    Percentage,
    Scientific,
    Currency,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrencySymbolPosition {
    Before,
    After,
}

// Report structures

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub total_locales: usize,
    pub validated_locales: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub missing_keys: HashMap<String, Vec<String>>,
    pub extra_keys: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionReport {
    pub extracted_strings: Vec<ExtractedString>,
    pub files_processed: usize,
    pub total_strings: usize,
}

#[derive(Debug, Clone)]
pub struct ExtractedString {
    pub key: String,
    pub file: PathBuf,
    pub line_number: usize,
    pub context: String,
}

// Convenience macros for translation

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        // This would integrate with a global translation system
        format!("[{}]", $key)
    };
    ($key:expr, $($param_key:ident = $param_value:expr),*) => {
        {
            let mut params = std::collections::HashMap::new();
            $(
                params.insert(stringify!($param_key).to_string(), $param_value.to_string());
            )*
            // This would integrate with a global translation system
            format!("[{}]", $key)
        }
    };
}

#[macro_export]
macro_rules! tn {
    ($key:expr, $count:expr) => {
        // This would integrate with a global translation system for pluralization
        format!("[{}_{}]", $key, if $count == 1 { "one" } else { "other" })
    };
    ($key:expr, $count:expr, $($param_key:ident = $param_value:expr),*) => {
        {
            let mut params = std::collections::HashMap::new();
            params.insert("count".to_string(), $count.to_string());
            $(
                params.insert(stringify!($param_key).to_string(), $param_value.to_string());
            )*
            // This would integrate with a global translation system
            format!("[{}_{}]", $key, if $count == 1 { "one" } else { "other" })
        }
    };
}

// Default implementations and factory methods

impl Default for InternationalizationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LanguagePack {
    fn default() -> Self {
        Self {
            locale: "en-US".to_string(),
            name: "English (United States)".to_string(),
            messages: HashMap::new(),
            metadata: LanguagePackMetadata {
                version: "1.0.0".to_string(),
                author: "Unknown".to_string(),
                last_updated: SystemTime::now(),
                completion_percentage: 0.0,
            },
        }
    }
}

// Default is derived above for ValidationReport

// Default is derived above for ExtractionReport

// Integration with global context system
impl InternationalizationSystem {
    pub fn create_context_aware_translator(&self) -> ContextAwareTranslator {
        ContextAwareTranslator {
            i18n_system: self.clone(),
            current_context: None,
            context_stack: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextAwareTranslator {
    i18n_system: InternationalizationSystem,
    current_context: Option<String>,
    context_stack: Vec<String>,
}

impl ContextAwareTranslator {
    pub fn push_context(&mut self, context: &str) {
        if let Some(current) = &self.current_context {
            self.context_stack.push(current.clone());
        }
        self.current_context = Some(context.to_string());
    }

    pub fn pop_context(&mut self) {
        self.current_context = self.context_stack.pop();
    }

    pub fn translate(&self, key: &str) -> String {
        let full_key = if let Some(context) = &self.current_context {
            format!("{context}.{key}")
        } else {
            key.to_string()
        };

        self.i18n_system.get_message(&full_key, None)
    }

    pub fn translate_with_params(&self, key: &str, params: HashMap<String, String>) -> String {
        let full_key = if let Some(context) = &self.current_context {
            format!("{context}.{key}")
        } else {
            key.to_string()
        };

        self.i18n_system.get_message(&full_key, Some(params))
    }
}
