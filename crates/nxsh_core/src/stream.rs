//! Stream abstraction for NexusShell
//!
//! This module provides stream abstractions that support both traditional
//! byte streams and structured object streams for advanced pipeline operations.

use crate::error::{ErrorKind, ShellError, ShellResult};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Stream type enumeration for different data formats
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamType {
    /// Raw byte stream (traditional shell pipes)
    Byte,
    /// UTF-8 text stream with line-based processing
    Text,
    /// JSON object stream for structured data
    Json,
    /// Custom structured object stream
    Object(String), // Type name
    /// Mixed stream that can contain different types
    Mixed,
    /// Error stream for error propagation
    Error,
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamType::Byte => write!(f, "byte"),
            StreamType::Text => write!(f, "text"),
            StreamType::Json => write!(f, "json"),
            StreamType::Object(name) => write!(f, "object:{name}"),
            StreamType::Mixed => write!(f, "mixed"),
            StreamType::Error => write!(f, "error"),
        }
    }
}

/// Stream data container that can hold different types of data
#[derive(Debug, Clone)]
pub enum StreamData {
    /// Raw bytes
    Bytes(Vec<u8>),
    /// UTF-8 text
    Text(String),
    /// JSON value
    Json(serde_json::Value),
    /// Custom object (serialized)
    Object { type_name: String, data: Vec<u8> },
    /// Multiple items in a collection
    Collection(Vec<StreamData>),
    /// Key-value pairs (like a record/struct)
    Record(HashMap<String, StreamData>),
    /// Error information
    Error(String),
}

impl StreamData {
    /// Get the stream type for this data
    pub fn stream_type(&self) -> StreamType {
        match self {
            StreamData::Bytes(_) => StreamType::Byte,
            StreamData::Text(_) => StreamType::Text,
            StreamData::Json(_) => StreamType::Json,
            StreamData::Object { type_name, .. } => StreamType::Object(type_name.clone()),
            StreamData::Collection(_) => StreamType::Mixed,
            StreamData::Record(_) => StreamType::Object("record".to_string()),
            StreamData::Error(_) => StreamType::Error,
        }
    }

    /// Convert to bytes for output
    pub fn to_bytes(&self) -> ShellResult<Vec<u8>> {
        match self {
            StreamData::Bytes(bytes) => Ok(bytes.clone()),
            StreamData::Text(text) => Ok(text.as_bytes().to_vec()),
            StreamData::Json(value) => {
                let json_str = serde_json::to_string(value).map_err(|e| {
                    ShellError::new(
                        ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::ConversionError),
                        format!("JSON serialization failed: {e}"),
                    )
                })?;
                Ok(json_str.into_bytes())
            }
            StreamData::Object { data, .. } => Ok(data.clone()),
            StreamData::Collection(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.extend(item.to_bytes()?);
                    result.push(b'\n'); // Separate items with newlines
                }
                Ok(result)
            }
            StreamData::Record(record) => {
                let json_value: serde_json::Value = record
                    .iter()
                    .map(|(k, v)| {
                        let value = match v {
                            StreamData::Text(s) => serde_json::Value::String(s.clone()),
                            StreamData::Json(j) => j.clone(),
                            StreamData::Bytes(b) => {
                                serde_json::Value::String(String::from_utf8_lossy(b).to_string())
                            }
                            _ => serde_json::Value::String(format!("{v:?}")),
                        };
                        (k.clone(), value)
                    })
                    .collect::<serde_json::Map<_, _>>()
                    .into();

                let json_str = serde_json::to_string(&json_value).map_err(|e| {
                    ShellError::new(
                        ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::ConversionError),
                        format!("Record serialization failed: {e}"),
                    )
                })?;
                Ok(json_str.into_bytes())
            }
            StreamData::Error(msg) => Ok(msg.as_bytes().to_vec()),
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> ShellResult<String> {
        match self {
            StreamData::Text(text) => Ok(text.clone()),
            StreamData::Bytes(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
            StreamData::Json(value) => serde_json::to_string_pretty(value).map_err(|e| {
                ShellError::new(
                    ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::ConversionError),
                    format!("JSON serialization failed: {e}"),
                )
            }),
            StreamData::Object { type_name, data } => {
                Ok(format!("{}({})", type_name, String::from_utf8_lossy(data)))
            }
            StreamData::Collection(items) => {
                let strings: Result<Vec<_>, _> =
                    items.iter().map(|item| item.to_string()).collect();
                Ok(strings?.join("\n"))
            }
            StreamData::Record(record) => {
                let mut parts = Vec::new();
                for (key, value) in record {
                    parts.push(format!("{}: {}", key, value.to_string()?));
                }
                Ok(parts.join(", "))
            }
            StreamData::Error(msg) => Ok(msg.clone()),
        }
    }

    /// Try to parse bytes as JSON
    pub fn try_parse_json(bytes: &[u8]) -> Option<StreamData> {
        if let Ok(text) = std::str::from_utf8(bytes) {
            if let Ok(value) = serde_json::from_str(text) {
                return Some(StreamData::Json(value));
            }
        }
        None
    }

    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        match self {
            StreamData::Bytes(b) => b.is_empty(),
            StreamData::Text(t) => t.is_empty(),
            StreamData::Json(v) => v.is_null(),
            StreamData::Object { data, .. } => data.is_empty(),
            StreamData::Collection(c) => c.is_empty(),
            StreamData::Record(r) => r.is_empty(),
            StreamData::Error(_) => false,
        }
    }
}

/// Stream reader trait for reading different stream types
pub trait StreamReader: Send + Sync {
    /// Read the next item from the stream
    fn read_next(&mut self) -> ShellResult<Option<StreamData>>;

    /// Get the stream type
    fn stream_type(&self) -> StreamType;

    /// Check if the stream has more data
    fn has_more(&self) -> bool;

    /// Close the stream
    fn close(&mut self) -> ShellResult<()>;
}

/// Stream writer trait for writing different stream types
pub trait StreamWriter: Send + Sync {
    /// Write data to the stream
    fn write_data(&mut self, data: StreamData) -> ShellResult<()>;

    /// Get the stream type
    fn stream_type(&self) -> StreamType;

    /// Flush the stream
    fn flush(&mut self) -> ShellResult<()>;

    /// Close the stream
    fn close(&mut self) -> ShellResult<()>;
}

/// Main stream abstraction that can handle different types of data
#[derive(Debug, Clone)]
pub struct Stream {
    /// Stream type
    stream_type: StreamType,
    /// Internal data buffer
    data: Arc<Mutex<Vec<StreamData>>>,
    /// Stream metadata
    metadata: HashMap<String, String>,
    /// Stream position for reading
    position: Arc<Mutex<usize>>,
    /// Whether the stream is closed
    closed: Arc<Mutex<bool>>,
}

impl Stream {
    /// Create a new stream of the specified type
    pub fn new(stream_type: StreamType) -> Self {
        Self {
            stream_type,
            data: Arc::new(Mutex::new(Vec::new())),
            metadata: HashMap::new(),
            position: Arc::new(Mutex::new(0)),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a byte stream from raw data
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let mut stream = Self::new(StreamType::Byte);
        let _ = stream.write(StreamData::Bytes(data)); // Ignore errors in constructor helpers
        stream
    }

    /// Create a text stream from string
    pub fn from_string(text: String) -> Self {
        let mut stream = Self::new(StreamType::Text);
        let _ = stream.write(StreamData::Text(text)); // Ignore errors in constructor helpers
        stream
    }

    /// Create a JSON stream from serde_json::Value
    pub fn from_json(value: serde_json::Value) -> Self {
        let mut stream = Self::new(StreamType::Json);
        let _ = stream.write(StreamData::Json(value)); // Ignore errors in constructor helpers
        stream
    }

    /// Create an object stream
    pub fn from_object(type_name: String, data: Vec<u8>) -> Self {
        let mut stream = Self::new(StreamType::Object(type_name.clone()));
        let _ = stream.write(StreamData::Object { type_name, data }); // Ignore errors in constructor helpers
        stream
    }

    /// Get the stream type
    pub fn stream_type(&self) -> &StreamType {
        &self.stream_type
    }

    /// Write data to the stream
    pub fn write(&mut self, data: StreamData) -> ShellResult<()> {
        let closed = *self.closed.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream closed lock poisoned",
            )
        })?;

        if closed {
            return Err(ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::BrokenPipe),
                "Cannot write to closed stream",
            ));
        }

        let mut buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        buffer.push(data);
        Ok(())
    }

    /// Read the next item from the stream
    pub fn read(&mut self) -> ShellResult<Option<StreamData>> {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        let mut pos = self.position.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream position lock poisoned",
            )
        })?;

        if *pos >= buffer.len() {
            return Ok(None);
        }

        let data = buffer[*pos].clone();
        *pos += 1;
        Ok(Some(data))
    }

    /// Read all data from the stream
    pub fn read_all(&mut self) -> ShellResult<Vec<StreamData>> {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        Ok(buffer.clone())
    }

    /// Check if the stream has more data
    pub fn has_more(&self) -> bool {
        if let (Ok(buffer), Ok(pos)) = (self.data.lock(), self.position.lock()) {
            *pos < buffer.len()
        } else {
            false // Assume no more data if locks are poisoned
        }
    }

    /// Get stream length
    pub fn len(&self) -> usize {
        self.data.lock().map(|buffer| buffer.len()).unwrap_or(0)
    }

    /// Check if stream is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Close the stream
    pub fn close(&mut self) -> ShellResult<()> {
        let mut closed = self.closed.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream closed lock poisoned",
            )
        })?;
        *closed = true;
        Ok(())
    }

    /// Check if stream is closed
    pub fn is_closed(&self) -> bool {
        self.closed.lock().map(|closed| *closed).unwrap_or(true) // Assume closed if lock is poisoned
    }

    /// Convert stream to bytes (for traditional pipe compatibility)
    pub fn to_bytes(&self) -> ShellResult<Vec<u8>> {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        let mut result = Vec::new();
        for item in buffer.iter() {
            result.extend(item.to_bytes()?);
        }
        Ok(result)
    }

    /// Auto-detect stream type from data
    pub fn auto_detect_type(data: &[u8]) -> StreamType {
        // Try to detect if it's valid UTF-8 text
        if let Ok(text) = std::str::from_utf8(data) {
            // Try to parse as JSON
            if serde_json::from_str::<serde_json::Value>(text).is_ok() {
                return StreamType::Json;
            }

            // Check if it looks like structured text
            if text
                .lines()
                .any(|line| line.contains(':') || line.contains('='))
            {
                return StreamType::Object("structured".to_string());
            }

            return StreamType::Text;
        }

        StreamType::Byte
    }

    /// Transform stream data using a function
    pub fn map<F>(&self, f: F) -> ShellResult<Stream>
    where
        F: Fn(&StreamData) -> ShellResult<StreamData>,
    {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        let mut result = Stream::new(self.stream_type.clone());

        for item in buffer.iter() {
            let transformed = f(item)?;
            result.write(transformed)?;
        }

        Ok(result)
    }

    /// Filter stream data using a predicate
    pub fn filter<F>(&self, predicate: F) -> ShellResult<Stream>
    where
        F: Fn(&StreamData) -> bool,
    {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;

        let mut result = Stream::new(self.stream_type.clone());

        for item in buffer.iter() {
            if predicate(item) {
                result.write(item.clone())?;
            }
        }

        Ok(result)
    }

    /// Collect stream into a vector
    pub fn collect(&self) -> ShellResult<Vec<StreamData>> {
        let buffer = self.data.lock().map_err(|_| {
            ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Stream data lock poisoned",
            )
        })?;
        Ok(buffer.clone())
    }

    /// Set stream metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get stream metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Get all metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Stream conversion utilities
pub struct StreamConverter;

impl StreamConverter {
    /// Convert between different stream types
    pub fn convert(stream: &Stream, target_type: StreamType) -> ShellResult<Stream> {
        if stream.stream_type() == &target_type {
            return Ok(stream.clone());
        }

        let data = stream.collect()?;
        let mut result = Stream::new(target_type.clone());

        for item in data {
            let converted = Self::convert_data(item, &target_type)?;
            result.write(converted)?;
        }

        Ok(result)
    }

    /// Convert a single data item to target type
    fn convert_data(data: StreamData, target_type: &StreamType) -> ShellResult<StreamData> {
        match target_type {
            StreamType::Byte => Ok(StreamData::Bytes(data.to_bytes()?)),
            StreamType::Text => Ok(StreamData::Text(data.to_string()?)),
            StreamType::Json => match data {
                StreamData::Json(v) => Ok(StreamData::Json(v)),
                StreamData::Text(s) => {
                    let value: serde_json::Value = serde_json::from_str(&s).map_err(|e| {
                        ShellError::new(
                            ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                            format!("JSON parse error: {e}"),
                        )
                    })?;
                    Ok(StreamData::Json(value))
                }
                StreamData::Bytes(b) => {
                    let text = String::from_utf8_lossy(&b);
                    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
                        ShellError::new(
                            ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError),
                            format!("JSON parse error: {e}"),
                        )
                    })?;
                    Ok(StreamData::Json(value))
                }
                _ => Err(ShellError::new(
                    ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::ConversionError),
                    "Cannot convert to JSON",
                )),
            },
            StreamType::Object(type_name) => Ok(StreamData::Object {
                type_name: type_name.clone(),
                data: data.to_bytes()?,
            }),
            StreamType::Mixed => Ok(data),
            StreamType::Error => Ok(StreamData::Error(data.to_string()?)),
        }
    }
}

/// Pipe operators for stream processing
pub enum PipeOperator {
    /// Traditional byte pipe (|)
    Byte,
    /// Object pipe by value (|>)
    ObjectValue,
    /// Object pipe by reference (||>)
    ObjectReference,
}

impl fmt::Display for PipeOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipeOperator::Byte => write!(f, "|"),
            PipeOperator::ObjectValue => write!(f, "|>"),
            PipeOperator::ObjectReference => write!(f, "||>"),
        }
    }
}

/// Stream pipeline for chaining operations
pub struct StreamPipeline {
    stages: Vec<Box<dyn Fn(Stream) -> ShellResult<Stream> + Send + Sync>>,
}

impl StreamPipeline {
    /// Create a new pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline
    pub fn add_stage<F>(&mut self, stage: F) -> &mut Self
    where
        F: Fn(Stream) -> ShellResult<Stream> + Send + Sync + 'static,
    {
        self.stages.push(Box::new(stage));
        self
    }

    /// Execute the pipeline on a stream
    pub fn execute(&self, mut stream: Stream) -> ShellResult<Stream> {
        for stage in &self.stages {
            stream = stage(stream)?;
        }
        Ok(stream)
    }
}

impl Default for StreamPipeline {
    fn default() -> Self {
        Self::new()
    }
}
