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

    // Parse table back (very naive split by lines)
    let mut lines = table_str.lines();
    lines.next(); // skip border
    let mut parsed = Vec::new();
    for line in lines {
        if line.starts_with('+') { break; }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() >= 3 {
            let name = cols[1].trim();
            let value = cols[2].trim();
            parsed.push(json!({"name": name, "value": value}));
        }
    }
    assert_eq!(data, json!(parsed));
} 