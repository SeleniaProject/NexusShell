//! Internationalization support for NexusShell builtin commands.
//!
//! This module provides comprehensive i18n support using the Fluent localization system.
//! All user-facing strings are localized and support multiple languages.

use fluent::{FluentBundle, FluentResource};
use fluent_bundle::FluentArgs;
use std::collections::HashMap;
use std::sync::{OnceLock, Arc};
use unic_langid::LanguageIdentifier;
use anyhow::{Result, anyhow};

/// Supported languages in NexusShell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Japanese,
    Chinese,
    Korean,
    Spanish,
    French,
    German,
    Russian,
    Portuguese,
    Italian,
}

impl Language {
    pub fn lang_id(&self) -> LanguageIdentifier {
        match self {
            Language::English => "en-US".parse().unwrap(),
            Language::Japanese => "ja-JP".parse().unwrap(),
            Language::Chinese => "zh-CN".parse().unwrap(),
            Language::Korean => "ko-KR".parse().unwrap(),
            Language::Spanish => "es-ES".parse().unwrap(),
            Language::French => "fr-FR".parse().unwrap(),
            Language::German => "de-DE".parse().unwrap(),
            Language::Russian => "ru-RU".parse().unwrap(),
            Language::Portuguese => "pt-BR".parse().unwrap(),
            Language::Italian => "it-IT".parse().unwrap(),
        }
    }

    pub fn from_env() -> Self {
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .unwrap_or_else(|_| "en_US.UTF-8".to_string());

        if lang.starts_with("ja") {
            Language::Japanese
        } else if lang.starts_with("zh") {
            Language::Chinese
        } else if lang.starts_with("ko") {
            Language::Korean
        } else if lang.starts_with("es") {
            Language::Spanish
        } else if lang.starts_with("fr") {
            Language::French
        } else if lang.starts_with("de") {
            Language::German
        } else if lang.starts_with("ru") {
            Language::Russian
        } else if lang.starts_with("pt") {
            Language::Portuguese
        } else if lang.starts_with("it") {
            Language::Italian
        } else {
            Language::English
        }
    }
}

/// Global localization manager (thread-safe with Fluent)
#[derive(Clone)]
pub struct I18n {
    bundles: Arc<parking_lot::Mutex<HashMap<Language, FluentBundle<FluentResource>>>>,
    current_language: Arc<parking_lot::Mutex<Language>>,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n")
            .field("current_language", &*self.current_language.lock())
            .field("bundles_count", &self.bundles.lock().len())
            .finish()
    }
}

unsafe impl Send for I18n {}
unsafe impl Sync for I18n {}

static I18N: OnceLock<I18n> = OnceLock::new();

impl I18n {
    /// Create a new I18n instance (for compatibility)
    pub fn new() -> Self {
        Self::global().clone()
    }

    /// Initialize the global i18n instance
    pub fn init() -> Result<()> {
        let mut bundles = HashMap::new();
        
        // Load all language bundles
        for lang in [
            Language::English,
            Language::Japanese,
            Language::Chinese,
            Language::Korean,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Russian,
            Language::Portuguese,
            Language::Italian,
        ] {
            let mut bundle = FluentBundle::new(vec![lang.lang_id()]);
            let resource = Self::load_language_resource(lang)?;
            bundle.add_resource(resource)
                .map_err(|_| anyhow!("Failed to add resource for language {:?}", lang))?;
            bundles.insert(lang, bundle);
        }

        let i18n = I18n {
            bundles: Arc::new(parking_lot::Mutex::new(bundles)),
            current_language: Arc::new(parking_lot::Mutex::new(Language::from_env())),
        };

        I18N.set(i18n).map_err(|_| anyhow!("Failed to initialize i18n"))?;
        Ok(())
    }

    /// Get the global i18n instance
    pub fn global() -> &'static I18n {
        I18N.get_or_init(|| {
            I18n {
                bundles: Arc::new(parking_lot::Mutex::new(HashMap::new())),
                current_language: Arc::new(parking_lot::Mutex::new(Language::English)),
            }
        })
    }

    /// Load language resource for a specific language
    fn load_language_resource(lang: Language) -> Result<FluentResource> {
        let content = match lang {
            Language::English => include_str!("../../locales/en-US.ftl"),
            Language::Japanese => include_str!("../../locales/ja-JP.ftl"),
            Language::Chinese => include_str!("../../locales/zh-CN.ftl"),
            Language::Korean => include_str!("../../locales/ko-KR.ftl"),
            Language::Spanish => include_str!("../../locales/es-ES.ftl"),
            Language::French => include_str!("../../locales/fr-FR.ftl"),
            Language::German => include_str!("../../locales/de-DE.ftl"),
            Language::Russian => include_str!("../../locales/ru-RU.ftl"),
            Language::Portuguese => include_str!("../../locales/pt-BR.ftl"),
            Language::Italian => include_str!("../../locales/it-IT.ftl"),
        };

        FluentResource::try_new(content.to_string())
            .map_err(|_| anyhow!("Failed to parse fluent resource for {:?}", lang))
    }

    /// Get localized message
    pub fn get(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let current_lang = *self.current_language.lock();
        let bundles = self.bundles.lock();
        
        let bundle = bundles.get(&current_lang)
            .or_else(|| bundles.get(&Language::English));

        if let Some(bundle) = bundle {
            let msg = bundle.get_message(key)
                .and_then(|msg| msg.value());
                
            if let Some(pattern) = msg {
                let mut errors = vec![];
                let formatted = bundle.format_pattern(pattern, args, &mut errors);
                
                if !errors.is_empty() {
                    tracing::warn!("Translation errors for key {}: {:?}", key, errors);
                }
                
                return formatted.into_owned();
            }
        }
        
        // Fallback to key if translation not found
        key.to_string()
    }
    
    /// Get localized message with single argument (for compatibility)
    pub fn get_single(&self, key: &str) -> String {
        self.get(key, None)
    }

    /// Set current language
    pub fn set_language(&self, lang: Language) {
        *self.current_language.lock() = lang;
    }

    /// Get current language
    pub fn current_language(&self) -> Language {
        *self.current_language.lock()
    }

    /// Get current locale string (for compatibility)
    pub fn current_locale(&self) -> String {
        match self.current_language() {
            Language::English => "en-US".to_string(),
            Language::Japanese => "ja-JP".to_string(),
            Language::Chinese => "zh-CN".to_string(),
            Language::Spanish => "es-ES".to_string(),
            Language::French => "fr-FR".to_string(),
            Language::German => "de-DE".to_string(),
            Language::Russian => "ru-RU".to_string(),
            Language::Korean => "ko-KR".to_string(),
            Language::Portuguese => "pt-BR".to_string(),
            Language::Italian => "it-IT".to_string(),
        }
    }
}

/// Convenience macro for getting localized strings
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::common::i18n::I18n::global().get($key, None)
    };
    ($key:expr, $($name:expr => $value:expr),+) => {{
        let mut args = fluent_bundle::FluentArgs::new();
        $(
            args.set($name, $value);
        )+
        $crate::common::i18n::I18n::global().get($key, Some(&args))
    }};
}

/// Initialize i18n system - should be called once at startup
pub fn init_i18n() -> Result<()> {
    I18n::init()
}

/// Get localized string
pub fn t(key: &str) -> String {
    I18n::global().get(key, None)
}

/// Get localized string with arguments
pub fn t_args(key: &str, args: &FluentArgs) -> String {
    I18n::global().get(key, Some(args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        std::env::set_var("LANG", "ja_JP.UTF-8");
        assert_eq!(Language::from_env(), Language::Japanese);

        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(Language::from_env(), Language::English);
    }

    #[test]
    fn test_lang_code() {
        assert_eq!(Language::English.lang_id(), "en-US".parse().unwrap());
        assert_eq!(Language::Japanese.lang_id(), "ja-JP".parse().unwrap());
    }
} 
