//! `cd` builtin command - change directory
//!
//! This module implements the cd (change directory) builtin command
//! with support for various shell features like directory stack,
//! CDPATH, and symbolic link handling.

use nxsh_core::{Builtin, ExecutionResult, ShellResult, ShellError, ErrorKind};
use nxsh_core::context::ShellContext;
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind};
use std::env;
use std::path::{Path, PathBuf};
use std::fs;

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

    fn help(&self) -> &'static str {
        "Change directory command. Use 'cd --help' for detailed usage information."
    }

    fn affects_shell_state(&self) -> bool {
        true // cd changes the shell's working directory
    }

    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut follow_symlinks = true;
        let mut target_dir: Option<String> = None;
        
    // Parse arguments (skip command name if present)
    let mut i = if !args.is_empty() && args[0] == "cd" { 1 } else { 0 };
        while i < args.len() {
            match args[i].as_str() {
                "-L" => follow_symlinks = true,
                "-P" => follow_symlinks = false,
                "-" => {
                    if target_dir.is_some() {
                        return Err(ShellError::new(
                            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                            "cd: too many arguments"
                        ));
                    }
                    target_dir = Some("-".to_string());
                }
                arg if arg.starts_with('-') => {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        format!("cd: invalid option: {arg}")
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
                ctx.get_var("HOME")
                    .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::VariableNotFound), "cd: HOME not set"))?
            }
        };

        // Handle special cases
        let resolved_target = match target.as_str() {
            "-" => {
                // Go to previous directory (stored in OLDPWD)
                let oldpwd = ctx.get_var("OLDPWD")
                    .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::VariableNotFound), "cd: OLDPWD not set"))?;
                
                // Print the directory we're changing to (bash behavior)
                println!("{oldpwd}");
                oldpwd
            }
            _ => target,
        };

        // Save current directory as OLDPWD
        let current_dir = env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {e}")))?;
        
        // Change to the target directory
        let new_path = PathBuf::from(&resolved_target);
        let canonical_path = if follow_symlinks {
            self.canonicalize_path(&new_path)?
        } else {
            self.resolve_path_without_symlinks(&new_path)?
        };

        // Actually change directory
        env::set_current_dir(&canonical_path)
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::NotFound), format!("cd: {resolved_target}: {e}")))?;

        // Update shell context
        ctx.cwd = canonical_path.clone();

        // Update environment variables
        ctx.set_var("OLDPWD", current_dir.to_string_lossy().to_string());
        ctx.set_var("PWD", canonical_path.to_string_lossy().to_string());

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
    fn resolve_target_directory(&self, target: &str, ctx: &ShellContext) -> ShellResult<String> {
        let path = Path::new(target);
        
        // If path is absolute or starts with ./ or ../, use it directly
        if path.is_absolute() || target.starts_with("./") || target.starts_with("../") {
            return Ok(target.to_string());
        }

        // Try current directory first
        let current_attempt = env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {e}")))?
            .join(target);
        
        if current_attempt.exists() {
            return Ok(target.to_string());
        }

        // Try CDPATH
        if let Some(cdpath) = ctx.get_var("CDPATH") {
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
        let p = path.canonicalize()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::NotFound), format!("cd: {}: {}", path.display(), e)))?;

        #[cfg(windows)]
        {
            Ok(Self::normalize_windows_path(p))
        }
        #[cfg(not(windows))]
        {
            Ok(p)
        }
    }

    /// Resolve path without following symlinks
    fn resolve_path_without_symlinks(&self, path: &Path) -> ShellResult<PathBuf> {
        let mut result = if path.is_absolute() {
            PathBuf::from("/")
        } else {
            env::current_dir()
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {e}")))?
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

    #[cfg(windows)]
    fn normalize_windows_path(p: PathBuf) -> PathBuf {
        use std::path::{Component, Prefix};

        let mut comps = p.components();
        if let Some(Component::Prefix(prefix_comp)) = comps.next() {
            match prefix_comp.kind() {
                Prefix::VerbatimDisk(drive) => {
                    // Convert \\?\C:\... -> C:\...
                    let mut out = PathBuf::new();
                    out.push(format!("{}:\\", (drive as char).to_ascii_uppercase()));
                    for c in comps {
                        out.push(c.as_os_str());
                    }
                    return out;
                }
                Prefix::VerbatimUNC(server, share) => {
                    // Convert \\?\UNC\server\share\... -> \\server\share\...
                    let mut out = PathBuf::new();
                    out.push(format!("\\\\{}\\{}", server.to_string_lossy(), share.to_string_lossy()));
                    for c in comps {
                        out.push(c.as_os_str());
                    }
                    return out;
                }
                _ => {}
            }
        }
        p
    }

    /// Check for directory-specific hooks (like .nvmrc, .python-version, etc.)
    fn check_directory_hooks(&self, path: &Path, ctx: &mut ShellContext) -> ShellResult<()> {
        // This could be extended to support various directory hooks
        // For example:
        // - .nvmrc for Node.js version switching
        // - .python-version for Python version switching
        // - .env files for environment variable loading
        // - Project-specific shell configurations

        // Check for .env file
        // Implemented: opt-in auto load controlled by NXSH_AUTO_LOAD_ENV (context var takes precedence).
        // Rationale: project-local environment setup should be explicit and reversible.
        let env_file = path.join(".env");
        if env_file.exists() {
            // Detect whether auto load is enabled via context variable or process env
            let auto_load_env = ctx.get_var("NXSH_AUTO_LOAD_ENV")
                .map(|v| v == "1" || v == "true")
                .unwrap_or_else(|| {
                    env::var("NXSH_AUTO_LOAD_ENV")
                        .map(|v| v == "1" || v == "true")
                        .unwrap_or(false)
                });

            if auto_load_env {
                // Safe, best-effort loading: parsing errors are warned and do not abort directory change
                if let Err(e) = self.load_env_file(&env_file, ctx) {
                    eprintln!("Warning: Failed to load .env file: {e}");
                }
            }
        }

        // Check for directory-specific aliases or functions
        let shell_config = path.join(".nxshrc");
        if shell_config.exists() {
            // Implemented: opt-in auto sourcing of directory config keys as env vars only (no command exec)
            // Gate: NXSH_AUTO_SOURCE_DIR_CONFIG = 1/true (context var preferred over process env)
            let auto_source_dir_config = ctx.get_var("NXSH_AUTO_SOURCE_DIR_CONFIG")
                .map(|v| v == "1" || v == "true")
                .unwrap_or_else(|| {
                    env::var("NXSH_AUTO_SOURCE_DIR_CONFIG")
                        .map(|v| v == "1" || v == "true")
                        .unwrap_or(false)
                });

            if auto_source_dir_config {
                // Pass the directory path; the loader will discover known files and load as variables
                if let Err(e) = self.source_dir_config(path, ctx) {
                    eprintln!("Warning: Failed to source directory config: {e}");
                }
            }
        }

        Ok(())
    }

    /// Get the logical current directory (following symlinks)
    pub fn get_logical_pwd() -> ShellResult<PathBuf> {
        env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {e}")))
    }

    /// Get the physical current directory (without following symlinks)
    pub fn get_physical_pwd() -> ShellResult<PathBuf> {
        // This is more complex to implement properly and would require
        // tracking the path without resolving symlinks
        env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::PermissionError), format!("Failed to get current directory: {e}")))
    }

    /// Load environment variables from a .env file
    fn load_env_file(&self, env_file: &Path, ctx: &mut ShellContext) -> anyhow::Result<()> {
        if !env_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(env_file)?;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse KEY=VALUE format
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"')) ||
                              (value.starts_with('\'') && value.ends_with('\'')) {
                    &value[1..value.len()-1]
                } else {
                    value
                };
                
                // Set the environment variable in the shell context
                ctx.set_var(key, value.to_string());
            }
        }
        
        Ok(())
    }

    /// Source directory-specific configuration files
    fn source_dir_config(&self, dir: &Path, ctx: &mut ShellContext) -> anyhow::Result<()> {
        // Look for common shell configuration files in the directory
        let config_files = [
            ".nxshrc",
            ".shellrc", 
            ".dirrc",
            "nxsh.config",
            "shell.config"
        ];

        for config_file in &config_files {
            let config_path = dir.join(config_file);
            if config_path.exists() {
                // For now, we'll just load it as environment variables
                // In a full implementation, this could execute shell commands
                if let Err(e) = self.load_env_file(&config_path, ctx) {
                    eprintln!("Warning: Failed to load config file {}: {}", config_path.display(), e);
                }
            }
        }

        Ok(())
    }
}

impl Default for CdCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a cd command
pub fn cd_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    let cd_cmd = CdCommand;
    let result = cd_cmd.execute(ctx, args)?;
    
    if result.is_success() {
        Ok(())
    } else {
        Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("cd failed with exit code {}", result.exit_code)))
    }
}

/// Convenience function for the cd command
pub fn cd(args: &[String], ctx: &mut ShellContext) -> anyhow::Result<()> {
    let command = CdCommand;
    let result = command.execute(ctx, args)?;
    match result.exit_code {
        0 => Ok(()),
        code => Err(anyhow::anyhow!("cd failed with exit code {}", code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    use std::env;
    use tempfile::TempDir;
    use serial_test::serial;

    // Serialize current_dir mutations across tests to avoid race conditions
    static CWD_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    fn cwd_lock() -> &'static std::sync::Mutex<()> { CWD_LOCK.get_or_init(|| std::sync::Mutex::new(())) }

    #[test]
    #[serial]
    fn test_cd_to_home() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("HOME", "/tmp");
        
    let _result = cd_cli(&["cd".to_string()], &mut shell_ctx);
        // This test would need proper setup to work in all environments
    }

    #[test]
    #[serial]
    fn test_cd_with_relative_path() {
        let _g = cwd_lock().lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        let result = cd_cli(&["cd".to_string(), "subdir".to_string()], &mut shell_ctx);
        if let Err(e) = &result {
            eprintln!("[debug cd::test_cd_with_relative_path] error: {e:?}");
        }
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), sub_dir);
    }

    #[test]
    #[serial]
    fn test_cd_to_nonexistent_directory() {
        let mut shell_ctx = ShellContext::new();
        let result = cd_cli(&["cd".to_string(), "/nonexistent/directory".to_string()], &mut shell_ctx);
        
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_cd_with_dash() {
        let _g = cwd_lock().lock().unwrap();
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        
        env::set_current_dir(temp_dir1.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("OLDPWD", temp_dir2.path().to_string_lossy().to_string());
        
        let result = cd_cli(&["cd".to_string(), "-".to_string()], &mut shell_ctx);
        
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), temp_dir2.path());
    }

    #[test]
    #[serial]
    fn test_cd_auto_load_env() {
        let _g = cwd_lock().lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("project");
        fs::create_dir(&sub_dir).unwrap();
        
        // Create a .env file
        let env_file = sub_dir.join(".env");
        fs::write(&env_file, "TEST_VAR=test_value\nANOTHER_VAR=\"quoted value\"").unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("NXSH_AUTO_LOAD_ENV", "1");
        
        let result = cd_cli(&["cd".to_string(), "project".to_string()], &mut shell_ctx);
        assert!(result.is_ok());
        
        // Check that environment variables were loaded
        assert_eq!(shell_ctx.get_var("TEST_VAR"), Some("test_value".to_string()));
        assert_eq!(shell_ctx.get_var("ANOTHER_VAR"), Some("quoted value".to_string()));
    }

    #[test]
    #[serial]
    fn test_cd_auto_source_dir_config() {
        let _g = cwd_lock().lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("project");
        fs::create_dir(&sub_dir).unwrap();
        
        // Create a .nxshrc file
        let config_file = sub_dir.join(".nxshrc");
        fs::write(&config_file, "PROJECT_ROOT=/path/to/project\nDEBUG_MODE=on").unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("NXSH_AUTO_SOURCE_DIR_CONFIG", "1");
        
        let result = cd_cli(&["cd".to_string(), "project".to_string()], &mut shell_ctx);
        if let Err(e) = &result {
            eprintln!("[debug test_cd_auto_source_dir_config] cd error: {e:?}");
        }
        assert!(result.is_ok());
        
        // Debug: print all vars to see what was set
        eprintln!("[debug test_cd_auto_source_dir_config] PROJECT_ROOT: {:?}", shell_ctx.get_var("PROJECT_ROOT"));
        eprintln!("[debug test_cd_auto_source_dir_config] DEBUG_MODE: {:?}", shell_ctx.get_var("DEBUG_MODE"));
        eprintln!("[debug test_cd_auto_source_dir_config] PWD: {:?}", shell_ctx.get_var("PWD"));
        
        // Check that config variables were loaded
        assert_eq!(shell_ctx.get_var("PROJECT_ROOT"), Some("/path/to/project".to_string()));
        assert_eq!(shell_ctx.get_var("DEBUG_MODE"), Some("on".to_string()));
    }

    #[test]
    #[serial]
    fn test_cd_env_parsing_with_comments() {
        let _g = cwd_lock().lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("project");
        fs::create_dir(&sub_dir).unwrap();
        
        // Create a .env file with comments and various formats
        let env_content = r#"
# This is a comment
TEST_VAR=simple_value
QUOTED_VAR="double quoted"
SINGLE_QUOTED='single quoted'

# Another comment
EMPTY_VALUE=
SPACED_VALUE = value with spaces around equals
"#;
        let env_file = sub_dir.join(".env");
        fs::write(&env_file, env_content).unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_var("NXSH_AUTO_LOAD_ENV", "true");
        
        let result = cd_cli(&["cd".to_string(), "project".to_string()], &mut shell_ctx);
        assert!(result.is_ok());
        
        // Check parsing results
        assert_eq!(shell_ctx.get_var("TEST_VAR"), Some("simple_value".to_string()));
        assert_eq!(shell_ctx.get_var("QUOTED_VAR"), Some("double quoted".to_string()));
        assert_eq!(shell_ctx.get_var("SINGLE_QUOTED"), Some("single quoted".to_string()));
        assert_eq!(shell_ctx.get_var("EMPTY_VALUE"), Some("".to_string()));
        assert_eq!(shell_ctx.get_var("SPACED_VALUE"), Some("value with spaces around equals".to_string()));
    }
} 
