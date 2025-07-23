use anyhow::Result;
use std::path::{PathBuf, Path};
use std::fs;

// Always import these for now to avoid compilation issues
use git2::{Repository, Status, StatusOptions};
use tabled::{Table, Tabled};
use humansize::{file_size_opts as options, FileSize};
use ansi_term::Colour;

use crate::icons::icon_for;

#[derive(Tabled)]
struct LsRow {
    #[tabled(rename = "")]
    icon: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Git")]
    git: String,
}

/// Asynchronous version of ls command with Git status integration
/// 
/// This function provides a complete file listing with:
/// - File icons based on type
/// - Color coding for files and directories
/// - Git status indicators (?, M, A, D, R, T, !)
/// - File size information
/// - Sorted output by filename
pub async fn ls_async(dir: Option<&str>) -> Result<()> {
    let path = PathBuf::from(dir.unwrap_or("."));
    let repo = Repository::discover(&path).ok();

    let entries = tokio::fs::read_dir(&path).await?;
    let mut entries = entries;
    let mut rows = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let md = entry.metadata().await?;
        let is_dir = md.is_dir();
        let icon = icon_for(&entry_path, is_dir);
        let name = entry_path.file_name().unwrap().to_string_lossy().to_string();
        let size = if is_dir {
            "-".into()
        } else {
            md.len().file_size(options::CONVENTIONAL)?
        };
        
        // Get Git status and apply color formatting
        let (git_status, git_color) = if let Some(r) = &repo {
            git_status_with_color(r, &entry_path)
        } else {
            (" ".to_string(), None)
        };
        
        // Apply color to name based on file type and Git status
        let colored_name = apply_color_to_name(&name, is_dir, git_color);
        let colored_icon = apply_color_to_icon(icon, git_color);
        
        rows.push(LsRow { 
            icon: colored_icon, 
            name: colored_name, 
            size, 
            git: git_status 
        });
    }
    
    // Sort rows by name for consistent output
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    
    println!("{}", Table::new(rows).to_string());
    Ok(())
}

/// Synchronous version of ls command with Git status integration
/// 
/// This function provides a complete file listing with:
/// - File icons based on type
/// - Color coding for files and directories  
/// - Git status indicators (?, M, A, D, R, T, !)
/// - File size information
/// - Sorted output by filename
pub fn ls_sync(dir: Option<&str>) -> Result<()> {
    let path = PathBuf::from(dir.unwrap_or("."));
    let repo = Repository::discover(&path).ok();

    let entries = fs::read_dir(&path)?;
    let mut rows = Vec::new();
    
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let md = entry.metadata()?;
        let is_dir = md.is_dir();
        let icon = icon_for(&entry_path, is_dir);
        let name = entry_path.file_name().unwrap().to_string_lossy().to_string();
        let size = if is_dir {
            "-".into()
        } else {
            md.len().file_size(options::CONVENTIONAL)?
        };
        
        // Get Git status and apply color formatting
        let (git_status, git_color) = if let Some(r) = &repo {
            git_status_with_color(r, &entry_path)
        } else {
            (" ".to_string(), None)
        };
        
        // Apply color to name based on file type and Git status
        let colored_name = apply_color_to_name(&name, is_dir, git_color);
        let colored_icon = apply_color_to_icon(icon, git_color);
        
        rows.push(LsRow { 
            icon: colored_icon, 
            name: colored_name, 
            size, 
            git: git_status 
        });
    }
    
    // Sort rows by name for consistent output
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    
    println!("{}", Table::new(rows).to_string());
    Ok(())
}

/// Enhanced Git status detection with color information
/// 
/// Returns a tuple of (status_character, color_option) where:
/// - status_character: Single character representing Git status (?, M, A, D, R, T, !, or space)
/// - color_option: Optional color to apply to the file/icon
/// 
/// Git Status Characters:
/// - `?` = Untracked file (red)
/// - `M` = Modified file (yellow for working tree, green for index)
/// - `A` = Added file (green)
/// - `D` = Deleted file (red for working tree, green for index)
/// - `R` = Renamed file (blue for working tree, green for index)
/// - `T` = Type changed (cyan for working tree, green for index)
/// - `!` = Ignored file (dark gray)
/// - ` ` = Clean tracked file or not in repository
fn git_status_with_color(repo: &Repository, path: &Path) -> (String, Option<Colour>) {
    // Get relative path from repository root
    let repo_workdir = match repo.workdir() {
        Some(workdir) => workdir,
        None => return (" ".to_string(), None),
    };
    
    let relative_path = match path.strip_prefix(repo_workdir) {
        Ok(rel_path) => rel_path,
        Err(_) => return (" ".to_string(), None),
    };
    
    // Convert path to string for Git operations
    let path_str = match relative_path.to_str() {
        Some(s) => s,
        None => return (" ".to_string(), None),
    };
    
    // Handle empty path (root directory)
    if path_str.is_empty() {
        return (" ".to_string(), None);
    }
    
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .pathspec(path_str);
    
    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => statuses,
        Err(_) => return (" ".to_string(), None),
    };
    
    if statuses.is_empty() {
        // File might be tracked and clean, or not in the repository
        // Try to check if it's tracked by looking at the index
        if let Ok(index) = repo.index() {
            if index.get_path(Path::new(path_str), 0).is_some() {
                // File is tracked and clean
                return (" ".to_string(), None);
            }
        }
        return (" ".to_string(), None);
    }
    
    // Find the status entry that matches our file
    let mut status_flags = None;
    for status_entry in statuses.iter() {
        if let Some(entry_path) = status_entry.path() {
            if entry_path == path_str {
                status_flags = Some(status_entry.status());
                break;
            }
        }
    }
    
    let status_flags = match status_flags {
        Some(flags) => flags,
        None => {
            // If no specific entry found, check if it's a clean tracked file
            return (" ".to_string(), None);
        }
    };
    
    // Determine status character and color based on Git status flags
    // Priority: working tree changes first, then index changes
    if status_flags.contains(Status::WT_NEW) {
        ("?".to_string(), Some(Colour::Red))
    } else if status_flags.contains(Status::WT_MODIFIED) {
        ("M".to_string(), Some(Colour::Yellow))
    } else if status_flags.contains(Status::WT_DELETED) {
        ("D".to_string(), Some(Colour::Red))
    } else if status_flags.contains(Status::WT_RENAMED) {
        ("R".to_string(), Some(Colour::Blue))
    } else if status_flags.contains(Status::WT_TYPECHANGE) {
        ("T".to_string(), Some(Colour::Cyan))
    } else if status_flags.contains(Status::INDEX_NEW) {
        ("A".to_string(), Some(Colour::Green))
    } else if status_flags.contains(Status::INDEX_MODIFIED) {
        ("M".to_string(), Some(Colour::Green))
    } else if status_flags.contains(Status::INDEX_DELETED) {
        ("D".to_string(), Some(Colour::Green))
    } else if status_flags.contains(Status::INDEX_RENAMED) {
        ("R".to_string(), Some(Colour::Green))
    } else if status_flags.contains(Status::INDEX_TYPECHANGE) {
        ("T".to_string(), Some(Colour::Green))
    } else if status_flags.contains(Status::IGNORED) {
        ("!".to_string(), Some(Colour::Fixed(8))) // Dark gray
    } else {
        // Clean files (tracked but unchanged)
        (" ".to_string(), None)
    }
}

/// Apply color to filename based on file type and Git status
/// 
/// Priority order:
/// 1. Git status color (if available)
/// 2. File type color (based on extension)
/// 3. Default color (white for files, blue for directories)
fn apply_color_to_name(name: &str, is_dir: bool, git_color: Option<Colour>) -> String {
    let base_color = if is_dir {
        Colour::Blue
    } else {
        // Determine color based on file extension
        match std::path::Path::new(name).extension().and_then(|s| s.to_str()) {
            Some("rs") => Colour::Fixed(208), // Orange for Rust files
            Some("md") => Colour::White,
            Some("toml") | Some("yaml") | Some("yml") | Some("json") => Colour::Yellow,
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => Colour::Magenta,
            Some("zip") | Some("tar") | Some("gz") | Some("bz2") | Some("xz") => Colour::Red,
            Some("exe") | Some("bin") => Colour::Green,
            Some("sh") | Some("bash") | Some("zsh") | Some("fish") => Colour::Green,
            _ => Colour::White,
        }
    };
    
    // Use Git status color if available, otherwise use file type color
    let final_color = git_color.unwrap_or(base_color);
    final_color.paint(name).to_string()
}

/// Apply color to icon based on Git status
/// 
/// If Git status color is available, apply it to the icon.
/// Otherwise, return the icon without color formatting.
fn apply_color_to_icon(icon: &'static str, git_color: Option<Colour>) -> String {
    match git_color {
        Some(color) => color.paint(icon).to_string(),
        None => icon.to_string(),
    }
}

/// Complete implementation of git_status_char function
/// 
/// Returns the appropriate Git status character for a file.
/// This is a simplified version that returns just the character without color information.
/// 
/// Git Status Characters:
/// - `?` = Untracked file
/// - `M` = Modified file
/// - `A` = Added file
/// - `D` = Deleted file
/// - `R` = Renamed file
/// - `T` = Type changed
/// - `!` = Ignored file
/// - ` ` = Clean tracked file or not in repository
fn git_status_char(repo: &Repository, path: &Path) -> Option<&'static str> {
    // Get relative path from repository root
    let repo_workdir = repo.workdir()?;
    let relative_path = path.strip_prefix(repo_workdir).ok()?;
    
    // Convert path to string for Git operations
    let path_str = relative_path.to_str()?;
    
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .pathspec(path_str);
    
    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    
    if statuses.is_empty() {
        // File is tracked and clean
        return Some(" ");
    }
    
    // Get the status for this specific file
    for status_entry in statuses.iter() {
        let entry_path = status_entry.path()?;
        if entry_path == path_str {
            let status_flags = status_entry.status();
            
            // Priority order: working tree changes first, then index changes
            if status_flags.contains(Status::WT_NEW) {
                return Some("?"); // Untracked
            } else if status_flags.contains(Status::WT_MODIFIED) {
                return Some("M"); // Modified in working tree
            } else if status_flags.contains(Status::WT_DELETED) {
                return Some("D"); // Deleted in working tree
            } else if status_flags.contains(Status::WT_RENAMED) {
                return Some("R"); // Renamed in working tree
            } else if status_flags.contains(Status::WT_TYPECHANGE) {
                return Some("T"); // Type changed in working tree
            } else if status_flags.contains(Status::INDEX_NEW) {
                return Some("A"); // Added to index
            } else if status_flags.contains(Status::INDEX_MODIFIED) {
                return Some("M"); // Modified in index
            } else if status_flags.contains(Status::INDEX_DELETED) {
                return Some("D"); // Deleted in index
            } else if status_flags.contains(Status::INDEX_RENAMED) {
                return Some("R"); // Renamed in index
            } else if status_flags.contains(Status::INDEX_TYPECHANGE) {
                return Some("T"); // Type changed in index
            } else if status_flags.contains(Status::IGNORED) {
                return Some("!"); // Ignored
            }
        }
    }
    
    // File is tracked and clean
    Some(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_git_status_char_functionality() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();
        
        // Initialize a Git repository
        let repo = Repository::init(repo_path).expect("Failed to initialize Git repository");
        
        // Create a test file
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
        
        // Test untracked file
        let status = git_status_char(&repo, &test_file);
        assert_eq!(status, Some("?"), "Untracked file should show '?'");
        
        // Configure Git user for commits
        let mut config = repo.config().expect("Failed to get repo config");
        config.set_str("user.name", "Test User").expect("Failed to set user name");
        config.set_str("user.email", "test@example.com").expect("Failed to set user email");
        
        // Add file to index
        let mut index = repo.index().expect("Failed to get index");
        index.add_path(Path::new("test.txt")).expect("Failed to add file to index");
        index.write().expect("Failed to write index");
        
        // Test added file
        let status = git_status_char(&repo, &test_file);
        assert_eq!(status, Some("A"), "Added file should show 'A'");
        
        // Commit the file
        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature = git2::Signature::now("Test User", "test@example.com").expect("Failed to create signature");
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        ).expect("Failed to create initial commit");
        
        // Test clean file
        let status = git_status_char(&repo, &test_file);
        assert_eq!(status, Some(" "), "Clean file should show ' '");
        
        // Modify the file
        fs::write(&test_file, "Modified content").expect("Failed to modify file");
        
        // Test modified file
        let status = git_status_char(&repo, &test_file);
        assert_eq!(status, Some("M"), "Modified file should show 'M'");
    }

    #[test]
    fn test_ls_sync_basic_functionality() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_path = temp_dir.path();
        
        // Create some test files
        fs::write(test_path.join("test1.txt"), "content1").expect("Failed to write test file");
        fs::write(test_path.join("test2.rs"), "fn main() {}").expect("Failed to write test file");
        fs::create_dir(test_path.join("subdir")).expect("Failed to create subdirectory");
        
        // Change to test directory
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(test_path).expect("Failed to change directory");
        
        // Test ls_sync - should not panic
        let result = ls_sync(None);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");
        
        assert!(result.is_ok(), "ls_sync should succeed");
    }
}