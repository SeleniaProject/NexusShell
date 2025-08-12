use nxsh_ui::theme_validator::{ThemeValidator, ValidationResult};
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("🎨 NexusShell テーマバリデータ");
    println!("==============================");

    let themes_dir = Path::new("../../assets/themes");
    let schema_path = themes_dir.join("theme-schema.json");
    
    if !themes_dir.exists() {
        println!("❌ themes ディレクトリが見つかりません: {:?}", themes_dir);
        return Ok(());
    }
    
    if !schema_path.exists() {
        println!("❌ スキーマファイルが見つかりません: {:?}", schema_path);
        return Ok(());
    }

    let validator = ThemeValidator::new()?;
    
    let mut total_themes = 0;
    let mut valid_themes = 0;
    let mut total_warnings = 0;
    let mut total_errors = 0;
    
    // テーマファイルを検索
    let entries = fs::read_dir(themes_dir)?;
    let mut theme_files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map(|ext| ext == "json").unwrap_or(false) &&
            entry.file_name().to_string_lossy() != "theme_schema.json"
        })
        .collect();
    
    theme_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    
    println!("検証中のテーマ数: {}", theme_files.len());
    println!();
    
    for entry in theme_files {
        let path = entry.path();
        let theme_name = path.file_stem().unwrap().to_string_lossy();
        total_themes += 1;
        
        print!("📄 {} ... ", theme_name);
        
        match validator.validate_theme_file(&path) {
            Ok(result) => {
                if result.is_valid() {
                    if result.has_warnings() {
                        println!("⚠️  有効（警告 {}個）", result.warnings.len());
                        valid_themes += 1;
                        total_warnings += result.warnings.len();
                        for warning in result.warnings {
                            println!("    ⚠️  {}", warning);
                        }
                    } else {
                        println!("✅ 完全に有効");
                        valid_themes += 1;
                    }
                } else {
                    println!("❌ 無効（エラー {}個）", result.errors.len());
                    total_errors += result.errors.len();
                    for error in result.errors {
                        println!("    ❌ {}", error);
                    }
                }
            }
            Err(e) => {
                println!("💥 検証失敗: {}", e);
                total_errors += 1;
            }
        }
    }
    
    // サマリー表示
    println!();
    println!("=== 検証結果サマリー ===");
    println!("総テーマ数: {}", total_themes);
    println!("有効テーマ数: {}", valid_themes);
    println!("無効テーマ数: {}", total_themes - valid_themes);
    println!("総警告数: {}", total_warnings);
    println!("総エラー数: {}", total_errors);
    
    let success_rate = if total_themes > 0 {
        (valid_themes as f64 / total_themes as f64) * 100.0
    } else {
        0.0
    };
    
    println!("成功率: {:.1}%", success_rate);
    
    if valid_themes == total_themes {
        println!("🎉 すべてのテーマが検証に合格しました！");
    } else if success_rate >= 80.0 {
        println!("✅ 多くのテーマが検証に合格しました");
    } else {
        println!("⚠️  いくつかのテーマに問題があります");
    }
    
    Ok(())
}
