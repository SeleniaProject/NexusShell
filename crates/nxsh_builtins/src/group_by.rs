use anyhow::Result;
use std::collections::HashMap;
use serde_json::{Value, json};
use std::io::{self, Read};

/// Group JSON array objects by the specified key (simple field access).
/// Usage: group-by FIELD
pub fn group_by_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("group-by requires field name");
    }
    let field = &args[0];
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let arr: Vec<Value> = serde_json::from_str(&buf)?;

    let mut map: HashMap<String, Vec<Value>> = HashMap::new();
    for item in arr {
        if let Some(val) = item.get(field) {
            let key = val.to_string();
            map.entry(key).or_default().push(item);
        }
    }
    let grouped: Value = json!(map);
    println!("{}", serde_json::to_string_pretty(&grouped)?);
    Ok(())
} 