use once_cell::sync::OnceCell;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter};

static INIT: OnceCell<()> = OnceCell::new();

/// Initialize global logger with optional level filter.
/// Safe to call multiple times; initialization happens only once.
pub fn init(level: Option<Level>) {
    INIT.get_or_init(|| {
        let filter = if let Some(lvl) = level {
            EnvFilter::default().add_directive(lvl.into())
        } else if let Ok(var) = std::env::var("NXSH_LOG") {
            EnvFilter::new(var)
        } else {
            EnvFilter::new("info")
        };

        fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .init();
    });
}

/// Log an internationalized informational message.
/// `msg_ja`: Japanese message, `msg_en`: English message.
pub fn info_i18n(msg_ja: &str, msg_en: &str) {
    if is_lang_ja() {
        info!("{}", msg_ja);
    } else {
        info!("{}", msg_en);
    }
}

/// Detect if current locale is Japanese.
fn is_lang_ja() -> bool {
    std::env::var("LANG")
        .map(|l| l.starts_with("ja"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logger_initializes_once() {
        init(Some(Level::INFO));
        init(Some(Level::DEBUG)); // should not panic
        info!("Test message");
    }
} 