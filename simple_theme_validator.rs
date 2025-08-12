use std::fs;
use std::path::Path;

// シンプルなテーマ検証（外部依存なし）
fn main() {
    let themes_dir = Path::new("assets/themes");
    
    println!("テーマファイルの基本検証を開始...");
    
    if !themes_dir.exists() {
        println!("❌ assets/themes ディレクトリが存在しません");
        return;
    }
    
    let mut total_themes = 0;
    let mut valid_themes = 0;
    let mut warnings = 0;
    
    // テーマファイルを列挙して基本検証
    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map(|s| s == "json").unwrap_or(false) {
                    total_themes += 1;
                    println!("\n📄 検証中: {}", path.file_name().unwrap().to_string_lossy());
                    
                    match validate_theme_file(&path) {
                        Ok(warning_count) => {
                            valid_themes += 1;
                            if warning_count > 0 {
                                warnings += warning_count;
                                println!("✅ 有効（警告 {}個）", warning_count);
                            } else {
                                println!("✅ 完全に有効");
                            }
                        }
                        Err(e) => {
                            println!("❌ 無効: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    // スキーマファイルの検証
    let schema_path = themes_dir.join("theme_schema.json");
    if schema_path.exists() {
        println!("\n📋 スキーマファイル検証中...");
        match validate_theme_file(&schema_path) {
            Ok(_) => println!("✅ スキーマファイル有効"),
            Err(e) => println!("❌ スキーマファイル無効: {}", e),
        }
    }
    
    println!("\n=== 検証結果サマリー ===");
    println!("総テーマ数: {}", total_themes);
    println!("有効テーマ数: {}", valid_themes);
    println!("無効テーマ数: {}", total_themes - valid_themes);
    println!("総警告数: {}", warnings);
    
    if valid_themes == total_themes {
        println!("🎉 すべてのテーマが有効です！");
    } else {
        println!("⚠️  いくつかのテーマに問題があります");
    }
}

fn validate_theme_file(path: &Path) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("ファイル読み込みエラー: {}", e))?;
    
    // 基本的なJSON解析
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON解析エラー: {}", e))?;
    
    let mut warning_count = 0;
    
    // 基本フィールドの存在確認
    let required_fields = ["name", "version", "colors"];
    for field in &required_fields {
        if !json.get(field).is_some() {
            return Err(format!("必須フィールド '{}' が見つかりません", field));
        }
    }
    
    // nameフィールドの確認
    if let Some(name) = json.get("name") {
        if !name.is_string() || name.as_str().unwrap().is_empty() {
            return Err("nameフィールドは空でない文字列である必要があります".to_string());
        }
    }
    
    // versionフィールドの確認
    if let Some(version) = json.get("version") {
        if !version.is_string() {
            return Err("versionフィールドは文字列である必要があります".to_string());
        }
        let version_str = version.as_str().unwrap();
        if !is_valid_semver(version_str) {
            warning_count += 1;
            println!("  ⚠️  バージョン形式が非標準: {}", version_str);
        }
    }
    
    // colorsフィールドの確認
    if let Some(colors) = json.get("colors") {
        if !colors.is_object() {
            return Err("colorsフィールドはオブジェクトである必要があります".to_string());
        }
        
        // 基本色の確認
        let basic_colors = ["primary", "background", "foreground"];
        for color in &basic_colors {
            if let Some(color_value) = colors.get(color) {
                if let Some(color_str) = color_value.as_str() {
                    if !is_valid_hex_color(color_str) {
                        return Err(format!("無効な色形式 '{}': {}", color, color_str));
                    }
                } else {
                    warning_count += 1;
                    println!("  ⚠️  色 '{}' が文字列ではありません", color);
                }
            }
        }
    }
    
    Ok(warning_count)
}

fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    
    for part in parts {
        if part.parse::<u32>().is_err() {
            return false;
        }
    }
    true
}

fn is_valid_hex_color(color: &str) -> bool {
    if !color.starts_with('#') {
        return false;
    }
    
    let hex_part = &color[1..];
    if hex_part.len() != 6 && hex_part.len() != 3 {
        return false;
    }
    
    hex_part.chars().all(|c| c.is_ascii_hexdigit())
}
