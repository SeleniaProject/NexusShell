use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use anyhow::Result;

/// Ultra-fast completion system optimized for <1ms target
#[derive(Debug)]
pub struct FastCompletionEngine {
    builtin_cache: Arc<Vec<&'static str>>,
    stats: Arc<RwLock<CompletionStats>>,
}

impl Default for FastCompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FastCompletionEngine {
    pub fn new() -> Self {
        Self {
            builtin_cache: Arc::new(BUILTIN_COMMANDS.to_vec()),
            stats: Arc::new(RwLock::new(CompletionStats::default())),
        }
    }

    /// Ultra-fast completion (target: <1ms)
    pub fn get_completions_fast(&self, input: &str) -> Result<Vec<FastCompletion>> {
        let start = Instant::now();
        
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut completions = Vec::with_capacity(10);
        
        // Only search built-in commands for maximum speed
        for &builtin in self.builtin_cache.iter() {
            if builtin.starts_with(input) {
                completions.push(FastCompletion {
                    text: builtin.to_string(),
                    score: if builtin == input { 100.0 } else { 50.0 },
                });
                
                if completions.len() >= 8 {  // Strict limit for speed
                    break;
                }
            }
        }
        
        // Sort by score (exact matches first)
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        let duration = start.elapsed();
        self.record_completion(duration, completions.len());
        
        Ok(completions)
    }

    pub fn stats(&self) -> CompletionStats {
        self.stats.read().unwrap().clone()
    }

    fn record_completion(&self, duration: Duration, count: usize) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_requests += 1;
            stats.total_time += duration;
            stats.total_completions += count;
            
            if duration < stats.fastest_completion || stats.fastest_completion == Duration::ZERO {
                stats.fastest_completion = duration;
            }
            
            if duration > stats.slowest_completion {
                stats.slowest_completion = duration;
            }
        }
    }
}

/// Minimal completion result for maximum speed
#[derive(Debug, Clone)]
pub struct FastCompletion {
    pub text: String,
    pub score: f64,
}

/// Pre-compiled builtin commands list
static BUILTIN_COMMANDS: &[&str] = &[
    "cd", "ls", "pwd", "echo", "cat", "grep", "find", "ps", "kill",
    "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown",
    "tar", "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz", "zip", "unzip",
    "zstd", "unzstd",
    "curl", "wget", "git", "ssh", "scp",
    "head", "tail", "sort", "uniq", "wc", "awk", "sed", "tr",
    "du", "df", "free", "top", "htop", "ping", "nc", "telnet",
];

/// Completion statistics (reused from original)
#[derive(Debug, Clone, Default)]
pub struct CompletionStats {
    pub total_requests: u64,
    pub total_completions: usize,
    pub total_time: Duration,
    pub fastest_completion: Duration,
    pub slowest_completion: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl CompletionStats {
    pub fn avg_completion_time(&self) -> Duration {
        if self.total_requests > 0 {
            self.total_time / self.total_requests as u32
        } else {
            Duration::ZERO
        }
    }

    pub fn performance_target_met(&self) -> bool {
        self.avg_completion_time() < Duration::from_millis(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_completion_engine() {
        let engine = FastCompletionEngine::new();
        
        let completions = engine.get_completions_fast("l").unwrap();
        assert!(!completions.is_empty());
        
        // Should contain "ls"
        let ls_completion = completions.iter().find(|c| c.text == "ls");
        assert!(ls_completion.is_some());
    }

    #[test]
    fn test_performance_target() {
        let engine = FastCompletionEngine::new();
        
        // Run multiple completions to get average
        for _ in 0..10 {
            let _ = engine.get_completions_fast("ls").unwrap();
        }
        
        let stats = engine.stats();
        assert!(stats.performance_target_met(), 
            "Average completion time: {:?}, should be < 1ms", stats.avg_completion_time());
    }

    #[test]
    fn test_exact_match_priority() {
        let engine = FastCompletionEngine::new();
        
        let completions = engine.get_completions_fast("ls").unwrap();
        assert!(!completions.is_empty());
        
        // Exact match should be first
        assert_eq!(completions[0].text, "ls");
        assert_eq!(completions[0].score, 100.0);
    }
}
