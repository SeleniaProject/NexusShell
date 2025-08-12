// Investigation of extglob negation behavior to inform implementation decision
// This test file explores the expected behavior of !(pattern) for proper implementation.

use nxsh_core::{context::ShellContext, executor::Executor, result::ExecuteResult, shell_error::ShellError};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn setup_test_files(dir: &TempDir) -> std::io::Result<()> {
    let base = dir.path();
    
    // Create test files with various names
    File::create(base.join("abc.txt"))?;
    File::create(base.join("def.txt"))?;
    File::create(base.join("ghi.log"))?;
    File::create(base.join("xyz.log"))?;
    File::create(base.join("test.dat"))?;
    File::create(base.join("test.txt"))?;
    File::create(base.join("readme"))?;
    File::create(base.join("config.yaml"))?;
    
    Ok(())
}

#[tokio::test]
async fn test_extglob_negation_current_behavior() {
    let context = ShellContext::new();
    let executor = Executor::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    setup_test_files(&temp_dir).expect("Failed to setup test files");
    
    // Change to test directory
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Test current behavior with !(*.txt) pattern
    // Should currently be treated as literal "!(*.txt)" filename
    let result = executor.execute("echo !(*.txt)", &context).await;
    
    // Document current behavior for comparison
    match result {
        Ok(result) => {
            println!("Current !(*.txt) output: {:?}", result.output);
            // This should currently echo literally "!(*.txt)" since no such file exists
        }
        Err(e) => println!("Current !(*.txt) error: {:?}", e),
    }
}

#[tokio::test] 
async fn test_extglob_negation_expected_behavior() {
    let context = ShellContext::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    setup_test_files(&temp_dir).expect("Failed to setup test files");
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // List all files to establish baseline
    let all_files = fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    
    println!("Test files: {:?}", all_files);
    
    // Expected behavior of !(*.txt) should match files that DON'T end in .txt
    // That would be: ghi.log, xyz.log, test.dat, readme, config.yaml
    let expected_non_txt: Vec<String> = all_files
        .into_iter()
        .filter(|f| !f.ends_with(".txt"))
        .collect();
    
    println!("Expected !(*.txt) matches: {:?}", expected_non_txt);
    
    // This test documents what proper !(pattern) implementation should produce
    assert!(!expected_non_txt.is_empty(), "Should have non-.txt files for testing");
}

#[tokio::test]
async fn test_other_extglob_patterns_work() {
    let context = ShellContext::new();
    let executor = Executor::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    setup_test_files(&temp_dir).expect("Failed to setup test files");
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Test that other extglob patterns work correctly
    let patterns_to_test = vec![
        "*(*.txt)", // should match .txt files (zero or more times)
        "+(*.txt)", // should match .txt files (one or more times)  
        "?(*.txt)", // should match .txt files (zero or one time)
        "@(*.txt|*.log)", // should match either .txt or .log files
    ];
    
    for pattern in patterns_to_test {
        println!("\nTesting pattern: {}", pattern);
        let cmd = format!("ls {}", pattern);
        let result = executor.execute(&cmd, &context).await;
        
        match result {
            Ok(result) => println!("  Output: {:?}", result.output),
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}

// Helper to test bash compatibility (requires bash on system)
#[tokio::test]
#[ignore] // Run with --ignored flag when bash is available
async fn test_bash_negation_reference() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    setup_test_files(&temp_dir).expect("Failed to setup test files");
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Test bash behavior for reference (requires bash with extglob enabled)
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg("shopt -s extglob && echo !(*.txt)")
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Bash !(*.txt) stdout: {}", stdout.trim());
            if !stderr.is_empty() {
                println!("Bash !(*.txt) stderr: {}", stderr.trim());
            }
        }
        Err(e) => println!("Cannot run bash reference test: {}", e),
    }
}

#[tokio::test]
async fn test_complex_negation_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    setup_test_files(&temp_dir).expect("Failed to setup test files");
    
    // Add some additional test files for complex patterns
    let base = temp_dir.path();
    File::create(base.join("script.sh")).unwrap();
    File::create(base.join("data.json")).unwrap();
    File::create(base.join("image.png")).unwrap();
    
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Document expected behavior for complex negation patterns
    let test_cases = vec![
        ("!(*.txt|*.log)", "Should match files that are neither .txt nor .log"),
        ("!(test*)", "Should match files that don't start with 'test'"),
        ("!(*.{txt,log,dat})", "Should match files not having .txt, .log, or .dat extensions"),
    ];
    
    for (pattern, description) in test_cases {
        println!("\nPattern: {} - {}", pattern, description);
        
        // List files for manual verification of expected behavior
        let all_files = fs::read_dir(temp_dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        
        println!("  All files: {:?}", all_files);
        
        // Implement expected logic manually to verify our understanding
        let expected = match pattern {
            "!(*.txt|*.log)" => all_files.iter()
                .filter(|f| !f.ends_with(".txt") && !f.ends_with(".log"))
                .cloned()
                .collect::<Vec<_>>(),
            "!(test*)" => all_files.iter()
                .filter(|f| !f.starts_with("test"))
                .cloned()
                .collect::<Vec<_>>(),
            "!(*.{txt,log,dat})" => all_files.iter()
                .filter(|f| !f.ends_with(".txt") && !f.ends_with(".log") && !f.ends_with(".dat"))
                .cloned()
                .collect::<Vec<_>>(),
            _ => vec![]
        };
        
        println!("  Expected matches: {:?}", expected);
    }
}
