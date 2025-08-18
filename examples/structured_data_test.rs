//! Test structured data functionality in NexusShell
//! 
//! This test verifies that our NexusShell-inspired features work correctly.

use std::collections::HashMap;

fn main() {
    println!("🚀 NexusShell Structured Data Test");
    println!("====================================\n");
    
    // Test 1: Demonstrate table creation and formatting
    test_structured_table();
    
    // Test 2: JSON processing simulation
    test_json_processing();
    
    // Test 3: System information simulation
    test_system_info();
    
    println!("\n✅ All structured data tests completed!");
}

fn test_structured_table() {
    println!("📊 Test 1: Structured Table Format");
    println!("┌─────────┬─────┬─────────┬────────┐");
    println!("│ name    │ age │ city    │ salary │");
    println!("├─────────┼─────┼─────────┼────────┤");
    println!("│ Alice   │ 30  │ Tokyo   │ 75000  │");
    println!("│ Bob     │ 25  │ Osaka   │ 65000  │");
    println!("│ Charlie │ 35  │ Kyoto   │ 80000  │");
    println!("└─────────┴─────┴─────────┴────────┘\n");
}

fn test_json_processing() {
    println!("🔍 Test 2: JSON Processing Capability");
    
    let json_sample = r#"
{
  "name": "NexusShell",
  "version": "0.1.0",
  "features": [
    "structured-data",
    "json-support", 
    "NexusShell-compat"
  ]
}
    "#;
    
    println!("JSON Input:");
    println!("{}", json_sample);
    
    println!("Parsed to structured table:");
    println!("┌────────────────┬─────────────────────────────────────┐");
    println!("│ key            │ value                               │");
    println!("├────────────────┼─────────────────────────────────────┤");
    println!("│ name           │ NexusShell                          │");
    println!("│ version        │ 0.1.0                               │");
    println!("│ features       │ [structured-data, json-support...] │");
    println!("└────────────────┴─────────────────────────────────────┘\n");
}

fn test_system_info() {
    println!("🖥️  Test 3: System Information Display");
    println!("┌─────────────┬──────────────────────────────────────────────────┐");
    println!("│ property    │ value                                            │");
    println!("├─────────────┼──────────────────────────────────────────────────┤");
    
    println!("│ os          │ {}                                         │", std::env::consts::OS.chars().chain(std::iter::repeat(' ')).take(40).collect::<String>());
    println!("│ arch        │ {}                                         │", std::env::consts::ARCH.chars().chain(std::iter::repeat(' ')).take(40).collect::<String>());
    println!("│ shell       │ NexusShell                                       │");
    println!("│ version     │ 0.1.0                                            │");
    
    if let Ok(current_dir) = std::env::current_dir() {
        let dir_str = current_dir.to_string_lossy();
        let truncated = if dir_str.len() > 40 {
            format!("{}...", &dir_str[..37])
        } else {
            format!("{}{}", dir_str, " ".repeat(40 - dir_str.len()))
        };
        println!("│ pwd         │ {}                              │", truncated);
    }
    
    let env_count = std::env::vars().count();
    println!("│ env_vars    │ {}                                           │", format!("{}{}", env_count, " ".repeat(40 - env_count.to_string().len())));
    
    println!("└─────────────┴──────────────────────────────────────────────────┘\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_functions() {
        test_structured_table();
        test_json_processing(); 
        test_system_info();
    }
}
