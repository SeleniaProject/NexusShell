use anyhow::{bail, Result};
use dirs_next::home_dir;
use std::env;
use std::path::{Path, PathBuf};
use nxsh_core::context::ShellContext;
use super::logging;

/// Change current working directory with POSIX-like semantics.
/// - `arg` None means change to `$HOME`.
/// - `~` expands to home.
/// - `-` switches to `$OLDPWD`.
/// Supports `CDPATH` lookup for non-absolute paths.
pub fn cd(arg: Option<&str>, ctx: &ShellContext) -> Result<()> {
    let target = match arg {
        None | Some("") | Some("~") => home_dir().ok_or_else(|| anyhow::anyhow!("HOME not found"))?,
        Some("-") => {
            let oldpwd = ctx.get_var("OLDPWD").ok_or_else(|| anyhow::anyhow!("OLDPWD not set"))?;
            PathBuf::from(oldpwd)
        }
        Some(path) => expand_path(path, ctx)?,
    };

    let prev = env::current_dir()?;
    env::set_current_dir(&target)?;

    // Update PWD and OLDPWD in context (thread-safe)
    ctx.set_var("OLDPWD", prev.to_string_lossy().to_string());
    ctx.set_var("PWD", target.to_string_lossy().to_string());

    logging::info_i18n(
        &format!("ディレクトリを {} に変更", target.display()),
        &format!("Changed directory to {}", target.display()),
    );

    Ok(())
}

fn expand_path(path: &str, ctx: &ShellContext) -> Result<PathBuf> {
    if Path::new(path).is_absolute() {
        return Ok(PathBuf::from(path));
    }

    // Handle ~/foo
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return Ok(home.join(stripped));
        }
    }

    // CDPATH support for first component when path does not start with ./ or ../
    if !path.starts_with("./") && !path.starts_with("../") {
        if let Some(cdpath) = ctx.get_var("CDPATH") {
            for entry in cdpath.split(':') {
                let candidate = Path::new(entry).join(path);
                if candidate.is_dir() {
                    return Ok(candidate);
                }
            }
        }
    }

    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home() {
        let ctx = ShellContext::new();
        let p = expand_path("~/", &ctx).unwrap();
        assert!(p.is_absolute());
    }
} 