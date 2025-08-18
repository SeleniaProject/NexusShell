//! 高性能補完統合ヘルパー
//! 
//! このモジュールは、高性能補完エンジンをrustylineと統合するためのヘルパーを提供します。

use anyhow::Result;
use rustyline::{
    completion::{Completer, Pair},
    Context as RustylineContext,
};
use std::sync::{Arc, Mutex};
use crate::{
    completion::NexusCompleter,
    completion_engine::{AdvancedCompletionEngine, CompletionResult},
};
use tokio::runtime::Runtime;

/// 高性能補完を統合するRustylineヘルパー
pub struct FastCompletionHelper {
    /// 高性能エンジン（優先）
    advanced_engine: Option<Arc<AdvancedCompletionEngine>>,
    /// フォールバック用基本補完
    basic_completer: Arc<Mutex<NexusCompleter>>,
    /// 非同期ランタイム
    runtime: Runtime,
    /// パフォーマンス統計
    stats: Arc<Mutex<CompletionStats>>,
}

impl FastCompletionHelper {
    /// 新しいヘルパーを作成
    pub fn new() -> Result<Self> {
        let basic_completer = Arc::new(Mutex::new(NexusCompleter::new()?));
        let advanced_engine = match AdvancedCompletionEngine::new() {
            Ok(engine) => {
                println!("🚀 高性能補完エンジンが有効化されました");
                Some(Arc::new(engine))
            }
            Err(e) => {
                eprintln!("⚠️  高性能エンジンの初期化に失敗、基本モードで動作: {}", e);
                None
            }
        };

        Ok(Self {
            advanced_engine,
            basic_completer,
            runtime: Runtime::new()?,
            stats: Arc::new(Mutex::new(CompletionStats::new())),
        })
    }

    /// 補完の統計情報を取得
    pub fn get_stats(&self) -> CompletionStats {
        self.stats.lock().unwrap().clone()
    }

    /// 統計情報をリセット
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = CompletionStats::new();
        }
    }
}

impl Completer for FastCompletionHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustylineContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start_time = std::time::Instant::now();
        
        // 高性能エンジンを優先的に使用
        if let Some(ref engine) = self.advanced_engine {
            match self.runtime.block_on(engine.get_completions(line, pos)) {
                Ok(result) => {
                    let elapsed = start_time.elapsed();
                    
                    // 統計更新
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.record_completion(elapsed, result.candidates.len(), true);
                    }

                    // 結果をRustyline形式に変換
                    let pairs: Vec<Pair> = result.candidates
                        .into_iter()
                        .map(|candidate| Pair {
                            display: format!("{} - {}", candidate.text, candidate.description),
                            replacement: candidate.text,
                        })
                        .collect();

                    // 単語の開始位置を計算
                    let word_start = line[..pos].rfind(|c: char| c.is_whitespace() || ";&|<>()".contains(c))
                        .map(|i| i + 1)
                        .unwrap_or(0);

                    return Ok((word_start, pairs));
                }
                Err(e) => {
                    eprintln!("高性能エンジンエラー、フォールバック: {}", e);
                }
            }
        }

        // フォールバック：基本補完を使用
        if let Ok(completer) = self.basic_completer.lock() {
            match completer.complete(line, pos, _ctx) {
                Ok((start_pos, pairs)) => {
                    let elapsed = start_time.elapsed();
                    
                    // 統計更新
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.record_completion(elapsed, pairs.len(), false);
                    }

                    Ok((start_pos, pairs))
                }
                Err(e) => Err(e),
            }
        } else {
            // 最後のフォールバック：空の結果
            Ok((pos, Vec::new()))
        }
    }
}

impl FastCompletionHelper {
    /// 高性能補完APIのためのヘルパーメソッド（ベンチマーク用）
    pub fn get_completions(&self, line: &str, pos: usize) -> anyhow::Result<Vec<String>> {
        // デフォルトのRustylineコンテキストを作成
        use rustyline::history::DefaultHistory;
        let history = DefaultHistory::new();
        let context = rustyline::Context::new(&history);
        
        // 内部の補完メソッドを呼び出す
        match self.complete(line, pos, &context) {
            Ok((_, pairs)) => {
                let results = pairs.into_iter()
                    .map(|pair| pair.replacement)
                    .collect();
                Ok(results)
            }
            Err(e) => Err(anyhow::anyhow!("補完エラー: {:?}", e))
        }
    }

    /// パフォーマンス統計を取得
    pub fn get_performance_stats(&self) -> Option<PerformanceStats> {
        self.stats.lock().ok().map(|stats| PerformanceStats {
            total_completions: stats.total_completions,
            avg_response_time_us: stats.average_latency_ms * 1000.0,
            cache_hit_rate: if stats.total_completions > 0 {
                stats.advanced_engine_used as f64 / stats.total_completions as f64
            } else {
                0.0
            },
        })
    }
}

impl rustyline::Helper for FastCompletionHelper {}
impl rustyline::highlight::Highlighter for FastCompletionHelper {}
impl rustyline::hint::Hinter for FastCompletionHelper {
    type Hint = String;
}
impl rustyline::validate::Validator for FastCompletionHelper {}

/// 補完パフォーマンス統計
#[derive(Debug, Clone)]
pub struct CompletionStats {
    pub total_completions: u64,
    pub advanced_engine_used: u64,
    pub basic_completer_used: u64,
    pub average_latency_ms: f64,
    pub max_latency_ms: f64,
    pub min_latency_ms: f64,
    pub total_candidates: u64,
}

impl CompletionStats {
    pub fn new() -> Self {
        Self {
            total_completions: 0,
            advanced_engine_used: 0,
            basic_completer_used: 0,
            average_latency_ms: 0.0,
            max_latency_ms: 0.0,
            min_latency_ms: f64::INFINITY,
            total_candidates: 0,
        }
    }

    pub fn record_completion(&mut self, elapsed: std::time::Duration, candidates: usize, used_advanced: bool) {
        let latency_ms = elapsed.as_secs_f64() * 1000.0;
        
        self.total_completions += 1;
        if used_advanced {
            self.advanced_engine_used += 1;
        } else {
            self.basic_completer_used += 1;
        }
        
        self.total_candidates += candidates as u64;
        
        // レイテンシ統計更新
        self.max_latency_ms = self.max_latency_ms.max(latency_ms);
        self.min_latency_ms = self.min_latency_ms.min(latency_ms);
        
        // 平均レイテンシ更新（指数移動平均）
        if self.total_completions == 1 {
            self.average_latency_ms = latency_ms;
        } else {
            self.average_latency_ms = 0.9 * self.average_latency_ms + 0.1 * latency_ms;
        }
    }

    pub fn print_summary(&self) {
        println!("\n📊 補完パフォーマンス統計:");
        println!("  総補完回数: {}", self.total_completions);
        println!("  高性能エンジン使用: {} ({:.1}%)", 
                 self.advanced_engine_used, 
                 (self.advanced_engine_used as f64 / self.total_completions as f64) * 100.0);
        println!("  基本補完使用: {} ({:.1}%)", 
                 self.basic_completer_used,
                 (self.basic_completer_used as f64 / self.total_completions as f64) * 100.0);
        println!("  平均レイテンシ: {:.3}ms", self.average_latency_ms);
        println!("  最小レイテンシ: {:.3}ms", self.min_latency_ms);
        println!("  最大レイテンシ: {:.3}ms", self.max_latency_ms);
        println!("  平均候補数: {:.1}", self.total_candidates as f64 / self.total_completions as f64);
        
        // パフォーマンス評価
        if self.average_latency_ms < 1.0 {
            println!("  ✅ 目標レイテンシ(1ms)を達成");
        } else {
            println!("  ⚠️  目標レイテンシ(1ms)を超過");
        }
    }
}

/// 補完品質メトリクス
#[derive(Debug, Clone)]
pub struct CompletionQualityMetrics {
    pub relevance_score: f64,
    pub diversity_score: f64,
    pub freshness_score: f64,
}

impl CompletionQualityMetrics {
    pub fn calculate(result: &CompletionResult, user_input: &str) -> Self {
        let relevance_score = Self::calculate_relevance(&result.candidates, user_input);
        let diversity_score = Self::calculate_diversity(&result.candidates);
        let freshness_score = Self::calculate_freshness(&result.candidates);
        
        Self {
            relevance_score,
            diversity_score,
            freshness_score,
        }
    }

    fn calculate_relevance(candidates: &[crate::completion_engine::CompletionCandidate], user_input: &str) -> f64 {
        if candidates.is_empty() {
            return 0.0;
        }

        let relevant_count = candidates.iter()
            .filter(|c| c.text.to_lowercase().contains(&user_input.to_lowercase()))
            .count();
        
        relevant_count as f64 / candidates.len() as f64
    }

    fn calculate_diversity(candidates: &[crate::completion_engine::CompletionCandidate]) -> f64 {
        use std::collections::HashSet;
        
        if candidates.is_empty() {
            return 0.0;
        }

        let types: HashSet<_> = candidates.iter()
            .map(|c| std::mem::discriminant(&c.candidate_type))
            .collect();
        
        types.len() as f64 / 6.0 // 6は候補タイプの総数
    }

    fn calculate_freshness(candidates: &[crate::completion_engine::CompletionCandidate]) -> f64 {
        // 簡略化された実装：スマート提案の割合を計算
        if candidates.is_empty() {
            return 0.0;
        }

        let smart_count = candidates.iter()
            .filter(|c| matches!(c.candidate_type, crate::completion_engine::CandidateType::SmartSuggestion))
            .count();
        
        smart_count as f64 / candidates.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_creation() {
        let helper = FastCompletionHelper::new();
        assert!(helper.is_ok(), "ヘルパーの作成に失敗");
    }

    #[test]
    fn test_stats_recording() {
        let mut stats = CompletionStats::new();
        let elapsed = std::time::Duration::from_millis(5);
        
        stats.record_completion(elapsed, 10, true);
        
        assert_eq!(stats.total_completions, 1);
        assert_eq!(stats.advanced_engine_used, 1);
        assert_eq!(stats.total_candidates, 10);
        assert_eq!(stats.average_latency_ms, 5.0);
    }

    #[test]
    fn test_multiple_completions() {
        let mut stats = CompletionStats::new();
        
        // 複数の補完を記録
        stats.record_completion(std::time::Duration::from_millis(2), 5, true);
        stats.record_completion(std::time::Duration::from_millis(8), 15, false);
        
        assert_eq!(stats.total_completions, 2);
        assert_eq!(stats.advanced_engine_used, 1);
        assert_eq!(stats.basic_completer_used, 1);
        assert_eq!(stats.max_latency_ms, 8.0);
        assert_eq!(stats.min_latency_ms, 2.0);
    }
}

/// パフォーマンス統計構造体（外部API用）
pub struct PerformanceStats {
    pub total_completions: u64,
    pub avg_response_time_us: f64,
    pub cache_hit_rate: f64,
}
