use std::path::Path;

/// Check whether a path exists on the filesystem.
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    Path::exists(path.as_ref())
} 