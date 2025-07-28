#[cfg(unix)]
use nix::unistd::pipe;
#[cfg(unix)]
use std::os::fd::FromRawFd;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::error::{HalError, HalResult};

#[cfg(unix)]
pub fn pipe_nonblock() -> std::io::Result<(std::fs::File, std::fs::File)> {
    let (read_fd, write_fd) = pipe()?;
    unsafe {
        // Set non-blocking and close-on-exec flags
        let flags = libc::fcntl(read_fd, libc::F_GETFL);
        libc::fcntl(read_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        libc::fcntl(read_fd, libc::F_SETFD, libc::FD_CLOEXEC);
        
        let flags = libc::fcntl(write_fd, libc::F_GETFL);
        libc::fcntl(write_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        libc::fcntl(write_fd, libc::F_SETFD, libc::FD_CLOEXEC);
        
        Ok((
            std::fs::File::from_raw_fd(read_fd),
            std::fs::File::from_raw_fd(write_fd),
        ))
    }
}

#[cfg(windows)]
pub fn pipe_nonblock() -> std::io::Result<(std::fs::File, std::fs::File)> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "pipe_nonblock not yet supported on Windows",
    ))
}

/// Handle to a pipe
#[derive(Debug)]
pub struct PipeHandle {
    pub id: u32,
    pub read_fd: Option<std::fs::File>,
    pub write_fd: Option<std::fs::File>,
}

impl PipeHandle {
    pub fn new(id: u32, read_fd: std::fs::File, write_fd: std::fs::File) -> Self {
        Self {
            id,
            read_fd: Some(read_fd),
            write_fd: Some(write_fd),
        }
    }
}

/// Manager for system pipes
#[derive(Debug)]
pub struct PipeManager {
    pipes: Arc<Mutex<HashMap<u32, PipeHandle>>>,
    next_id: Arc<Mutex<u32>>,
}

impl PipeManager {
    pub fn new() -> Self {
        Self {
            pipes: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn create_pipe(&self) -> HalResult<PipeHandle> {
        let (read_fd, write_fd) = pipe_nonblock()
            .map_err(|e| HalError::io_error("create_pipe", None, e))?;
        
        let id = {
            let mut next_id = self.next_id.lock()
                .map_err(|_| HalError::resource_error("Pipe ID counter lock poisoned"))?;
            *next_id += 1;
            *next_id
        };

        let handle = PipeHandle::new(id, read_fd, write_fd);
        
        {
            let mut pipes = self.pipes.lock()
                .map_err(|_| HalError::resource_error("Pipe manager lock poisoned"))?;
            pipes.insert(id, PipeHandle {
                id: handle.id,
                read_fd: None, // Move ownership to caller
                write_fd: None, // Move ownership to caller
            });
        }

        Ok(handle)
    }

    pub fn get_pipe(&self, id: u32) -> HalResult<Option<u32>> {
        let pipes = self.pipes.lock()
            .map_err(|_| HalError::resource_error("Pipe manager lock poisoned"))?;
        Ok(pipes.get(&id).map(|handle| handle.id))
    }

    pub fn remove_pipe(&self, id: u32) -> HalResult<()> {
        let mut pipes = self.pipes.lock()
            .map_err(|_| HalError::resource_error("Pipe manager lock poisoned"))?;
        pipes.remove(&id);
        Ok(())
    }
}

impl Default for PipeManager {
    fn default() -> Self {
        Self::new()
    }
} 