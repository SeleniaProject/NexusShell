#[cfg(unix)]
use nix::unistd::pipe;
#[cfg(unix)]
use std::os::fd::FromRawFd;

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