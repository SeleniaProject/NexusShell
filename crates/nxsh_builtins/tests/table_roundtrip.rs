use tabled::{Table, Tabled};
use serde_json::json;

#[derive(Tabled)]
struct Row<'a> {
    name: &'a str,
    value: &'a str,
}

#[test]
fn json_table_roundtrip() {
    // Original JSON array
    let data = json!([
        {"name":"alpha","value":"1"},
        {"name":"beta","value":"2"}
    ]);
    // Convert to table
    let rows: Vec<Row> = data
        .as_array()
        .unwrap()
        .iter()
        .map(|obj| Row {
            name: obj.get("name").unwrap().as_str().unwrap(),
            value: obj.get("value").unwrap().as_str().unwrap(),
        })
        .collect();
    let table_str = Table::new(rows).to_string();

    // Debug the table output
    println!("Table output:\n{table_str}");
    
    // Parse table back (more robust parsing)
    let lines: Vec<&str> = table_str.lines().collect();
    let mut parsed = Vec::new();
    
    // Skip header lines and borders
    for line in lines.iter().skip(2) { // Skip top border and header
        if line.starts_with('+') || !line.starts_with('|') { 
            continue; // Skip border lines and empty lines
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() >= 3 {
            let name = cols[1].trim();
            let value = cols[2].trim();
            // Skip header row
            if name != "name" && value != "value" {
                parsed.push(json!({"name": name, "value": value}));
            }
        }
    }
    
    assert_eq!(data, json!(parsed));
} 