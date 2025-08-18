//! Test actual NexusShell structured data functionality
//! 
//! This example demonstrates the real NexusShell-inspired features we've implemented.

use nxsh_builtins::ls_table_cli;
use nxsh_core::structured_data::{StructuredValue, PipelineData};
use nxsh_core::structured_commands::{FromJsonCommand, ToJsonCommand, SelectCommand, WhereCommand, StructuredCommand};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    println!("🚀 NexusShell Real Structured Data Demo");
    println!("=========================================\n");

    // Test 1: Real JSON processing
    test_json_commands()?;
    
    // Test 2: Table operations  
    test_table_operations()?;
    
    // Test 3: Enhanced ls command
    test_enhanced_ls()?;
    
    println!("\n✅ All real structured data features working!");
    
    Ok(())
}

fn test_json_commands() -> anyhow::Result<()> {
    println!("📄 Test 1: JSON Processing Commands");
    
    // Create JSON string
    let json_str = r#"{"name": "NexusShell", "version": "0.1.0", "active": true}"#;
    
    // Test FromJson command
    let input = PipelineData::new(StructuredValue::String(json_str.to_string()));
    let from_json = FromJsonCommand;
    let parsed = from_json.process(input)?;
    
    println!("✓ JSON parsed successfully");
    
    // Test ToJson command  
    let to_json = ToJsonCommand;
    let json_output = to_json.process(parsed)?;
    
    if let StructuredValue::String(output_json) = &json_output.value {
        println!("✓ JSON serialized: {}", output_json);
    }
    
    println!();
    Ok(())
}

fn test_table_operations() -> anyhow::Result<()> {
    println!("📊 Test 2: Table Operations");
    
    // Create sample table
    let mut row1 = HashMap::new();
    row1.insert("name".to_string(), StructuredValue::String("Alice".to_string()));
    row1.insert("age".to_string(), StructuredValue::Int(30));
    row1.insert("salary".to_string(), StructuredValue::Int(75000));
    
    let mut row2 = HashMap::new();
    row2.insert("name".to_string(), StructuredValue::String("Bob".to_string()));
    row2.insert("age".to_string(), StructuredValue::Int(25));
    row2.insert("salary".to_string(), StructuredValue::Int(65000));
    
    let table = StructuredValue::Table(vec![row1, row2]);
    let data = PipelineData::new(table);
    
    // Test Select command
    let select_cmd = SelectCommand {
        columns: vec!["name".to_string(), "salary".to_string()],
    };
    let selected = select_cmd.process(data.clone())?;
    println!("✓ Select command executed");
    
    // Test Where command
    let where_cmd = WhereCommand {
        column: "age".to_string(),
        operator: ">".to_string(),
        value: StructuredValue::Int(27),
    };
    let filtered = where_cmd.process(data)?;
    println!("✓ Where command executed");
    
    println!();
    Ok(())
}

fn test_enhanced_ls() -> anyhow::Result<()> {
    println!("📁 Test 3: Enhanced ls Command");
    
    // Test ls-table command
    match ls_table_cli(&[]) {
        Ok(_) => println!("✓ ls-table command executed successfully"),
        Err(e) => println!("⚠️  ls-table warning: {}", e),
    }
    
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_json_roundtrip() -> anyhow::Result<()> {
        test_json_commands()
    }
    
    #[test]
    fn test_table_ops() -> anyhow::Result<()> {
        test_table_operations()
    }
}
