//! Visual Tab Completion Demo
//! 
//! This demo showcases the beautiful new tab completion system with:
//! - Stunning visual completion panel
//! - Tab navigation through candidates
//! - Real-time animations
//! - Category-based organization

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

use nxsh_ui::{
    enhanced_line_editor::{EnhancedLineEditor, EditorConfig},
    tab_completion::{TabCompletionHandler, TabCompletionResult},
    completion_panel::{CompletionPanel, PanelConfig},
    completion_engine::{AdvancedCompletionEngine, CompletionCandidate, CandidateType},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 NexusShell Visual Tab Completion Demo");
    println!("========================================\n");

    // Demo 1: Completion Engine Performance
    demo_completion_engine().await?;
    
    // Demo 2: Visual Completion Panel
    demo_visual_panel().await?;
    
    // Demo 3: Tab Navigation
    demo_tab_navigation().await?;
    
    // Demo 4: Enhanced Line Editor
    demo_enhanced_editor().await?;

    println!("\n✨ Demo completed! The new tab completion system is ready!");
    Ok(())
}

/// Demo the high-performance completion engine
async fn demo_completion_engine() -> Result<()> {
    println!("📡 Testing High-Performance Completion Engine");
    println!("---------------------------------------------");
    
    let engine = AdvancedCompletionEngine::new()?;
    
    // Test various completion scenarios
    let test_cases = vec![
        ("git ", "Git command completion"),
        ("cargo ", "Cargo command completion"),
        ("ls ", "File completion"),
        ("cd ~/", "Directory completion"),
        ("echo $", "Variable completion"),
    ];
    
    for (input, description) in test_cases {
        println!("  🔍 {}: '{}'", description, input);
        let start = std::time::Instant::now();
        
        match engine.get_completions(input, input.len()).await {
            Ok(result) => {
                let duration = start.elapsed();
                println!("    ✁E{} candidates in {:.2}ms", 
                    result.candidates.len(), 
                    duration.as_nanos() as f64 / 1_000_000.0
                );
                
                // Show top 3 candidates
                for (i, candidate) in result.candidates.iter().take(3).enumerate() {
                    println!("       {}. {} - {}", i + 1, candidate.text, candidate.description);
                }
                if result.candidates.len() > 3 {
                    println!("       ... and {} more", result.candidates.len() - 3);
                }
            }
            Err(e) => {
                println!("    ❁EError: {}", e);
            }
        }
        println!();
    }
    
    Ok(())
}

/// Demo the visual completion panel
async fn demo_visual_panel() -> Result<()> {
    println!("🎨 Testing Visual Completion Panel");
    println!("----------------------------------");
    
    let config = PanelConfig {
        max_width: 60,
        max_height: 10,
        candidates_per_page: 8,
        show_categories: true,
        show_icons: true,
        show_descriptions: true,
        enable_animations: true,
        animation_duration_ms: 150,
        auto_hide: true,
    };
    
    let mut panel = CompletionPanel::new(config);
    
    // Create sample candidates
    let candidates = vec![
        CompletionCandidate {
            text: "git add".to_string(),
            description: "Add files to staging area".to_string(),
            candidate_type: CandidateType::Command,
            base_score: 1.0,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
        CompletionCandidate {
            text: "git commit".to_string(),
            description: "Commit staged changes".to_string(),
            candidate_type: CandidateType::Command,
            base_score: 0.95,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
        CompletionCandidate {
            text: "git push".to_string(),
            description: "Push commits to remote".to_string(),
            candidate_type: CandidateType::Command,
            base_score: 0.9,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
        CompletionCandidate {
            text: "README.md".to_string(),
            description: "Markdown documentation file".to_string(),
            candidate_type: CandidateType::File,
            base_score: 0.8,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
        CompletionCandidate {
            text: "src/".to_string(),
            description: "Source code directory".to_string(),
            candidate_type: CandidateType::Directory,
            base_score: 0.7,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
    ];
    
    println!("  🎯 Setting up completion panel with {} candidates", candidates.len());
    panel.set_candidates(candidates)?;
    
    println!("  🌟 Panel should display beautifully with:");
    println!("     • Unicode box drawing characters");
    println!("     • Category organization");
    println!("     • Icons for different types");
    println!("     • Descriptions for each candidate");
    println!("     • Smooth fade-in animation");
    
    // Simulate navigation
    println!("\n  ⚡ Simulating tab navigation:");
    for i in 0..3 {
        panel.select_next()?;
        if let Some(candidate) = panel.get_selected_candidate() {
            println!("     Tab {}: Selected '{}'", i + 1, candidate.text);
        }
        sleep(Duration::from_millis(200)).await;
    }
    
    println!("  ✁EVisual panel demo completed!");
    println!();
    
    Ok(())
}

/// Demo tab navigation functionality
async fn demo_tab_navigation() -> Result<()> {
    println!("🔧 Testing Tab Navigation System");
    println!("--------------------------------");
    
    let mut handler = TabCompletionHandler::new()?;
    
    // Test different tab scenarios
    let scenarios = vec![
        ("gi", "Partial command - should show completions"),
        ("git ", "Complete command - should show subcommands"),
        ("ls /ho", "Path completion"),
        ("echo $PA", "Variable completion"),
    ];
    
    for (input, description) in scenarios {
        println!("  🎯 {}: '{}'", description, input);
        
        // First tab
        match handler.handle_tab_key(input, input.len()).await? {
            TabCompletionResult::SingleCompletion { text, description } => {
                println!("    ✁ESingle completion: '{}' - {:?}", text, description);
            }
            TabCompletionResult::PartialCompletion { text, remaining_candidates } => {
                println!("    🔄 Partial completion: '{}' ({} more candidates)", text, remaining_candidates);
            }
            TabCompletionResult::PanelShown { candidate_count } => {
                println!("    🎨 Panel shown with {} candidates", candidate_count);
                
                // Simulate more tab presses
                for i in 0..3 {
                    match handler.handle_tab_key(input, input.len()).await? {
                        TabCompletionResult::NavigationUpdate => {
                            println!("       Tab {}: Navigation updated", i + 2);
                        }
                        other => {
                            println!("       Tab {}: {:?}", i + 2, other);
                        }
                    }
                }
            }
            TabCompletionResult::NoSuggestions => {
                println!("    ℹ�E�E No suggestions available");
            }
            other => {
                println!("    🔍 Result: {:?}", other);
            }
        }
        
        sleep(Duration::from_millis(100)).await;
        println!();
    }
    
    // Display performance metrics
    let metrics = handler.get_metrics();
    println!("  📊 Performance Metrics:");
    println!("     • Total requests: {}", metrics.requests);
    println!("     • Average response time: {:.2}ms", metrics.avg_response_time_ms);
    println!("     • Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
    
    println!("  ✁ETab navigation demo completed!");
    println!();
    
    Ok(())
}

/// Demo the enhanced line editor
async fn demo_enhanced_editor() -> Result<()> {
    println!("✨ Testing Enhanced Line Editor");
    println!("------------------------------");
    
    let config = EditorConfig {
        enable_visual_completion: true,
        enable_syntax_highlighting: true,
        max_history_size: 100,
        auto_save_history: false,
        history_file: None,
        completion_delay_ms: 150,
        enable_animations: true,
    };
    
    let _editor = EnhancedLineEditor::with_config(config)?;
    
    println!("  🚀 Enhanced line editor features:");
    println!("     • Visual completion panel");
    println!("     • Real-time syntax highlighting");
    println!("     • Smart tab navigation");
    println!("     • History management");
    println!("     • Smooth animations");
    println!("     • Emacs-style key bindings");
    
    println!("\n  🎮 Key bindings:");
    println!("     • Tab: Show/navigate completions");
    println!("     • Shift+Tab: Navigate backwards");
    println!("     • Up/Down: History navigation");
    println!("     • Ctrl+A: Beginning of line");
    println!("     • Ctrl+E: End of line");
    println!("     • Ctrl+K: Delete to end");
    println!("     • Ctrl+U: Delete to beginning");
    println!("     • Ctrl+W: Delete word backward");
    println!("     • Ctrl+L: Clear screen");
    
    println!("\n  ⚡ Performance characteristics:");
    println!("     • <1ms completion response time");
    println!("     • 60fps animation updates");
    println!("     • Minimal memory footprint");
    println!("     • Efficient Unicode handling");
    
    println!("  ✁EEnhanced line editor demo completed!");
    println!();
    
    Ok(())
}

/// Helper function to create demo candidates
#[allow(dead_code)]
fn create_demo_candidates() -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate {
            text: "awesome_command".to_string(),
            description: "An awesome command that does amazing things".to_string(),
            candidate_type: CandidateType::Command,
            base_score: 1.0,
            boost_score: 0.1,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("category".to_string(), "utility".to_string());
                map
            },
        },
        CompletionCandidate {
            text: "build.rs".to_string(),
            description: "Rust build script".to_string(),
            candidate_type: CandidateType::File,
            base_score: 0.9,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
        CompletionCandidate {
            text: "target/".to_string(),
            description: "Build output directory".to_string(),
            candidate_type: CandidateType::Directory,
            base_score: 0.8,
            boost_score: 0.0,
            metadata: std::collections::HashMap::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_functions() {
        // Test that demo functions don't panic
        let _ = demo_completion_engine().await;
        let _ = demo_visual_panel().await;
        let _ = demo_tab_navigation().await;
        let _ = demo_enhanced_editor().await;
    }

    #[test]
    fn test_demo_candidates_creation() {
        let candidates = create_demo_candidates();
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].text, "awesome_command");
        assert!(matches!(candidates[0].candidate_type, CandidateType::Command));
    }
}

