//! Internationalization support for NexusShell builtin commands.
//!
//! This module provides comprehensive i18n support using the Fluent localization system.
//! All user-facing strings are localized and support multiple languages.

use fluent::{FluentBundle, FluentResource};
use fluent_bundle::FluentArgs;
use std::collections::HashMap;
use std::sync::OnceLock;
use unic_langid::{LanguageIdentifier, langid};
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
            Language::English => langid!("en-US"),
            Language::Japanese => langid!("ja-JP"),
            Language::Chinese => langid!("zh-CN"),
            Language::Korean => langid!("ko-KR"),
            Language::Spanish => langid!("es-ES"),
            Language::French => langid!("fr-FR"),
            Language::German => langid!("de-DE"),
            Language::Russian => langid!("ru-RU"),
            Language::Portuguese => langid!("pt-BR"),
            Language::Italian => langid!("it-IT"),
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

/// Global localization manager
pub struct I18n {
    bundles: HashMap<Language, FluentBundle<FluentResource>>,
    current_language: Language,
}

static I18N: OnceLock<I18n> = OnceLock::new();

impl I18n {
    /// Initialize the global i18n instance
    pub fn init() -> Result<()> {
        let mut i18n = I18n {
            bundles: HashMap::new(),
            current_language: Language::from_env(),
        };

        // Load all language resources
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
            i18n.bundles.insert(lang, bundle);
        }

        I18N.set(i18n).map_err(|_| anyhow!("Failed to initialize i18n"))?;
        Ok(())
    }

    /// Get the global i18n instance
    pub fn global() -> &'static I18n {
        I18N.get().expect("I18n not initialized")
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
        let bundle = self.bundles.get(&self.current_language)
            .or_else(|| self.bundles.get(&Language::English))
            .expect("No language bundle available");

        let msg = bundle.get_message(key)
            .and_then(|msg| msg.value())
            .unwrap_or_else(|| {
                tracing::warn!("Missing translation key: {}", key);
                return key.into();
            });

        let mut errors = vec![];
        let formatted = bundle.format_pattern(msg, args, &mut errors);
        
        if !errors.is_empty() {
            tracing::warn!("Translation errors for key {}: {:?}", key, errors);
        }

        formatted.into_owned()
    }

    /// Set current language
    pub fn set_language(&mut self, lang: Language) {
        self.current_language = lang;
    }

    /// Get current language
    pub fn current_language(&self) -> Language {
        self.current_language
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
    fn test_lang_id() {
        assert_eq!(Language::English.lang_id(), langid!("en-US"));
        assert_eq!(Language::Japanese.lang_id(), langid!("ja-JP"));
    }
} 