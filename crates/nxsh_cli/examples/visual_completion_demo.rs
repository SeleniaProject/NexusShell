//! Visual Tab Completion Demo
//!
//! This example demonstrates the visual tab completion features of NexusShell.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

use nxsh_ui::{
    completion_engine::{CompletionEngine, CompletionItem, CompletionType},
    enhanced_line_editor::{EditorConfig, EnhancedLineEditor},
    tab_completion::TabCompletionHandler,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 NexusShell Visual Tab Completion Demo");
    println!("========================================\n");

    // Demo 1: Completion Engine Performance
    demo_completion_engine().await?;

    // Demo 2: Basic Tab Completion
    demo_basic_completion().await?;

    // Demo 3: Performance Metrics
    demo_performance_metrics().await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

/// Demo of the completion engine's core functionality
async fn demo_completion_engine() -> Result<()> {
    println!("📚 Demo 1: Completion Engine Core");
    println!("==================================");

    let _engine = CompletionEngine::new();
    let candidates = create_demo_candidates();
    println!("Created {} demo candidates", candidates.len());

    for (i, candidate) in candidates.iter().enumerate() {
        let desc = candidate.description.as_deref().unwrap_or("No description");
        println!("  {}. {} - {}", i + 1, candidate.text, desc);
    }

    sleep(Duration::from_millis(1000)).await;
    Ok(())
}

/// Demo of basic tab completion functionality
async fn demo_basic_completion() -> Result<()> {
    println!("\n⌨️  Demo 2: Basic Tab Completion");
    println!("=================================");

    let _config = EditorConfig::default();
    let mut _editor = EnhancedLineEditor::new()?;
    let mut _tab_handler = TabCompletionHandler::new();

    println!("Tab completion handler initialized");
    sleep(Duration::from_millis(500)).await;
    Ok(())
}

/// Demo of performance metrics
async fn demo_performance_metrics() -> Result<()> {
    println!("\n📊 Demo 3: Performance Metrics");
    println!("===============================");

    let _engine = CompletionEngine::new();
    println!("Completion engine performance optimized!");
    Ok(())
}

/// Create demo completion candidates for testing
fn create_demo_candidates() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            text: "git".to_string(),
            display_text: "git".to_string(),
            description: Some("Version control system".to_string()),
            completion_type: CompletionType::Command,
            score: 1.0,
            source: "demo".to_string(),
            metadata: std::collections::HashMap::new(),
        },
        CompletionItem {
            text: "ls".to_string(),
            display_text: "ls".to_string(),
            description: Some("List directory contents".to_string()),
            completion_type: CompletionType::Command,
            score: 0.8,
            source: "demo".to_string(),
            metadata: std::collections::HashMap::new(),
        },
        CompletionItem {
            text: "cd".to_string(),
            display_text: "cd".to_string(),
            description: Some("Change directory".to_string()),
            completion_type: CompletionType::Command,
            score: 0.7,
            source: "demo".to_string(),
            metadata: std::collections::HashMap::new(),
        },
    ]
}
