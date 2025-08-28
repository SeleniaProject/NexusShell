//! Structured data processing commands
//! 
//! Nushell-inspired commands for working with structured data

use anyhow::Result;
use crate::structured_data::{StructuredValue, PipelineData, StructuredCommand};
use std::collections::HashMap;

/// `from json` command - parse JSON data
pub struct FromJsonCommand;

impl StructuredCommand for FromJsonCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        let json_str = match &input.value {
            StructuredValue::String(s) => s,
            _ => return Err(anyhow::anyhow!("from json requires string input")),
        };

        let parsed = StructuredValue::from_json(json_str)?;
        Ok(PipelineData::new(parsed))
    }
}

/// `to json` command - convert to JSON
pub struct ToJsonCommand;

impl StructuredCommand for ToJsonCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        let json_str = input.value.to_json()?;
        Ok(PipelineData::new(StructuredValue::String(json_str)))
    }
}

/// `select` command - select columns from table
pub struct SelectCommand {
    pub columns: Vec<String>,
}

impl StructuredCommand for SelectCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        match input.value {
            StructuredValue::Table(rows) => {
                let mut result = Vec::new();
                for row in rows {
                    let mut new_row = HashMap::new();
                    for col in &self.columns {
                        if let Some(value) = row.get(col) {
                            new_row.insert(col.clone(), value.clone());
                        }
                    }
                    result.push(new_row);
                }
                Ok(PipelineData::new(StructuredValue::Table(result)))
            }
            StructuredValue::Record(record) => {
                let mut new_record = HashMap::new();
                for col in &self.columns {
                    if let Some(value) = record.get(col) {
                        new_record.insert(col.clone(), value.clone());
                    }
                }
                Ok(PipelineData::new(StructuredValue::Record(new_record)))
            }
            _ => Err(anyhow::anyhow!("select requires table or record input")),
        }
    }
}

/// `where` command - filter rows/items
pub struct WhereCommand {
    pub column: String,
    pub operator: String,
    pub value: StructuredValue,
}

impl StructuredCommand for WhereCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        let filtered = input.value.filter(|item| {
            let field_value = match item {
                StructuredValue::Record(record) => record.get(&self.column),
                _ => None,
            };

            if let Some(field_value) = field_value {
                match self.operator.as_str() {
                    "==" => Ok(field_value == &self.value),
                    "!=" => Ok(field_value != &self.value),
                    ">" => match (field_value.as_float(), self.value.as_float()) {
                        (Some(a), Some(b)) => Ok(a > b),
                        _ => Ok(false),
                    },
                    "<" => match (field_value.as_float(), self.value.as_float()) {
                        (Some(a), Some(b)) => Ok(a < b),
                        _ => Ok(false),
                    },
                    ">=" => match (field_value.as_float(), self.value.as_float()) {
                        (Some(a), Some(b)) => Ok(a >= b),
                        _ => Ok(false),
                    },
                    "<=" => match (field_value.as_float(), self.value.as_float()) {
                        (Some(a), Some(b)) => Ok(a <= b),
                        _ => Ok(false),
                    },
                    "contains" => {
                        if let (Some(haystack), Some(needle)) = (field_value.as_string(), self.value.as_string()) {
                            Ok(haystack.contains(needle))
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        })?;

        Ok(PipelineData::new(filtered))
    }
}

/// `sort-by` command - sort table by column
pub struct SortByCommand {
    pub column: String,
    pub reverse: bool,
}

impl StructuredCommand for SortByCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        match input.value {
            StructuredValue::Table(mut rows) => {
                rows.sort_by(|a, b| {
                    let a_val = a.get(&self.column);
                    let b_val = b.get(&self.column);

                    let cmp = match (a_val, b_val) {
                        (Some(a), Some(b)) => {
                            // Try numeric comparison first
                            if let (Some(a_num), Some(b_num)) = (a.as_float(), b.as_float()) {
                                a_num.partial_cmp(&b_num).unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                // Fall back to string comparison
                                a.to_string().cmp(&b.to_string())
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (None, None) => std::cmp::Ordering::Equal,
                    };

                    if self.reverse { cmp.reverse() } else { cmp }
                });

                Ok(PipelineData::new(StructuredValue::Table(rows)))
            }
            _ => Err(anyhow::anyhow!("sort-by requires table input")),
        }
    }
}

/// `group-by` command - group table rows by column value
pub struct GroupByCommand {
    pub column: String,
}

impl StructuredCommand for GroupByCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        match input.value {
            StructuredValue::Table(rows) => {
                let mut groups: HashMap<String, Vec<HashMap<String, StructuredValue>>> = HashMap::new();

                for row in rows {
                    let key = row.get(&self.column)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "null".to_string());
                    
                    groups.entry(key).or_default().push(row);
                }

                let mut result = HashMap::new();
                for (key, group_rows) in groups {
                    result.insert(key, StructuredValue::Table(group_rows));
                }

                Ok(PipelineData::new(StructuredValue::Record(result)))
            }
            _ => Err(anyhow::anyhow!("group-by requires table input")),
        }
    }
}

/// `length` command - get length of list/table/string
pub struct LengthCommand;

impl StructuredCommand for LengthCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        let length = match &input.value {
            StructuredValue::List(items) => items.len() as i64,
            StructuredValue::Table(rows) => rows.len() as i64,
            StructuredValue::String(s) => s.len() as i64,
            StructuredValue::Record(fields) => fields.len() as i64,
            _ => 0,
        };

        Ok(PipelineData::new(StructuredValue::Int(length)))
    }
}

/// `first` command - get first N items
pub struct FirstCommand {
    pub count: usize,
}

impl StructuredCommand for FirstCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        match input.value {
            StructuredValue::List(items) => {
                let result = items.into_iter().take(self.count).collect();
                Ok(PipelineData::new(StructuredValue::List(result)))
            }
            StructuredValue::Table(rows) => {
                let result = rows.into_iter().take(self.count).collect();
                Ok(PipelineData::new(StructuredValue::Table(result)))
            }
            _ => Ok(input),
        }
    }
}

/// `last` command - get last N items
pub struct LastCommand {
    pub count: usize,
}

impl StructuredCommand for LastCommand {
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        match input.value {
            StructuredValue::List(items) => {
                let start = items.len().saturating_sub(self.count);
                let result = items.into_iter().skip(start).collect();
                Ok(PipelineData::new(StructuredValue::List(result)))
            }
            StructuredValue::Table(rows) => {
                let start = rows.len().saturating_sub(self.count);
                let result = rows.into_iter().skip(start).collect();
                Ok(PipelineData::new(StructuredValue::Table(result)))
            }
            _ => Ok(input),
        }
    }
}

/// `each` command - apply command to each item
pub struct EachCommand<F> {
    pub func: F,
}

impl<F> StructuredCommand for EachCommand<F>
where
    F: Fn(&StructuredValue) -> Result<StructuredValue>,
{
    fn process(&self, input: PipelineData) -> Result<PipelineData> {
        let result = input.value.map(&self.func)?;
        Ok(PipelineData::new(result))
    }
}

/// Convert file listing to structured table
pub fn files_to_table(files: Vec<std::fs::DirEntry>) -> Result<StructuredValue> {
    let mut rows = Vec::new();

    for file in files {
        let mut row = HashMap::new();
        let metadata = file.metadata()?;
        
        row.insert("name".to_string(), StructuredValue::String(
            file.file_name().to_string_lossy().to_string()
        ));
        
        row.insert("type".to_string(), StructuredValue::String(
            if metadata.is_dir() { "directory" } else { "file" }.to_string()
        ));
        
        row.insert("size".to_string(), StructuredValue::Int(metadata.len() as i64));
        
        if let Ok(modified) = metadata.modified() {
            let datetime = chrono::DateTime::<chrono::Utc>::from(modified);
            row.insert("modified".to_string(), StructuredValue::Date(datetime));
        }
        
        rows.push(row);
    }

    Ok(StructuredValue::Table(rows))
}

/// Convert a list of file paths to a structured table
pub fn paths_to_table(paths: &[std::path::PathBuf]) -> anyhow::Result<StructuredValue> {
    let mut rows = Vec::new();
    
    for path in paths {
        let mut row = std::collections::HashMap::new();
        
        row.insert("name".to_string(), StructuredValue::String(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        ));
        
        row.insert("path".to_string(), StructuredValue::Path(path.clone()));
        
        if let Ok(metadata) = path.metadata() {
            row.insert("type".to_string(), StructuredValue::String(
                if metadata.is_dir() { "directory" } else { "file" }.to_string()
            ));
            
            row.insert("size".to_string(), StructuredValue::Int(metadata.len() as i64));
            
            if let Ok(modified) = metadata.modified() {
                let datetime = chrono::DateTime::<chrono::Utc>::from(modified);
                row.insert("modified".to_string(), StructuredValue::Date(datetime));
            }
        }
        
        rows.push(row);
    }
    
    Ok(StructuredValue::Table(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_command() {
        let mut row1 = HashMap::new();
        row1.insert("name".to_string(), StructuredValue::String("Alice".to_string()));
        row1.insert("age".to_string(), StructuredValue::Int(30));
        row1.insert("city".to_string(), StructuredValue::String("Tokyo".to_string()));

        let mut row2 = HashMap::new();
        row2.insert("name".to_string(), StructuredValue::String("Bob".to_string()));
        row2.insert("age".to_string(), StructuredValue::Int(25));
        row2.insert("city".to_string(), StructuredValue::String("Osaka".to_string()));

        let table = StructuredValue::Table(vec![row1, row2]);
        let input = PipelineData::new(table);

        let select_cmd = SelectCommand {
            columns: vec!["name".to_string(), "age".to_string()],
        };

        let result = select_cmd.process(input).unwrap();
        
        if let StructuredValue::Table(rows) = result.value {
            assert_eq!(rows.len(), 2);
            assert!(rows[0].contains_key("name"));
            assert!(rows[0].contains_key("age"));
            assert!(!rows[0].contains_key("city"));
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_where_command() {
        let mut row1 = HashMap::new();
        row1.insert("name".to_string(), StructuredValue::String("Alice".to_string()));
        row1.insert("age".to_string(), StructuredValue::Int(30));

        let mut row2 = HashMap::new();
        row2.insert("name".to_string(), StructuredValue::String("Bob".to_string()));
        row2.insert("age".to_string(), StructuredValue::Int(25));

        let table = StructuredValue::Table(vec![row1, row2]);
        let input = PipelineData::new(table);

        let where_cmd = WhereCommand {
            column: "age".to_string(),
            operator: ">".to_string(),
            value: StructuredValue::Int(27),
        };

        let result = where_cmd.process(input).unwrap();
        
        if let StructuredValue::Table(rows) = result.value {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get("name").unwrap().as_string(), Some("Alice"));
        } else {
            panic!("Expected table");
        }
    }
}
