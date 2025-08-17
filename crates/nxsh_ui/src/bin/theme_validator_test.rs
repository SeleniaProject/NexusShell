use nxsh_ui::theme_validator::ThemeValidator;
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("🎨 NexusShell テーマバリデータ");
    println!("==============================");

    // CLI options
    let mut themes_dir: Option<PathBuf> = None;
    let mut out_format: Option<String> = None; // md|csv|json
    let mut out_path: Option<PathBuf> = None;
    let mut _min_contrast: Option<f64> = None; // not yet plumbed to validator rules, for future
    let mut strict: bool = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => themes_dir = Some(PathBuf::from(args.next().expect("--dir requires a path"))),
            "--out-format" => out_format = Some(args.next().expect("--out-format requires md|csv|json")),
            "--out" => out_path = Some(PathBuf::from(args.next().expect("--out requires a path"))),
            "--min-contrast" => {
                _min_contrast = Some(args.next().expect("--min-contrast requires a number").parse::<f64>().unwrap_or(4.5))
            }
            "--strict" => { strict = true; }
            _ => {}
        }
    }

    let themes_dir = themes_dir.unwrap_or_else(|| PathBuf::from("assets/themes"));
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
    let entries = fs::read_dir(&themes_dir)?;
    let mut theme_files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map(|ext| ext == "json").unwrap_or(false) &&
            entry.file_name().to_string_lossy() != "theme-schema.json"
        })
        .collect();
    
    theme_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    
    println!("検証中のテーマ数: {}", theme_files.len());
    println!();
    
    struct Row { name: String, valid: bool, warnings: usize, errors: usize }
    let mut rows: Vec<Row> = Vec::new();

    let mut invalid_themes = 0usize;
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
                        for warning in &result.warnings {
                            println!("    ⚠️  {}", warning);
                        }
                        rows.push(Row { name: theme_name.to_string(), valid: true, warnings: result.warnings.len(), errors: 0 });
                    } else {
                        println!("✅ 完全に有効");
                        valid_themes += 1;
                        rows.push(Row { name: theme_name.to_string(), valid: true, warnings: 0, errors: 0 });
                    }
                } else {
                    println!("❌ 無効（エラー {}個）", result.errors.len());
                    total_errors += result.errors.len();
                    for error in &result.errors {
                        println!("    ❌ {}", error);
                    }
                    rows.push(Row { name: theme_name.to_string(), valid: false, warnings: result.warnings.len(), errors: result.errors.len() });
                    invalid_themes += 1;
                }
            }
            Err(e) => {
                println!("💥 検証失敗: {}", e);
                total_errors += 1;
                rows.push(Row { name: theme_name.to_string(), valid: false, warnings: 0, errors: 1 });
                invalid_themes += 1;
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
    
    // Optional aggregated report
    if let (Some(fmt), Some(path)) = (out_format.as_deref(), out_path.as_ref()) {
        if let Some(parent) = path.parent() { if !parent.as_os_str().is_empty() { let _ = std::fs::create_dir_all(parent); } }
        match fmt {
            "md" => {
                let mut s = String::new();
                s.push_str("| Theme | Valid | Warnings | Errors |\n|---|---|---:|---:|\n");
                for r in &rows {
                    let v = if r.valid { "✅" } else { "❌" };
                    s.push_str(&format!("| {} | {} | {} | {} |\n", r.name, v, r.warnings, r.errors));
                }
                fs::write(path, s)?;
                println!("📝 Wrote Markdown report: {}", path.display());
            }
            "csv" => {
                let mut s = String::new();
                s.push_str("theme,valid,warnings,errors\n");
                for r in &rows {
                    s.push_str(&format!("{},{},{},{}\n", r.name, r.valid, r.warnings, r.errors));
                }
                fs::write(path, s)?;
                println!("📝 Wrote CSV report: {}", path.display());
            }
            "json" => {
                let mut arr = Vec::new();
                for r in &rows { arr.push(serde_json::json!({
                    "theme": r.name,
                    "valid": r.valid,
                    "warnings": r.warnings,
                    "errors": r.errors,
                })); }
                let doc = serde_json::json!({
                    "total": total_themes,
                    "valid": valid_themes,
                    "warnings": total_warnings,
                    "errors": total_errors,
                    "rows": arr,
                });
                fs::write(path, serde_json::to_string_pretty(&doc)?)?;
                println!("📝 Wrote JSON report: {}", path.display());
            }
            other => {
                println!("⚠️  Unknown --out-format '{}', skip writing report.", other);
            }
        }
    }

    if strict && invalid_themes > 0 { anyhow::bail!("{} themes invalid", invalid_themes); }
    Ok(())
}
