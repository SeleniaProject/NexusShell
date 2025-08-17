// Minimal internationalization example (heavily simplified & feature gated)
// If the internationalization feature is disabled we just print a stub message.

#[cfg(feature = "internationalization")]
use nxsh_core::i18n::I18nManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "internationalization"))]
    {
        println!("(i18n_test example built without internationalization feature)");
    }

    #[cfg(feature = "internationalization")]
    {
        let locale_dir = std::path::PathBuf::from("crates/nxsh_builtins/locales");
        let mut i18n = I18nManager::new(locale_dir);
        println!("Supported locales: {:?}", i18n.supported_locales());
        let test_number = 1234.56;
        for loc in &["en-US", "ja-JP", "de-DE"] {
            if i18n.set_locale(loc).is_ok() {
                println!("{} => {}", loc, i18n.format_number(test_number));
            }
        }
    }

    Ok(())
}
