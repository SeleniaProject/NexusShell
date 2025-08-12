// Integration tests for just target integration and README badge implementation
// Tests the implementation of "just ターゲット統合 & README へのバッジ表示"

#[cfg(test)]
mod just_integration_tests {
    use std::process::Command;
    use std::path::Path;
    use std::fs;

    /// Test that justfile contains the required command status targets
    #[test]
    fn test_justfile_contains_command_status_targets() {
        let justfile_content = fs::read_to_string("justfile")
            .expect("justfile should exist in project root");
        
        // Verify required targets are present
        assert!(justfile_content.contains("command-status-check:"), 
            "justfile should contain command-status-check target");
        
        assert!(justfile_content.contains("command-status:"), 
            "justfile should contain command-status target");
        
        assert!(justfile_content.contains("full-ci:"), 
            "justfile should contain full-ci target");
        
        assert!(justfile_content.contains("update-generated:"), 
            "justfile should contain update-generated target");
        
        assert!(justfile_content.contains("stats:"), 
            "justfile should contain stats target");
    }
    
    /// Test that README contains implementation status badges
    #[test]
    fn test_readme_contains_status_badges() {
        let readme_content = fs::read_to_string("README.md")
            .expect("README.md should exist in project root");
        
        // Verify required badges are present
        assert!(readme_content.contains("![Build Status]"), 
            "README should contain Build Status badge");
        
        assert!(readme_content.contains("![Command Coverage]"), 
            "README should contain Command Coverage badge");
        
        assert!(readme_content.contains("![Binary Size]"), 
            "README should contain Binary Size badge");
        
        assert!(readme_content.contains("![License]"), 
            "README should contain License badge");
        
        assert!(readme_content.contains("![Platform]"), 
            "README should contain Platform badge");
        
        // Verify badge links to COMMAND_STATUS.md
        assert!(readme_content.contains("](COMMAND_STATUS.md)"), 
            "Command Coverage badge should link to COMMAND_STATUS.md");
    }
    
    /// Test that gen_command_status.rs script supports --update flag
    #[test]
    fn test_gen_command_status_script_update_flag() {
        let script_content = fs::read_to_string("scripts/gen_command_status.rs")
            .expect("gen_command_status.rs should exist");
        
        // Verify --update flag support
        assert!(script_content.contains("--update"), 
            "gen_command_status.rs should support --update flag");
        
        assert!(script_content.contains("update_mode"), 
            "gen_command_status.rs should have update_mode logic");
        
        // Verify enhanced output format
        assert!(script_content.contains("Implementation Coverage:"), 
            "gen_command_status.rs should generate coverage statistics");
        
        assert!(script_content.contains("✅ Implemented"), 
            "gen_command_status.rs should use improved status indicators");
        
        assert!(script_content.contains("💤 Missing"), 
            "gen_command_status.rs should use improved status indicators for missing commands");
    }
    
    /// Test that script compilation works without external dependencies
    #[test]
    fn test_script_compilation_independence() {
        // Compile the script
        let output = Command::new("rustc")
            .args(&["--edition", "2021", "scripts/gen_command_status.rs", 
                   "-o", "target/test_gen_command_status.exe"])
            .output()
            .expect("Failed to compile gen_command_status.rs");
        
        assert!(output.status.success(), 
            "gen_command_status.rs should compile successfully without external dependencies: {}",
            String::from_utf8_lossy(&output.stderr));
        
        // Verify the executable was created
        assert!(Path::new("target/test_gen_command_status.exe").exists(), 
            "Compiled script executable should exist");
        
        // Clean up
        let _ = fs::remove_file("target/test_gen_command_status.exe");
    }
    
    /// Test command status generation with --update flag
    #[test]
    fn test_command_status_generation_with_update() {
        // Compile the script
        let compile_output = Command::new("rustc")
            .args(&["--edition", "2021", "scripts/gen_command_status.rs", 
                   "-o", "target/test_gen_command_status.exe"])
            .output()
            .expect("Failed to compile gen_command_status.rs");
        
        assert!(compile_output.status.success(), 
            "Script compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr));
        
        // Run the script with --update flag
        let run_output = Command::new("./target/test_gen_command_status.exe")
            .args(&["--update"])
            .output()
            .expect("Failed to run gen_command_status.exe");
        
        assert!(run_output.status.success(), 
            "Script execution with --update should succeed: {}",
            String::from_utf8_lossy(&run_output.stderr));
        
        // Verify output contains coverage information
        let stdout = String::from_utf8_lossy(&run_output.stdout);
        assert!(stdout.contains("coverage"), 
            "Script output should contain coverage information: {}", stdout);
        
        // Verify generated file exists
        assert!(Path::new("COMMAND_STATUS.generated.md").exists(), 
            "Script should generate COMMAND_STATUS.generated.md file");
        
        // Clean up
        let _ = fs::remove_file("target/test_gen_command_status.exe");
    }
    
    /// Test that COMMAND_STATUS.md format is correct
    #[test]
    fn test_command_status_format() {
        if !Path::new("COMMAND_STATUS.md").exists() {
            // Generate the file first
            let _ = Command::new("rustc")
                .args(&["--edition", "2021", "scripts/gen_command_status.rs", 
                       "-o", "target/gen_for_test.exe"])
                .output();
            
            let _ = Command::new("./target/gen_for_test.exe")
                .args(&["--update"])
                .output();
            
            let _ = fs::remove_file("target/gen_for_test.exe");
        }
        
        if Path::new("COMMAND_STATUS.md").exists() {
            let content = fs::read_to_string("COMMAND_STATUS.md")
                .expect("Should be able to read COMMAND_STATUS.md");
            
            // Verify header format
            assert!(content.contains("COMMAND_STATUS (auto-generated)"), 
                "COMMAND_STATUS.md should have proper header");
            
            // Verify coverage statistics
            assert!(content.contains("Implementation Coverage:"), 
                "COMMAND_STATUS.md should contain implementation coverage");
            
            // Verify table format
            assert!(content.contains("| Command | Status | Notes |"), 
                "COMMAND_STATUS.md should have proper table headers");
            
            // Verify status indicators are used
            assert!(content.contains("✅ Implemented") || content.contains("💤 Missing"), 
                "COMMAND_STATUS.md should contain proper status indicators");
            
            // Verify timestamp
            assert!(content.contains("Last updated:"), 
                "COMMAND_STATUS.md should contain timestamp information");
        }
    }
    
    /// Performance test: verify script execution time is reasonable
    #[test]
    fn test_script_execution_performance() {
        use std::time::Instant;
        
        // Compile the script
        let compile_start = Instant::now();
        let output = Command::new("rustc")
            .args(&["--edition", "2021", "scripts/gen_command_status.rs", 
                   "-o", "target/perf_test_gen.exe"])
            .output()
            .expect("Failed to compile gen_command_status.rs");
        
        let compile_time = compile_start.elapsed();
        assert!(compile_time.as_secs() < 10, 
            "Script compilation should complete within 10 seconds, took {:?}", compile_time);
        
        assert!(output.status.success(), 
            "Script compilation should succeed");
        
        // Run the script and measure execution time
        let run_start = Instant::now();
        let run_output = Command::new("./target/perf_test_gen.exe")
            .args(&["--update"])
            .output()
            .expect("Failed to run script");
        
        let run_time = run_start.elapsed();
        assert!(run_time.as_secs() < 5, 
            "Script execution should complete within 5 seconds, took {:?}", run_time);
        
        assert!(run_output.status.success(), 
            "Script execution should succeed: {}", String::from_utf8_lossy(&run_output.stderr));
        
        // Clean up
        let _ = fs::remove_file("target/perf_test_gen.exe");
    }
}
