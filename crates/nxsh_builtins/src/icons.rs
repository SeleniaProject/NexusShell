pub fn icon_for(path: &std::path::Path, is_dir: bool) -> &'static str {
    if is_dir {
        return "📁";
    }
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" => "🦀",
        "md" => "📄",
        "toml" => "⚙️",
        "png" | "jpg" | "jpeg" | "gif" => "🖼️",
        "zip" | "tar" | "gz" => "📦",
        _ => "📄",
    }
} 