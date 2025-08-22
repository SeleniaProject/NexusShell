//! JSON processing commands for NexusShell
//! 
//! NexusShell-inspired JSON manipulation and querying

use anyhow::Result;
use nxsh_core::structured_data::{StructuredValue, PipelineData, StructuredCommand};
use nxsh_core::structured_commands::{FromJsonCommand, ToJsonCommand, SelectCommand, WhereCommand};
use std::collections::HashMap;

/// Parse JSON from string input
pub fn from_json_cli(args: &[String]) -> Result<()> {
    let json_input = if args.is_empty() {
        // Read from stdin
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        args.join(" ")
    };

    let input = PipelineData::new(StructuredValue::String(json_input));
    let cmd = FromJsonCommand;
    let result = cmd.process(input)?;
    
    let output = result.format_table();
    print!("{}", output);
    
    Ok(())
}

/// Convert structured data to JSON
pub fn to_json_cli(_args: &[String]) -> Result<()> {
    // For now, create sample data to convert
    let mut sample_data = HashMap::new();
    sample_data.insert("name".to_string(), StructuredValue::String("NexusShell".to_string()));
    sample_data.insert("version".to_string(), StructuredValue::String("0.1.0".to_string()));
    sample_data.insert("features".to_string(), StructuredValue::List(vec![
        StructuredValue::String("structured-data".to_string()),
        StructuredValue::String("json-support".to_string()),
        StructuredValue::String("NexusShell-compat".to_string()),
    ]));

    let input = PipelineData::new(StructuredValue::Record(sample_data));
    let cmd = ToJsonCommand;
    let result = cmd.process(input)?;
    
    if let StructuredValue::String(json_str) = result.value {
        println!("{}", json_str);
    }
    
    Ok(())
}

/// Select specific fields from JSON/structured data
pub fn select_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("select requires at least one column name"));
    }

    // Create sample table data for demonstration
    let mut row1 = HashMap::new();
    row1.insert("name".to_string(), StructuredValue::String("Alice".to_string()));
    row1.insert("age".to_string(), StructuredValue::Int(30));
    row1.insert("city".to_string(), StructuredValue::String("Tokyo".to_string()));
    row1.insert("salary".to_string(), StructuredValue::Int(75000));

    let mut row2 = HashMap::new();
    row2.insert("name".to_string(), StructuredValue::String("Bob".to_string()));
    row2.insert("age".to_string(), StructuredValue::Int(25));
    row2.insert("city".to_string(), StructuredValue::String("Osaka".to_string()));
    row2.insert("salary".to_string(), StructuredValue::Int(65000));

    let mut row3 = HashMap::new();
    row3.insert("name".to_string(), StructuredValue::String("Charlie".to_string()));
    row3.insert("age".to_string(), StructuredValue::Int(35));
    row3.insert("city".to_string(), StructuredValue::String("Kyoto".to_string()));
    row3.insert("salary".to_string(), StructuredValue::Int(80000));

    let table = StructuredValue::Table(vec![row1, row2, row3]);
    let input = PipelineData::new(table);

    let cmd = SelectCommand {
        columns: args.to_vec(),
    };
    
    let result = cmd.process(input)?;
    let output = result.format_table();
    print!("{}", output);
    
    Ok(())
}

/// Filter data based on conditions
pub fn where_cli(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        return Err(anyhow::anyhow!("where requires column, operator, and value (e.g., 'where age > 30')"));
    }

    let column = args[0].clone();
    let operator = args[1].clone();
    let value_str = &args[2];

    // Parse value
    let value = if let Ok(int_val) = value_str.parse::<i64>() {
        StructuredValue::Int(int_val)
    } else if let Ok(float_val) = value_str.parse::<f64>() {
        StructuredValue::Float(float_val)
    } else if value_str == "true" {
        StructuredValue::Bool(true)
    } else if value_str == "false" {
        StructuredValue::Bool(false)
    } else {
        StructuredValue::String(value_str.to_string())
    };

    // Create sample table data for demonstration
    let mut row1 = HashMap::new();
    row1.insert("name".to_string(), StructuredValue::String("Alice".to_string()));
    row1.insert("age".to_string(), StructuredValue::Int(30));
    row1.insert("city".to_string(), StructuredValue::String("Tokyo".to_string()));
    row1.insert("salary".to_string(), StructuredValue::Int(75000));

    let mut row2 = HashMap::new();
    row2.insert("name".to_string(), StructuredValue::String("Bob".to_string()));
    row2.insert("age".to_string(), StructuredValue::Int(25));
    row2.insert("city".to_string(), StructuredValue::String("Osaka".to_string()));
    row2.insert("salary".to_string(), StructuredValue::Int(65000));

    let mut row3 = HashMap::new();
    row3.insert("name".to_string(), StructuredValue::String("Charlie".to_string()));
    row3.insert("age".to_string(), StructuredValue::Int(35));
    row3.insert("city".to_string(), StructuredValue::String("Kyoto".to_string()));
    row3.insert("salary".to_string(), StructuredValue::Int(80000));

    let table = StructuredValue::Table(vec![row1, row2, row3]);
    let input = PipelineData::new(table);

    let cmd = WhereCommand {
        column,
        operator,
        value,
    };
    
    let result = cmd.process(input)?;
    let output = result.format_table();
    print!("{}", output);
    
    Ok(())
}

/// Show system information in structured format
pub fn sys_cli(_args: &[String]) -> Result<()> {
    let mut system_info = HashMap::new();
    
    // OS information
    system_info.insert("os".to_string(), StructuredValue::String(std::env::consts::OS.to_string()));
    system_info.insert("arch".to_string(), StructuredValue::String(std::env::consts::ARCH.to_string()));
    system_info.insert("family".to_string(), StructuredValue::String(std::env::consts::FAMILY.to_string()));
    
    // Shell information
    system_info.insert("shell".to_string(), StructuredValue::String("NexusShell".to_string()));
    system_info.insert("version".to_string(), StructuredValue::String("0.1.0".to_string()));
    
    // Current process information
    if let Ok(exe_path) = std::env::current_exe() {
        system_info.insert("executable".to_string(), StructuredValue::Path(exe_path));
    }
    
    if let Ok(current_dir) = std::env::current_dir() {
        system_info.insert("pwd".to_string(), StructuredValue::Path(current_dir));
    }

    // Environment variables count
    let env_count = std::env::vars().count() as i64;
    system_info.insert("env_vars".to_string(), StructuredValue::Int(env_count));

    let input = PipelineData::new(StructuredValue::Record(system_info));
    let output = input.format_table();
    print!("{}", output);
    
    Ok(())
}

/// Demonstrate table operations
pub fn demo_table_cli(_args: &[String]) -> Result<()> {
    println!("🚀 NexusShell Structured Data Demo");
    println!("==================================\n");

    // Create sample employee data
    let mut employees = Vec::new();
    
    let departments = ["Engineering", "Sales", "Marketing", "HR"];
    let cities = ["Tokyo", "Osaka", "Kyoto", "Nagoya"];
    let names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"];
    
    for (i, name) in names.iter().enumerate() {
        let mut employee = HashMap::new();
        employee.insert("id".to_string(), StructuredValue::Int(i as i64 + 1));
        employee.insert("name".to_string(), StructuredValue::String(name.to_string()));
        employee.insert("department".to_string(), StructuredValue::String(departments[i % departments.len()].to_string()));
        employee.insert("age".to_string(), StructuredValue::Int(25 + (i as i64 * 3) % 20));
        employee.insert("salary".to_string(), StructuredValue::Int(50000 + (i as i64 * 5000)));
        employee.insert("city".to_string(), StructuredValue::String(cities[i % cities.len()].to_string()));
        employees.push(employee);
    }

    let table = StructuredValue::Table(employees);
    let data = PipelineData::new(table);

    println!("📊 All Employees:");
    println!("{}\n", data.format_table());

    // Demonstrate select
    println!("🔍 Select name and salary columns:");
    let select_cmd = SelectCommand {
        columns: vec!["name".to_string(), "salary".to_string()],
    };
    let selected = select_cmd.process(data.clone())?;
    println!("{}\n", selected.format_table());

    // Demonstrate where
    println!("🔎 Filter employees with salary > 60000:");
    let where_cmd = WhereCommand {
        column: "salary".to_string(),
        operator: ">".to_string(),
        value: StructuredValue::Int(60000),
    };
    let filtered = where_cmd.process(data.clone())?;
    println!("{}\n", filtered.format_table());

    println!("✨ NexusShell supports powerful data processing like NexusShell!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_round_trip() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), StructuredValue::String("test".to_string()));
        data.insert("value".to_string(), StructuredValue::Int(42));

        let record = StructuredValue::Record(data);
        let json_str = record.to_json().unwrap();
        let parsed = StructuredValue::from_json(&json_str).unwrap();

        assert_eq!(record, parsed);
    }

    #[test]
    fn test_select_command_integration() {
        // This would test the select command with actual data
        let result = select_cli(&["name".to_string(), "age".to_string()]);
        assert!(result.is_ok());
    }
}


