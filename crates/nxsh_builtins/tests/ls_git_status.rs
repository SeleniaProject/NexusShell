use std::fs;
use std::path::Path;
use tempfile::TempDir;
use git2::Repository;
use nxsh_builtins::ls::ls_sync;

#[test]
fn test_ls_git_status_functionality() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path();
    
    // Initialize a Git repository
    let repo = Repository::init(repo_path).expect("Failed to initialize Git repository");
    
    // Create some test files
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    let modified_file = repo_path.join("modified.txt");
    fs::write(&modified_file, "Original content").expect("Failed to write modified file");
    
    // Configure Git user for commits
    let mut config = repo.config().expect("Failed to get repo config");
    config.set_str("user.name", "Test User").expect("Failed to set user name");
    config.set_str("user.email", "test@example.com").expect("Failed to set user email");
    
    // Add and commit the modified file
    let mut index = repo.index().expect("Failed to get index");
    index.add_path(Path::new("modified.txt")).expect("Failed to add file to index");
    index.write().expect("Failed to write index");
    
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
    
    // Modify the committed file
    fs::write(&modified_file, "Modified content").expect("Failed to modify file");
    
    // Test the ls function - this should show Git status
    // We can't easily test the output directly, but we can ensure it doesn't panic
    std::env::set_current_dir(repo_path).expect("Failed to change directory");
    
    // This should work without panicking and show Git status information
    let result = ls_sync(None);
    assert!(result.is_ok(), "ls_sync should succeed in Git repository");
}

#[test]
fn test_ls_non_git_directory() {
    // Create a temporary directory that's not a Git repository
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let non_git_path = temp_dir.path();
    
    // Create a test file
    let test_file = non_git_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    std::env::set_current_dir(non_git_path).expect("Failed to change directory");
    
    // This should work without panicking and not show Git status information
    let result = ls_sync(None);
    assert!(result.is_ok(), "ls_sync should succeed in non-Git directory");
}