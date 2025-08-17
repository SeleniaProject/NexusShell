//! Locale-aware formatting utilities (numbers, dates, sizes) with light dependencies.
//! Avoids heavy ICU and compiles even when i18n features are disabled.
use num_format::{Locale as NumLocale, ToFormattedString};
use chrono::{DateTime, Local, TimeZone};

// Extract primary language subtag in lowercase from a BCP47-ish string.
// Examples: "ja-JP" -> "ja", "pt_BR" -> "pt", "EN" -> "en".
fn lang_code(langid: &str) -> String {
    let lower = langid.to_ascii_lowercase();
    let mut it = lower.split(|c| c == '-' || c == '_');
    it.next().unwrap_or("en").to_string()
}

fn resolve_num_locale(code: &str) -> NumLocale {
    match code {
        "fr" => NumLocale::fr,
        "de" => NumLocale::de,
        "ru" => NumLocale::ru,
        "it" => NumLocale::it,
        "es" => NumLocale::es,
        // Japanese and Korean: western grouping
        "ja" => NumLocale::en,
        "ko" => NumLocale::en,
        "pt" => NumLocale::pt,
        _ => NumLocale::en,
    }
}

pub fn format_integer_locale(value: i64, langid: &str) -> String {
    let code = lang_code(langid);
    let loc = resolve_num_locale(&code);
    value.to_formatted_string(&loc)
}

pub fn format_float_locale(value: f64, precision: usize, langid: &str) -> String {
    let code = lang_code(langid);
    let s = format!("{:.*}", precision, value);
    if let Some(dot) = s.find('.') {
        let (int_part, frac_part) = s.split_at(dot);
        let int_val: i64 = int_part.parse().unwrap_or(0);
        let grouped = format_integer_locale(int_val, langid);
        let dec_sep = match code.as_str() { "fr" | "de" | "it" | "es" | "pt" | "ru" => ',', _ => '.' };
        return format!("{}{}{}", grouped, dec_sep, &frac_part[1..]);
    }
    s
}

pub fn format_date_locale(ts: i64, langid: &str) -> String {
    let dt: DateTime<Local> = Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now);
    let code = lang_code(langid);
    match code.as_str() {
        "ja" => dt.format("%Y/%m/%d").to_string(),
        "de" => dt.format("%d.%m.%Y").to_string(),
        "fr" | "es" | "it" | "pt" => dt.format("%d/%m/%Y").to_string(),
        _ => dt.format("%Y-%m-%d").to_string(),
    }
}

/// Format local datetime according to locale (lightweight patterns)
pub fn format_datetime_locale(ts: i64, langid: &str) -> String {
    let dt: DateTime<Local> = Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now);
    let code = lang_code(langid);
    match code.as_str() {
        // Include seconds for Japanese (common convention), minutes for others
        "ja" => dt.format("%Y/%m/%d %H:%M:%S").to_string(),
        "de" => dt.format("%d.%m.%Y %H:%M").to_string(),
        "fr" | "es" | "it" | "pt" => dt.format("%d/%m/%Y %H:%M").to_string(),
        _ => dt.format("%Y-%m-%d %H:%M").to_string(),
    }
}

pub fn format_size_locale(bytes: u64, langid: &str) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut idx = 0usize;
    while value >= 1024.0 && idx < UNITS.len() - 1 { value /= 1024.0; idx += 1; }
    let num = format_float_locale(value, 1, langid);
    format!("{} {}", num, UNITS[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_integer_locale() {
        assert_eq!(format_integer_locale(1234567, "en-US"), "1,234,567");
        assert_eq!(format_integer_locale(1234567, "de-DE"), "1.234.567");
    }

    #[test]
    fn test_format_float_locale() {
        let en = format_float_locale(12345.75, 2, "en-US");
        assert!(en.contains(","));
        assert!(en.contains("."));
        let de = format_float_locale(12345.75, 2, "de-DE");
        assert!(de.contains("."));
        assert!(de.contains(","));
    }

    #[test]
    fn test_format_date_locale() {
        let ts = Local.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap().timestamp();
        assert_eq!(format_date_locale(ts, "ja-JP"), "2023/01/02");
        assert_eq!(format_date_locale(ts, "de-DE"), "02.01.2023");
        assert_eq!(format_date_locale(ts, "en-US"), "2023-01-02");
    }

    #[test]
    fn test_format_size_locale() {
        let s_en = format_size_locale(1_572_864, "en-US");
        assert!(s_en.contains("MB"));
        let s_fr = format_size_locale(1_572_864, "fr-FR");
        assert!(s_fr.contains("MB"));
    }
}


