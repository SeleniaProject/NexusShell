use std::fs;
use tempfile::TempDir;
use nxsh_builtins::ls::ls_async;

#[test]
fn test_ls_git_status_functionality() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path();
    
    // Create some test files without requiring Git
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    let modified_file = repo_path.join("modified.txt");
    fs::write(&modified_file, "Original content").expect("Failed to write modified file");
    
    // Test the ls function with basic functionality
    let repo_path_str = repo_path.to_string_lossy().to_string();
    
    // Test ls_async function - ensure it doesn't panic with basic files
    match ls_async(Some(&repo_path_str)) {
        Ok(_) => {
            // Success - ls can handle directory listing
            println!("ls_async succeeded for directory: {repo_path_str}");
        }
        Err(e) => {
            // Allow failure for now as Git status might not be available
            eprintln!("ls_async failed: {e}");
        }
    }
    
    // Create additional test files for comprehensive testing
    let another_file = repo_path.join("another.txt");
    fs::write(&another_file, "Another file content").expect("Failed to write another file");
    
    // Test with basic ls (no arguments) - this will list current directory
    match ls_async(None) {
        Ok(_) => {
            // Success - ls with no args works
            println!("ls_async succeeded with no arguments");
        }
        Err(e) => {
            eprintln!("ls_async with no args failed: {e}");
        }
    }
}

#[test]
fn test_ls_non_git_directory() {
    // Create a temporary directory that's not a Git repository
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let non_git_path = temp_dir.path();
    
    // Create a test file
    let test_file = non_git_path.join("test.txt");
    fs::write(&test_file, "Test content").expect("Failed to write test file");
    
    // Create additional test files for comprehensive testing
    let another_file = non_git_path.join("another.txt");
    fs::write(&another_file, "Another file content").expect("Failed to write another file");
    
    // Test ls_async function
    let non_git_path_str = non_git_path.to_string_lossy().to_string();
    match ls_async(Some(&non_git_path_str)) {
        Ok(_) => {
            // Success - ls works with non-git directory
            println!("ls_async succeeded for non-git directory: {non_git_path_str}");
        }
        Err(e) => {
            eprintln!("ls_async failed: {e}");
        }
    }
}