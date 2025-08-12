//! Internationalization support (full + stub fallback).
//!
//! When the `i18n` feature is enabled we compile the full Fluent based
//! localization system. When it is disabled we still expose the same public
//! API surface (types, functions and the `t!` macro) but they degrade to
//! simple passthroughs that just return the lookup key. This guarantees that
//! builtin command code can freely call localization helpers without having
//! to scatter `#[cfg(feature = "i18n")]` everywhere, keeping the BusyBox
//! minimal build simple and fast.
//!
//! The stub implementation is intentionally tiny and has zero dependencies
//! beyond `once_cell` / `parking_lot` already in the tree.

#![allow(clippy::arc_with_non_send_sync)]

// ===================== FULL IMPLEMENTATION =====================
#[cfg(feature = "i18n")]
mod full {
    use fluent::{FluentBundle, FluentResource};
    use fluent_bundle::FluentArgs;
    use std::collections::HashMap;
    use std::sync::{OnceLock, Arc};
    use unic_langid::LanguageIdentifier;
    use anyhow::{Result, anyhow};
    use parking_lot::Mutex;

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

            if lang.starts_with("ja") { Language::Japanese }
            else if lang.starts_with("zh") { Language::Chinese }
            else if lang.starts_with("ko") { Language::Korean }
            else if lang.starts_with("es") { Language::Spanish }
            else if lang.starts_with("fr") { Language::French }
            else if lang.starts_with("de") { Language::German }
            else if lang.starts_with("ru") { Language::Russian }
            else if lang.starts_with("pt") { Language::Portuguese }
            else if lang.starts_with("it") { Language::Italian }
            else { Language::English }
        }
    }

    /// Global localization manager (thread-safe with Fluent)
    #[derive(Clone)]
    pub struct I18n {
        bundles: Arc<Mutex<HashMap<Language, FluentBundle<FluentResource>>>>,
        current_language: Arc<Mutex<Language>>,
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

    impl Default for I18n { fn default() -> Self { Self::new() } }

    impl I18n {
        pub fn new() -> Self { Self::global().clone() }
        pub fn init() -> Result<()> {
            let mut bundles = HashMap::new();
            for lang in [
                Language::English, Language::Japanese, Language::Chinese, Language::Korean,
                Language::Spanish, Language::French, Language::German, Language::Russian,
                Language::Portuguese, Language::Italian,
            ] {
                let mut bundle = FluentBundle::new(vec![lang.lang_id()]);
                let resource = Self::load_language_resource(lang)?;
                bundle.add_resource(resource)
                    .map_err(|_| anyhow!("Failed to add resource for language {:?}", lang))?;
                bundles.insert(lang, bundle);
            }
            let i18n = I18n {
                bundles: Arc::new(Mutex::new(bundles)),
                current_language: Arc::new(Mutex::new(Language::from_env())),
            };
            I18N.set(i18n).map_err(|_| anyhow!("Failed to initialize i18n"))?;
            Ok(())
        }
        pub fn global() -> &'static I18n { I18N.get_or_init(|| I18n {
            bundles: Arc::new(Mutex::new(HashMap::new())),
            current_language: Arc::new(Mutex::new(Language::English)),
        }) }
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
        pub fn get(&self, key: &str, args: Option<&FluentArgs>) -> String {
            let current_lang = *self.current_language.lock();
            let bundles = self.bundles.lock();
            let bundle = bundles.get(&current_lang).or_else(|| bundles.get(&Language::English));
            if let Some(bundle) = bundle {
                let msg = bundle.get_message(key).and_then(|m| m.value());
                if let Some(pattern) = msg {
                    let mut errors = vec![];
                    let formatted = bundle.format_pattern(pattern, args, &mut errors);
                    if !errors.is_empty() { tracing::warn!(?errors, key, "translation errors"); }
                    return formatted.into_owned();
                }
            }
            key.to_string()
        }
        pub fn get_single(&self, key: &str) -> String { self.get(key, None) }
        pub fn set_language(&self, lang: Language) { *self.current_language.lock() = lang; }
        pub fn current_language(&self) -> Language { *self.current_language.lock() }
        pub fn current_locale(&self) -> String {
            match self.current_language() {
                Language::English => "en-US",
                Language::Japanese => "ja-JP",
                Language::Chinese => "zh-CN",
                Language::Spanish => "es-ES",
                Language::French => "fr-FR",
                Language::German => "de-DE",
                Language::Russian => "ru-RU",
                Language::Korean => "ko-KR",
                Language::Portuguese => "pt-BR",
                Language::Italian => "it-IT",
            }.to_string()
        }
    }

    #[macro_export]
    macro_rules! t {
        ($key:expr) => { $crate::common::i18n::I18n::global().get($key, None) };
        ($key:expr, $($name:expr => $value:expr),+ ) => {{
            let mut args = fluent_bundle::FluentArgs::new();
            $( args.set($name, $value); )+
            $crate::common::i18n::I18n::global().get($key, Some(&args))
        }};
    }

    pub fn init_i18n() -> Result<()> { I18n::init() }
    pub fn t(key: &str) -> String { I18n::global().get(key, None) }
    pub fn t_args(key: &str, args: &FluentArgs) -> String { I18n::global().get(key, Some(args)) }

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
    }
    // (Re-export不要: 最上位で #[cfg(feature="i18n")] pub use full::*; しているため重複定義を避ける)
}

// ===================== STUB IMPLEMENTATION =====================
#[cfg(not(feature = "i18n"))]
mod stub {
    use std::sync::OnceLock;
    use anyhow::Result;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Language { English }

    #[derive(Clone, Debug)]
    pub struct I18n;
    static I18N: OnceLock<I18n> = OnceLock::new();
    impl I18n {
        pub fn init() -> Result<()> { Ok(()) }
        pub fn global() -> &'static I18n { I18N.get_or_init(|| I18n) }
        pub fn new() -> I18n { I18n }
        pub fn get(&self, key:&str, _args: Option<&FluentArgs>) -> String { key.to_string() }
        pub fn current_locale(&self) -> String { "en-US".to_string() }
    }

    #[macro_export]
    macro_rules! t { ($key:expr) => { $key }; ($key:expr, $($name:expr => $value:expr),+ ) => { $key }; }

    pub fn init_i18n() -> Result<()> { Ok(()) }
    pub fn t(key:&str) -> String { key.to_string() }
    pub fn t_args(key:&str, _args: &FluentArgs) -> String { key.to_string() }

    // Dummy type so call sites using FluentArgs behind feature gates can still compile if they accidentally import it.
    pub struct FluentArgs; // zero sized placeholder

    // (no re-export here; top-level does conditional pub use)
}

// Public re-exports selecting the active implementation.
#[cfg(feature = "i18n")]
pub use full::*;
#[cfg(not(feature = "i18n"))]
pub use stub::{I18n, Language, FluentArgs, init_i18n, t, t_args};
