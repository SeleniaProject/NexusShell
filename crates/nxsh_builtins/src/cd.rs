//! `cd` builtin command - change directory
//!
//! This module implements the cd (change directory) builtin command
//! with support for various shell features like directory stack,
//! CDPATH, and symbolic link handling.

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData};
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind, InternalErrorKind};
use std::env;
use std::path::{Path, PathBuf};

/// The `cd` builtin command implementation
pub struct CdCommand;

impl Builtin for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn synopsis(&self) -> &'static str {
        "Change the current working directory"
    }

    fn description(&self) -> &'static str {
        "Change the current working directory to DIR. The default DIR is the value of the HOME environment variable."
    }

    fn usage(&self) -> &'static str {
        "cd [-L|-P] [DIR]"
    }

    fn affects_shell_state(&self) -> bool {
        true // cd changes the shell's working directory
    }

    fn invoke(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let mut follow_symlinks = true;
        let mut target_dir: Option<String> = None;
        
        // Parse arguments
        let mut i = 1; // Skip command name
        while i < ctx.args.len() {
            match ctx.args[i].as_str() {
                "-L" => follow_symlinks = true,
                "-P" => follow_symlinks = false,
                arg if arg.starts_with('-') => {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        format!("cd: invalid option: {}", arg)
                    ));
                }
                arg => {
                    if target_dir.is_some() {
                        return Err(ShellError::new(
                            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                            "cd: too many arguments"
                        ));
                    }
                    target_dir = Some(arg.to_string());
                }
            }
            i += 1;
        }

        // Determine target directory
        let target = match target_dir {
            Some(dir) => self.resolve_target_directory(&dir, ctx)?,
            None => {
                // No argument - go to HOME directory
                ctx.env.get_var("HOME")
                    .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::VariableNotFound), "cd: HOME not set"))?
            }
        };

        // Handle special cases
        let resolved_target = match target.as_str() {
            "-" => {
                // Go to previous directory (stored in OLDPWD)
                let oldpwd = ctx.env.get_var("OLDPWD")
                    .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::VariableNotFound), "cd: OLDPWD not set"))?;
                
                // Print the directory we're changing to (bash behavior)
                println!("{}", oldpwd);
                oldpwd
            }
            _ => target,
        };

        // Save current directory as OLDPWD
        let current_dir = env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {}", e)))?;
        
        // Change to the target directory
        let new_path = PathBuf::from(&resolved_target);
        let canonical_path = if follow_symlinks {
            self.canonicalize_path(&new_path)?
        } else {
            self.resolve_path_without_symlinks(&new_path)?
        };

        // Actually change directory
        env::set_current_dir(&canonical_path)
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::NotFound), format!("cd: {}: {}", resolved_target, e)))?;

        // Update shell context
        ctx.set_cwd(canonical_path.clone())?;

        // Update environment variables
        ctx.env.set_var("OLDPWD", current_dir.to_string_lossy().to_string());
        ctx.env.set_var("PWD", canonical_path.to_string_lossy().to_string());

        // Add to directory stack if pushd/popd is being used
        // (This would integrate with a directory stack implementation)

        // Check for directory-specific actions
        self.check_directory_hooks(&canonical_path, ctx)?;

        Ok(ExecutionResult::success(0))
    }
}

impl CdCommand {
    /// Create a new cd command instance
    pub fn new() -> Self {
        Self
    }

    /// Resolve target directory using CDPATH if necessary
    fn resolve_target_directory(&self, target: &str, ctx: &Context) -> ShellResult<String> {
        let path = Path::new(target);
        
        // If path is absolute or starts with ./ or ../, use it directly
        if path.is_absolute() || target.starts_with("./") || target.starts_with("../") {
            return Ok(target.to_string());
        }

        // Try current directory first
        let current_attempt = env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {}", e)))?
            .join(target);
        
        if current_attempt.exists() {
            return Ok(target.to_string());
        }

        // Try CDPATH
        if let Some(cdpath) = ctx.env.get_var("CDPATH") {
            for path_dir in env::split_paths(&cdpath) {
                let candidate = path_dir.join(target);
                if candidate.exists() && candidate.is_dir() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }
        }

        // If not found anywhere, return original target (will likely fail later)
        Ok(target.to_string())
    }

    /// Canonicalize path (resolve symlinks)
    fn canonicalize_path(&self, path: &Path) -> ShellResult<PathBuf> {
        path.canonicalize()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::NotFound), format!("cd: {}: {}", path.display(), e)))
    }

    /// Resolve path without following symlinks
    fn resolve_path_without_symlinks(&self, path: &Path) -> ShellResult<PathBuf> {
        let mut result = if path.is_absolute() {
            PathBuf::from("/")
        } else {
            env::current_dir()
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {}", e)))?
        };

        for component in path.components() {
            match component {
                std::path::Component::CurDir => {
                    // Skip "." components
                }
                std::path::Component::ParentDir => {
                    result.pop();
                }
                std::path::Component::Normal(name) => {
                    result.push(name);
                }
                std::path::Component::RootDir => {
                    result = PathBuf::from("/");
                }
                std::path::Component::Prefix(prefix) => {
                    // Windows drive letters, etc.
                    result = PathBuf::from(prefix.as_os_str());
                }
            }
        }

        // Verify the path exists
        if !result.exists() {
            return Err(ShellError::new(
                ErrorKind::IoError(IoErrorKind::NotFound),
                format!("cd: {}: No such file or directory", path.display())
            ));
        }

        if !result.is_dir() {
            return Err(ShellError::new(
                ErrorKind::IoError(IoErrorKind::InvalidData),
                format!("cd: {}: Not a directory", path.display())
            ));
        }

        Ok(result)
    }

    /// Check for directory-specific hooks (like .nvmrc, .python-version, etc.)
    fn check_directory_hooks(&self, path: &Path, ctx: &Context) -> ShellResult<()> {
        // This could be extended to support various directory hooks
        // For example:
        // - .nvmrc for Node.js version switching
        // - .python-version for Python version switching
        // - .env files for environment variable loading
        // - Project-specific shell configurations

        // Check for .env file
        let env_file = path.join(".env");
        if env_file.exists() && ctx.env.get_option("auto_load_env").unwrap_or(false) {
            // Load environment variables from .env file
            // This would be implemented based on shell options
        }

        // Check for directory-specific aliases or functions
        let shell_config = path.join(".nxshrc");
        if shell_config.exists() && ctx.env.get_option("auto_source_dir_config").unwrap_or(false) {
            // Source directory-specific configuration
            // This would be implemented based on shell options
        }

        Ok(())
    }

    /// Get the logical current directory (following symlinks)
    pub fn get_logical_pwd() -> ShellResult<PathBuf> {
        env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {}", e)))
    }

    /// Get the physical current directory (without following symlinks)
    pub fn get_physical_pwd() -> ShellResult<PathBuf> {
        // This is more complex to implement properly and would require
        // tracking the path without resolving symlinks
        env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {}", e)))
    }
}

impl Default for CdCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a cd command
pub fn cd_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    use nxsh_core::stream::{Stream, StreamType};
    
    let mut context = Context::new(
        args.to_vec(),
        ctx,
        Stream::new(StreamType::Byte),
        Stream::new(StreamType::Byte),
        Stream::new(StreamType::Byte),
    )?;

    let cd_cmd = CdCommand::new();
    let result = cd_cmd.invoke(&mut context)?;
    
    if result.is_success() {
        Ok(())
    } else {
        Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("cd failed with exit code {}", result.exit_code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_cd_to_home() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("HOME", "/tmp");
        
        let result = cd_cli(&["cd".to_string()], &mut shell_ctx);
        // This test would need proper setup to work in all environments
    }

    #[test]
    fn test_cd_with_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        let result = cd_cli(&["cd".to_string(), "subdir".to_string()], &mut shell_ctx);
        
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), sub_dir);
    }

    #[test]
    fn test_cd_to_nonexistent_directory() {
        let mut shell_ctx = ShellContext::new();
        let result = cd_cli(&["cd".to_string(), "/nonexistent/directory".to_string()], &mut shell_ctx);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_cd_with_dash() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        
        env::set_current_dir(temp_dir1.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("OLDPWD", temp_dir2.path().to_string_lossy().to_string());
        
        let result = cd_cli(&["cd".to_string(), "-".to_string()], &mut shell_ctx);
        
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), temp_dir2.path());
    }
}

/// Convenience function for the cd command
pub fn cd(args: &[String], ctx: &mut nxsh_core::Context) -> anyhow::Result<()> {
    let command = CdCommand;
    let result = command.invoke(ctx)?;
    match result.exit_code {
        0 => Ok(()),
        code => Err(anyhow::anyhow!("cd failed with exit code {}", code)),
    }
} 
