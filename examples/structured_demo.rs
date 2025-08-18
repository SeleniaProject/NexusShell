//! Structured Data Demo for NexusShell
//! Demonstrates NexusShell-inspired data processing capabilities

use nxsh_builtins::{demo_table_cli, sys_cli, json_select_cli};

fn main() -> anyhow::Result<()> {
    println!("🚀 NexusShell Structured Data Demo");
    println!("=====================================\n");

    // Demo structured data table
    println!("📊 Table Demo:");
    if let Err(e) = demo_table_cli(&[]) {
        println!("Demo table error: {}", e);
    }
    
    println!("\n🖥️  System Information:");
    if let Err(e) = sys_cli(&[]) {
        println!("System info error: {}", e);
    }
    
    println!("\n🔍 Select Demo (name and salary columns):");
    if let Err(e) = json_select_cli(&["name".to_string(), "salary".to_string()]) {
        println!("Select demo error: {}", e);
    }
    
    println!("\n✨ NexusShell now supports NexusShell-like structured data!");

    Ok(())
}
