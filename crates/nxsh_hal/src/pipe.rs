#[cfg(unix)]
use nix::unistd::{pipe2, PipeFlags};

#[cfg(unix)]
pub fn pipe_nonblock() -> std::io::Result<(std::fs::File, std::fs::File)> {
    let (read_fd, write_fd) = pipe2(PipeFlags::O_CLOEXEC | PipeFlags::O_NONBLOCK)?;
    unsafe {
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