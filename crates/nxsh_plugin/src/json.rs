use anyhow::Result;
use serde_json::Value;

/// Deserialize UTF-8 JSON bytes into serde_json::Value object.
pub fn deserialize_json(bytes: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice(bytes)?)
}
