use std::{collections::HashMap, fs, path::PathBuf};
use crate::context::ShellContext;

fn detect_locale() -> String {
    // Prefer LC_ALL, then LANG
    if let Ok(lc_all) = std::env::var("LC_ALL") { if !lc_all.is_empty() { return lc_all; } }
    if let Ok(lang) = std::env::var("LANG") { if !lang.is_empty() { return lang; } }
    "en_US.UTF-8".to_string()
}

fn normalize_locale_tag(tag: &str) -> String {
    // Convert LANG like ja_JP.UTF-8 to BCP47-like ja-JP
    let base = tag.split('.').next().unwrap_or(tag).replace('_', "-");
    let base = if base.is_empty() { "en-US".to_string() } else { base };
    // Uppercase region part if present (own the strings to avoid temporaries)
    let mut parts: Vec<String> = base.split('-').map(|s| s.to_string()).collect();
    if parts.len() >= 2 { parts[1] = parts[1].to_ascii_uppercase(); }
    parts.join("-")
}

fn home_dir_fallback() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("HOME") { return Some(PathBuf::from(h)); }
    if cfg!(windows) {
        if let Ok(p) = std::env::var("USERPROFILE") { return Some(PathBuf::from(p)); }
    }
    None
}

fn read_alias_toml(path: &PathBuf) -> Option<HashMap<String, String>> {
    let content = fs::read_to_string(path).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;
    let mut map = HashMap::new();
    if let Some(tbl) = parsed.get("alias").and_then(|v| v.as_table()) {
        for (k, v) in tbl.iter() {
            if let Some(val) = v.as_str() { map.insert(k.clone(), val.to_string()); }
        }
        return Some(map);
    }
    None
}

fn built_in_aliases_for(locale: &str) -> Option<HashMap<String, String>> {
    match locale {
        "ja-JP" => {
            let s = include_str!("../../../assets/aliases/ja-JP.toml");
            toml::from_str::<toml::Value>(s).ok().and_then(|v| v.get("alias").and_then(|t| t.as_table()).map(|t| t.iter().map(|(k,v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()))
        }
        "zh-CN" => {
            let s = include_str!("../../../assets/aliases/zh-CN.toml");
            toml::from_str::<toml::Value>(s).ok().and_then(|v| v.get("alias").and_then(|t| t.as_table()).map(|t| t.iter().map(|(k,v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()))
        }
        "ru-RU" => {
            let s = include_str!("../../../assets/aliases/ru-RU.toml");
            toml::from_str::<toml::Value>(s).ok().and_then(|v| v.get("alias").and_then(|t| t.as_table()).map(|t| t.iter().map(|(k,v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()))
        }
        _ => None,
    }
}

/// Apply locale-based aliases into the shell context.
pub fn apply_locale_aliases(ctx: &ShellContext) {
    let tag = detect_locale();
    let norm = normalize_locale_tag(&tag);

    // 1) Explicit alias file via env
    if let Ok(path) = std::env::var("NXSH_ALIAS_FILE_LOCALE") {
        let p = PathBuf::from(path);
        if let Some(map) = read_alias_toml(&p) {
            for (k, v) in map { let _ = ctx.set_alias(k, v); }
            return;
        }
    }

    // 2) User config: ~/.nxsh/aliases/<locale>.toml
    if let Some(mut home) = home_dir_fallback() {
        home.push(".nxsh"); home.push("aliases");
    home.push(format!("{norm}.toml"));
        if let Some(map) = read_alias_toml(&home) {
            for (k, v) in map { let _ = ctx.set_alias(k, v); }
            return;
        }
    }

    // 3) Built-in assets
    if let Some(map) = built_in_aliases_for(&norm) {
        for (k, v) in map { let _ = ctx.set_alias(k, v); }
    }
}


