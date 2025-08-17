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
        
        // Verify key targets are present (scripts removed; check active ones)
        for target in [
            "build:",
            "test:",
            "ci:",
            "busybox-build:",
            "busybox-size:",
            "busybox-size-gate:",
            "bench-gate:",
            "themes-validate:",
            "themes-validate-json:",
            "update-generated:",
            "stats:",
        ] {
            assert!(justfile_content.contains(target), "justfile should contain {target}");
        }
    }
    
    /// Test that README contains implementation status badges
    #[test]
    fn test_readme_contains_status_badges() {
        let readme_content = fs::read_to_string("README.md")
            .expect("README.md should exist in project root");
        
        // Verify required badges are present
        assert!(readme_content.contains("![CI]"), 
            "README should contain CI badge");
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
    
    // Script-based command status tests removed due to scripts deletion.
}
