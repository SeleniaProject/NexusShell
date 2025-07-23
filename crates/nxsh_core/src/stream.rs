use std::fs::File;
use std::io::{Read, Write, Result as IoResult};
use std::sync::{Arc, Mutex};

/// Trait combining `Read` and `Write` for duplex stream.
pub trait Duplex: Read + Write + Send + Sync {}
impl<T: Read + Write + Send + Sync> Duplex for T {}

/// Reference-counted stream wrapper.
#[derive(Clone)]
pub struct Stream {
    inner: Arc<Mutex<Box<dyn Duplex>>>,
}

impl Stream {
    /// Wrap a `File` into `Stream`.
    pub fn from_file(file: File) -> Self {
        Self { inner: Arc::new(Mutex::new(Box::new(file))) }
    }

    /// Read all bytes from stream.
    pub fn read_to_end(&self) -> IoResult<Vec<u8>> {
        let mut buf = Vec::new();
        let mut guard = self.inner.lock().unwrap();
        guard.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Write bytes to stream.
    pub fn write_all(&self, data: &[u8]) -> IoResult<()> {
        let mut guard = self.inner.lock().unwrap();
        guard.write_all(data)
    }
} 