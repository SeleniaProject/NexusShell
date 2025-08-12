use nxsh_builtins::common::locale_format::{format_integer_locale, format_float_locale, format_date_locale, format_size_locale};

#[test]
fn test_integer_grouping_locales() {
    assert_eq!(format_integer_locale(1_234_567, "en-US"), "1,234,567");
    assert_eq!(format_integer_locale(1_234_567, "de-DE"), "1.234.567");
}

#[test]
fn test_float_decimal_separator() {
    let en = format_float_locale(1234.5, 1, "en-US");
    assert!(en.contains("."));
    let de = format_float_locale(1234.5, 1, "de-DE");
    assert!(de.contains(","));
}

#[test]
fn test_date_and_size() {
    // 2023-05-04
    use chrono::TimeZone;
    let ts = chrono::Local.with_ymd_and_hms(2023, 5, 4, 0, 0, 0).unwrap().timestamp();
    assert!(format_date_locale(ts, "ja-JP").contains("2023/05/04"));
    let sz = format_size_locale(1_000_000, "en-US");
    assert!(sz.contains("MB") || sz.contains("KB"));
}


