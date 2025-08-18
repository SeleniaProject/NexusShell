//! 高性能タブ補完デモとベンチマーク
//! 
//! このプログラムは、NexusShellの新しい高性能タブ補完機能をデモンストレーションし、
//! パフォーマンステストを実行します。

use anyhow::Result;
use nxsh_ui::completion_engine::{AdvancedCompletionEngine, CompletionContext};
use nxsh_ui::completion::{NexusCompleter, CompletionConfig};
use nxsh_ui::completion_metrics::{CompletionTimer, measure_completion};
use std::time::Instant;
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 NexusShell 高性能タブ補完システム デモ");
    println!("=" .repeat(60));

    // 1. 基本的な補完エンジンのテスト
    println!("\n📋 基本補完エンジンテスト");
    test_basic_completer().await?;

    // 2. 高性能エンジンのテスト
    println!("\n⚡ 高性能補完エンジンテスト");
    test_advanced_engine().await?;

    // 3. パフォーマンス比較
    println!("\n📊 パフォーマンス比較");
    performance_comparison().await?;

    // 4. 実用的なシナリオテスト
    println!("\n🎯 実用シナリオテスト");
    practical_scenarios().await?;

    // 5. スマート提案のデモ
    println!("\n🧠 スマート提案デモ");
    smart_suggestions_demo().await?;

    println!("\n✅ デモ完了！");
    Ok(())
}

async fn test_basic_completer() -> Result<()> {
    let completer = NexusCompleter::new()?;
    
    let test_cases = vec![
        "ls",
        "git",
        "cargo",
        "cd /",
        "$PA",
        "grep -",
    ];

    for input in test_cases {
        let (completions, elapsed_ms) = measure_completion(|| {
            // 同期版を使用（非async環境のため）
            match Runtime::new() {
                Ok(rt) => rt.block_on(completer.get_completions(input)).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        });

        println!("  入力: '{}' → {} 候補 ({:.2}ms)", 
                 input, completions.len(), elapsed_ms);
        
        if !completions.is_empty() {
            println!("    例: {}", completions.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
        }
    }

    Ok(())
}

async fn test_advanced_engine() -> Result<()> {
    let engine = AdvancedCompletionEngine::new()?;
    
    let test_cases = vec![
        ("ls", "ファイル一覧"),
        ("git comm", "Git コミット"),
        ("cargo bu", "Cargo ビルド"),
        ("cd ~/Doc", "ディレクトリ移動"),
        ("$PATH", "環境変数"),
        ("docker ru", "Docker実行"),
        ("npm in", "NPMインストール"),
    ];

    for (input, description) in test_cases {
        let start_time = Instant::now();
        
        match engine.get_completions(input, input.len()).await {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                println!("  {} ('{}') → {} 候補 ({:.3}ms)", 
                         description, input, result.candidates.len(), elapsed.as_secs_f64() * 1000.0);
                
                // トップ候補を表示
                for (i, candidate) in result.candidates.iter().take(3).enumerate() {
                    println!("    {}. {} - {}", i + 1, candidate.text, candidate.description);
                }
                
                if elapsed.as_millis() > 1 {
                    println!("    ⚠️  目標レスポンス時間(1ms)を超過");
                }
            }
            Err(e) => {
                println!("  {} ('{}') → エラー: {}", description, input, e);
            }
        }
    }

    Ok(())
}

async fn performance_comparison() -> Result<()> {
    let completer = NexusCompleter::new()?;
    let engine = AdvancedCompletionEngine::new()?;
    
    let test_input = "git";
    let iterations = 100;
    
    // 基本補完のベンチマーク
    let start_time = Instant::now();
    for _ in 0..iterations {
        let _ = Runtime::new().unwrap().block_on(completer.get_completions(test_input));
    }
    let basic_avg = start_time.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    
    // 高性能エンジンのベンチマーク
    let start_time = Instant::now();
    for _ in 0..iterations {
        let _ = engine.get_completions(test_input, test_input.len()).await;
    }
    let advanced_avg = start_time.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    
    println!("  基本補完: {:.3}ms/回 (平均)", basic_avg);
    println!("  高性能エンジン: {:.3}ms/回 (平均)", advanced_avg);
    
    if advanced_avg < basic_avg {
        let improvement = ((basic_avg - advanced_avg) / basic_avg) * 100.0;
        println!("  🎉 性能改善: {:.1}%高速化", improvement);
    } else {
        let degradation = ((advanced_avg - basic_avg) / basic_avg) * 100.0;
        println!("  ⚠️  性能低下: {:.1}%遅く", degradation);
    }

    Ok(())
}

async fn practical_scenarios() -> Result<()> {
    let engine = AdvancedCompletionEngine::new()?;
    
    let scenarios = vec![
        ("長いファイルパス", "cd /usr/local/bin/very/long/path/to/som"),
        ("複雑なGitコマンド", "git log --oneline --graph --decorate --all --since="),
        ("Dockerコンテナ操作", "docker exec -it container_name /bin/"),
        ("Rustプロジェクト", "cargo build --release --target="),
        ("複数の環境変数", "$HOME/$USER/$SHELL"),
        ("パイプライン", "ls -la | grep -i pattern | head -"),
    ];

    for (scenario, input) in scenarios {
        let start_time = Instant::now();
        
        match engine.get_completions(input, input.len()).await {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                println!("  {}: {} 候補 ({:.3}ms)", 
                         scenario, result.candidates.len(), elapsed.as_secs_f64() * 1000.0);
                
                // 品質チェック
                let has_relevant = result.candidates.iter().any(|c| 
                    c.text.to_lowercase().contains(&input.split_whitespace().last().unwrap_or("").to_lowercase())
                );
                
                if has_relevant {
                    println!("    ✅ 関連性のある候補を発見");
                } else {
                    println!("    ⚠️  関連性のある候補が不足");
                }
            }
            Err(e) => {
                println!("  {}: エラー - {}", scenario, e);
            }
        }
    }

    Ok(())
}

async fn smart_suggestions_demo() -> Result<()> {
    let engine = AdvancedCompletionEngine::new()?;
    
    println!("  Git操作の文脈認識:");
    let git_input = "git";
    if let Ok(result) = engine.get_completions(git_input, git_input.len()).await {
        for candidate in result.candidates.iter().take(5) {
            if candidate.text.starts_with("git") {
                println!("    → {}", candidate.text);
            }
        }
    }

    println!("\n  Docker操作の文脈認識:");
    let docker_input = "docker";
    if let Ok(result) = engine.get_completions(docker_input, docker_input.len()).await {
        for candidate in result.candidates.iter().take(5) {
            if candidate.text.starts_with("docker") {
                println!("    → {}", candidate.text);
            }
        }
    }

    println!("\n  Node.js/NPM操作の文脈認識:");
    let npm_input = "npm";
    if let Ok(result) = engine.get_completions(npm_input, npm_input.len()).await {
        for candidate in result.candidates.iter().take(5) {
            if candidate.text.starts_with("npm") {
                println!("    → {}", candidate.text);
            }
        }
    }

    println!("\n  Rust/Cargo操作の文脈認識:");
    let cargo_input = "cargo";
    if let Ok(result) = engine.get_completions(cargo_input, cargo_input.len()).await {
        for candidate in result.candidates.iter().take(5) {
            if candidate.text.starts_with("cargo") {
                println!("    → {}", candidate.text);
            }
        }
    }

    Ok(())
}

/// パフォーマンステスト専用のヘルパー関数
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_completion_latency() {
        let engine = AdvancedCompletionEngine::new().expect("エンジンの作成に失敗");
        let test_cases = vec!["ls", "git", "cargo", "docker"];
        
        for input in test_cases {
            let start = Instant::now();
            let _result = engine.get_completions(input, input.len()).await;
            let elapsed = start.elapsed();
            
            // 1ms未満の応答時間を目標
            assert!(elapsed.as_millis() <= 1, 
                    "補完レスポンス時間が目標を超過: {}ms (入力: '{}')", 
                    elapsed.as_millis(), input);
        }
    }

    #[tokio::test]
    async fn test_completion_quality() {
        let engine = AdvancedCompletionEngine::new().expect("エンジンの作成に失敗");
        
        // Git補完の品質テスト
        let result = engine.get_completions("git comm", 8).await.expect("補完取得に失敗");
        let has_commit = result.candidates.iter().any(|c| c.text.contains("commit"));
        assert!(has_commit, "git commitの候補が見つからない");
        
        // ファイル補完の品質テスト
        let result = engine.get_completions("ls /usr/", 7).await.expect("補完取得に失敗");
        assert!(!result.candidates.is_empty(), "ファイル補完の候補が空");
    }

    #[tokio::test]
    async fn test_large_directory_completion() {
        let engine = AdvancedCompletionEngine::new().expect("エンジンの作成に失敗");
        
        // 大きなディレクトリでのパフォーマンステスト
        let start = Instant::now();
        let _result = engine.get_completions("/usr/bin/", 9).await;
        let elapsed = start.elapsed();
        
        // 大きなディレクトリでも5ms以内の応答を目標
        assert!(elapsed.as_millis() <= 5, 
                "大きなディレクトリでの補完が遅すぎる: {}ms", elapsed.as_millis());
    }
}
