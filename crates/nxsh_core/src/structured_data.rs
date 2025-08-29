//! NexusShell Structured Data System
//!
//! Provides NexusShell-like structured data processing capabilities with type safety
//! and powerful pipeline operations.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Structured value types supported by NexusShell
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructuredValue {
    /// Nothing/null value
    Nothing,
    /// Boolean value
    Bool(bool),
    /// Integer number
    Int(i64),
    /// Floating point number
    Float(f64),
    /// String value
    String(String),
    /// Date/time value
    Date(DateTime<Utc>),
    /// Binary data
    Binary(Vec<u8>),
    /// List of values
    List(Vec<StructuredValue>),
    /// Record/object with named fields
    Record(HashMap<String, StructuredValue>),
    /// Table (list of records)
    Table(Vec<HashMap<String, StructuredValue>>),
    /// File path
    Path(std::path::PathBuf),
    /// Duration
    Duration(chrono::Duration),
    /// Range
    Range { start: i64, end: i64, step: i64 },
}

impl StructuredValue {
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nothing => "nothing",
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Date(_) => "date",
            Self::Binary(_) => "binary",
            Self::List(_) => "list",
            Self::Record(_) => "record",
            Self::Table(_) => "table",
            Self::Path(_) => "path",
            Self::Duration(_) => "duration",
            Self::Range { .. } => "range",
        }
    }
}

impl fmt::Display for StructuredValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            Self::Nothing => "null".to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Int(i) => i.to_string(),
            Self::Float(f_val) => f_val.to_string(),
            Self::String(s) => s.clone(),
            Self::Date(dt) => dt.to_rfc3339(),
            Self::Binary(data) => format!("binary[{}]", data.len()),
            Self::List(items) => format!("list[{}]", items.len()),
            Self::Record(fields) => format!("record[{}]", fields.len()),
            Self::Table(rows) => format!("table[{}]", rows.len()),
            Self::Path(p) => p.display().to_string(),
            Self::Duration(d) => format!("{}s", d.num_seconds()),
            Self::Range { start, end, step } => format!("{start}..{end} step {step}"),
        };
        write!(f, "{output}")
    }
}

impl StructuredValue {
    /// Convert to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Get value as integer if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            Self::Float(f) => Some(*f as i64),
            Self::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Get value as float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64),
            Self::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Get value as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nothing => false,
            Self::Bool(b) => *b,
            Self::Int(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::List(items) => !items.is_empty(),
            Self::Record(fields) => !fields.is_empty(),
            Self::Table(rows) => !rows.is_empty(),
            _ => true,
        }
    }
}

/// Pipeline data structure for command chaining
#[derive(Debug, Clone)]
pub struct PipelineData {
    /// The structured value being passed through the pipeline
    pub value: StructuredValue,
    /// Metadata about the data
    pub metadata: HashMap<String, String>,
}

impl PipelineData {
    /// Create new pipeline data
    pub fn new(value: StructuredValue) -> Self {
        Self {
            value,
            metadata: HashMap::new(),
        }
    }

    /// Create pipeline data with metadata
    pub fn with_metadata(value: StructuredValue, metadata: HashMap<String, String>) -> Self {
        Self { value, metadata }
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Convert table data to formatted string
    pub fn format_table(&self) -> String {
        match &self.value {
            StructuredValue::Table(rows) => {
                if rows.is_empty() {
                    return "Empty table".to_string();
                }

                // Get all column names
                let mut columns = std::collections::HashSet::new();
                for row in rows {
                    columns.extend(row.keys().cloned());
                }
                let mut columns: Vec<_> = columns.into_iter().collect();
                columns.sort();

                // Format as table
                let mut result = String::new();

                // Header
                result.push('┌');
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        result.push('┬');
                    }
                    result.push_str(&"─".repeat(col.len().max(10)));
                }
                result.push_str("┐\n");

                // Column names
                result.push('│');
                for col in &columns {
                    result.push_str(&format!("{:^width$}│", col, width = col.len().max(10)));
                }
                result.push('\n');

                // Separator
                result.push('├');
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        result.push('┼');
                    }
                    result.push_str(&"─".repeat(col.len().max(10)));
                }
                result.push_str("┤\n");

                // Data rows
                for row in rows {
                    result.push('│');
                    for col in &columns {
                        let value = row.get(col).map(|v| v.to_string()).unwrap_or_default();
                        result.push_str(&format!("{:width$}│", value, width = col.len().max(10)));
                    }
                    result.push('\n');
                }

                // Bottom border
                result.push('└');
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        result.push('┴');
                    }
                    result.push_str(&"─".repeat(col.len().max(10)));
                }
                result.push('┘');

                result
            }
            _ => self.value.to_string(),
        }
    }
}

/// Trait for commands that can process structured data
pub trait StructuredCommand {
    /// Process pipeline data
    fn process(&self, input: PipelineData) -> Result<PipelineData>;
}

/// Higher-order functions for data processing
impl StructuredValue {
    /// Map function over list or table
    pub fn map<F>(&self, func: F) -> Result<StructuredValue>
    where
        F: Fn(&StructuredValue) -> Result<StructuredValue>,
    {
        match self {
            Self::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(func(item)?);
                }
                Ok(Self::List(result))
            }
            Self::Table(rows) => {
                let mut result = Vec::new();
                for row in rows {
                    let row_value = Self::Record(row.clone());
                    let mapped = func(&row_value)?;
                    if let Self::Record(mapped_row) = mapped {
                        result.push(mapped_row);
                    }
                }
                Ok(Self::Table(result))
            }
            _ => func(self),
        }
    }

    /// Filter function over list or table
    pub fn filter<F>(&self, predicate: F) -> Result<StructuredValue>
    where
        F: Fn(&StructuredValue) -> Result<bool>,
    {
        match self {
            Self::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    if predicate(item)? {
                        result.push(item.clone());
                    }
                }
                Ok(Self::List(result))
            }
            Self::Table(rows) => {
                let mut result = Vec::new();
                for row in rows {
                    let row_value = Self::Record(row.clone());
                    if predicate(&row_value)? {
                        result.push(row.clone());
                    }
                }
                Ok(Self::Table(result))
            }
            _ => {
                if predicate(self)? {
                    Ok(self.clone())
                } else {
                    Ok(Self::Nothing)
                }
            }
        }
    }

    /// Reduce function over list
    pub fn reduce<F>(&self, initial: StructuredValue, func: F) -> Result<StructuredValue>
    where
        F: Fn(&StructuredValue, &StructuredValue) -> Result<StructuredValue>,
    {
        match self {
            Self::List(items) => {
                let mut accumulator = initial;
                for item in items {
                    accumulator = func(&accumulator, item)?;
                }
                Ok(accumulator)
            }
            _ => Ok(self.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_value_types() {
        let int_val = StructuredValue::Int(42);
        assert_eq!(int_val.type_name(), "int");
        assert_eq!(int_val.as_int(), Some(42));

        let str_val = StructuredValue::String("hello".to_string());
        assert_eq!(str_val.type_name(), "string");
        assert_eq!(str_val.as_string(), Some("hello"));
    }

    #[test]
    fn test_pipeline_data() {
        let data = PipelineData::new(StructuredValue::Int(42))
            .add_metadata("source".to_string(), "test".to_string());

        assert_eq!(data.value.as_int(), Some(42));
        assert_eq!(data.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_map_function() {
        let list = StructuredValue::List(vec![
            StructuredValue::Int(1),
            StructuredValue::Int(2),
            StructuredValue::Int(3),
        ]);

        let doubled = list
            .map(|v| {
                if let Some(i) = v.as_int() {
                    Ok(StructuredValue::Int(i * 2))
                } else {
                    Ok(v.clone())
                }
            })
            .unwrap();

        if let StructuredValue::List(items) = doubled {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_int(), Some(2));
            assert_eq!(items[1].as_int(), Some(4));
            assert_eq!(items[2].as_int(), Some(6));
        } else {
            panic!("Expected list");
        }
    }
}
