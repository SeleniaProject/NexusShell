/*
 * NexusShell 高性能補完システム デモンストレーション
 * 
 * このデモは、超高速タブ補完システムの性能を測定し、
 * Nushellレベルの応答性を実現していることを確認します。
 */

use anyhow::Result;
use std::time::{Duration, Instant};
use nxsh_ui::completion_integration::FastCompletionHelper;

fn main() -> Result<()> {
    println!("🚀 NexusShell 高性能補完システム ベンチマーク");
    println!("===============================================");

    // 1. 補完エンジンの初期化
    let start = Instant::now();
    let completion_helper = FastCompletionHelper::new()?;
    let init_time = start.elapsed();
    println!("📦 補完エンジン初期化時間: {:?}", init_time);

    // 2. ウォームアップ（キャッシュ構築）
    println!("\n🔥 ウォームアップフェーズ...");
    let warm_start = Instant::now();
    let _warmup = completion_helper.get_completions("l", 1)?;
    let warm_time = warm_start.elapsed();
    println!("⏱️  ウォームアップ時間: {:?}", warm_time);

    // 3. パフォーマンステスト - ファイル補完
    println!("\n📁 ファイル補完パフォーマンステスト");
    let file_patterns = vec!["Cargo", "src/", "README", "target/"];
    let mut file_times = Vec::new();

    for pattern in &file_patterns {
        let start = Instant::now();
        let results = completion_helper.get_completions(pattern, pattern.len())?;
        let duration = start.elapsed();
        file_times.push(duration);
        
        println!("  '{}' -> {} 候補, {:?} ({:.2}μs)", 
                 pattern, results.len(), duration, duration.as_micros());
    }

    // 4. パフォーマンステスト - コマンド補完
    println!("\n⚡ コマンド補完パフォーマンステスト");
    let command_patterns = vec!["l", "ca", "git", "npm", "cargo"];
    let mut cmd_times = Vec::new();

    for pattern in &command_patterns {
        let start = Instant::now();
        let results = completion_helper.get_completions(pattern, pattern.len())?;
        let duration = start.elapsed();
        cmd_times.push(duration);
        
        println!("  '{}' -> {} 候補, {:?} ({:.2}μs)", 
                 pattern, results.len(), duration, duration.as_micros());
    }

    // 5. 大量補完テスト（ストレステスト）
    println!("\n🏋️  大量補完ストレステスト");
    let stress_start = Instant::now();
    let stress_iterations = 1000;
    
    for i in 0..stress_iterations {
        let pattern = if i % 4 == 0 { "l" } else if i % 3 == 0 { "git" } else if i % 2 == 0 { "src/" } else { "Cargo" };
        let _ = completion_helper.get_completions(pattern, pattern.len())?;
    }
    
    let stress_total = stress_start.elapsed();
    let avg_per_completion = stress_total / stress_iterations;
    
    println!("📊 {} 回の補完を {:?} で実行", stress_iterations, stress_total);
    println!("📈 平均補完時間: {:?} ({:.2}μs)", avg_per_completion, avg_per_completion.as_micros());

    // 6. 統計サマリー
    println!("\n📊 パフォーマンス統計");
    println!("=====================");
    
    let avg_file_time = file_times.iter().sum::<Duration>() / file_times.len() as u32;
    let avg_cmd_time = cmd_times.iter().sum::<Duration>() / cmd_times.len() as u32;
    
    println!("🗂️  平均ファイル補完時間: {:?} ({:.2}μs)", avg_file_time, avg_file_time.as_micros());
    println!("⚡ 平均コマンド補完時間: {:?} ({:.2}μs)", avg_cmd_time, avg_cmd_time.as_micros());
    
    // 7. 目標達成判定
    println!("\n🎯 目標達成判定");
    println!("================");
    
    let target_time = Duration::from_millis(1); // <1ms
    let file_pass = avg_file_time < target_time;
    let cmd_pass = avg_cmd_time < target_time;
    let stress_pass = avg_per_completion < target_time;
    
    println!("📁 ファイル補完 <1ms: {} {}", if file_pass { "✅" } else { "❌" }, 
             if file_pass { "PASS" } else { "FAIL" });
    println!("⚡ コマンド補完 <1ms: {} {}", if cmd_pass { "✅" } else { "❌" }, 
             if cmd_pass { "PASS" } else { "FAIL" });
    println!("🏋️  ストレス平均 <1ms: {} {}", if stress_pass { "✅" } else { "❌" }, 
             if stress_pass { "PASS" } else { "FAIL" });

    // 8. 最終判定
    let all_pass = file_pass && cmd_pass && stress_pass;
    println!("\n🏆 最終結果: {} {}", 
             if all_pass { "✅" } else { "❌" }, 
             if all_pass { "超高性能補完システム完成！" } else { "最適化が必要です" });

    if all_pass {
        println!("🎉 Nushellレベルのタブ補完性能を達成しました！");
        println!("⚡ 実際の体感では、ほぼ遅延なしでタブ補完が動作します");
    }

    // 9. メモリ使用量確認
    if let Some(stats) = completion_helper.get_performance_stats() {
        println!("\n💾 メモリ使用統計");
        println!("=================");
        println!("📊 合計補完数: {}", stats.total_completions);
        println!("📈 平均レスポンス時間: {:.2}μs", stats.avg_response_time_us);
        println!("💾 キャッシュヒット率: {:.1}%", stats.cache_hit_rate * 100.0);
    }

    Ok(())
}
