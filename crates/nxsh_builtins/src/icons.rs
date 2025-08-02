use std::path::Path;

/// Get an icon for a file based on its extension
pub fn get_file_icon(path: &Path) -> &'static str {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" => "🖼️",
            "mp4" | "avi" | "mov" | "mkv" => "🎬",
            "mp3" | "wav" | "flac" => "🎵",
            "pdf" => "📄",
            "txt" | "md" => "📝",
            "rs" => "🦀",
            "js" | "ts" => "💛",
            "py" => "🐍",
            "html" | "htm" => "🌐",
            "css" => "🎨",
            "json" => "📋",
            "xml" => "📰",
            "zip" | "tar" | "gz" => "📦",
            "exe" | "msi" => "⚙️",
            "dll" => "🔧",
            _ => "📄",
        }
    } else if path.is_dir() {
        "📁"
    } else {
        "📄"
    }
}

/// Get an icon for a directory
pub fn get_directory_icon() -> &'static str {
    "📁"
}

/// Get an icon for a file type
pub fn get_type_icon(file_type: &str) -> &'static str {
    match file_type {
        "directory" => "📁",
        "file" => "📄",
        "symlink" => "🔗",
        "socket" => "🔌",
        "pipe" => "🔀",
        "block_device" => "💽",
        "char_device" => "⌨️",
        _ => "❓",
    }
}

/// Get an icon for file permissions
pub fn get_permission_icon(is_readable: bool, is_writable: bool, is_executable: bool) -> String {
    let mut icon = String::new();
    
    if is_readable {
        icon.push('👁');
    }
    if is_writable {
        icon.push('✏');
    }
    if is_executable {
        icon.push('⚡');
    }
    
    if icon.is_empty() {
        icon.push('🚫');
    }
    
    icon
}

/// Get status icon for commands
pub fn get_status_icon(success: bool) -> &'static str {
    if success {
        "✅"
    } else {
        "❌"
    }
}

/// Get process status icon
pub fn get_process_icon(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "running" => "🏃",
        "sleeping" => "😴",
        "stopped" => "⛔",
        "zombie" => "🧟",
        "waiting" => "⏳",
        _ => "❓",
    }
}
