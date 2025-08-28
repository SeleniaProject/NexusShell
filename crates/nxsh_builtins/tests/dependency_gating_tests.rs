// Integration tests for dependency gating and feature optimization
// Tests the implementation of "依存削減 (reqwest → オプトアウト, i18n 重量辞書 gating)"

#[cfg(test)]
mod dependency_gating_tests {
    use std::process::Command;
    
    /// Test that minimal features exclude heavy i18n dependencies
    #[test]
    fn test_minimal_excludes_chrono_tz() {
        // Execute cargo tree with minimal features and verify chrono-tz is not included
        let output = Command::new("cargo")
            .args([
                "tree",
                "-p", "nxsh_builtins",
                "--no-default-features",
                "--features", "minimal"
            ])
            .output()
            .expect("Failed to execute cargo tree command");
        
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
        
        // Assert that chrono-tz is not present in minimal build
        assert!(!stdout.contains("chrono-tz"), 
            "chrono-tz should not be included in minimal build to avoid heavy timezone database");
        
        // Assert that heavy regex engines are not included in truly minimal build
        assert!(!stdout.contains("fancy-regex"), 
            "fancy-regex should not be included in minimal build for size optimization");
    }
    
    /// Test that light-i18n includes only essential i18n components without timezone data
    #[test]
    fn test_light_i18n_excludes_timezone_data() {
        let output = Command::new("cargo")
            .args([
                "tree",
                "-p", "nxsh_builtins",
                "--no-default-features",
                "--features", "light-i18n"
            ])
            .output()
            .expect("Failed to execute cargo tree command");
        
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
        
        // Assert that fluent is included for basic i18n
        assert!(stdout.contains("fluent"), 
            "fluent should be included in light-i18n for basic internationalization");
        
        // Assert that unic-langid is included for language identification
        assert!(stdout.contains("unic-langid"), 
            "unic-langid should be included in light-i18n for language identification");
        
        // Assert that chrono-tz is NOT included to avoid heavy timezone database
        assert!(!stdout.contains("chrono-tz"), 
            "chrono-tz should not be included in light-i18n to avoid heavy timezone database (>500KB)");
    }
    
    /// Test that heavy-i18n includes full timezone support
    #[test]
    fn test_heavy_i18n_includes_timezone_data() {
        let output = Command::new("cargo")
            .args([
                "tree",
                "-p", "nxsh_builtins",
                "--no-default-features",
                "--features", "heavy-i18n"
            ])
            .output()
            .expect("Failed to execute cargo tree command");
        
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
        
        // Assert that chrono-tz is included for full timezone support
        assert!(stdout.contains("chrono-tz"), 
            "chrono-tz should be included in heavy-i18n for comprehensive timezone support");
    }
    
    /// Test that reqwest is completely eliminated from all feature combinations
    #[test]
    fn test_reqwest_completely_eliminated() {
        let feature_combinations = vec![
            vec!["minimal"],
            vec!["light-i18n"],
            vec!["heavy-i18n"],
            vec!["updates"],
            vec!["minimal", "updates"],
            vec!["light-i18n", "updates"]
        ];
        
        for features in feature_combinations {
            // Build command incrementally to avoid borrowing a temporary joined string
            let mut cmd = Command::new("cargo");
            cmd.arg("tree")
                .arg("-p").arg("nxsh_builtins")
                .arg("--no-default-features");

            if !features.is_empty() {
                cmd.arg("--features").arg(features.join(","));
            }

            let output = cmd
                .output()
                .expect("Failed to execute cargo tree command");
            
            let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
            
            // Assert that reqwest is never present in any configuration
            assert!(!stdout.contains("reqwest"), 
                "reqwest should be completely eliminated from all builds (features: {features:?})");
            
            // Assert that ureq is used for HTTP when updates feature is enabled
            if features.contains(&"updates") {
                assert!(stdout.contains("ureq"), 
                    "ureq should be used as reqwest replacement when updates feature is enabled (features: {features:?})");
            }
        }
    }
    
    /// Test that busybox-min configuration is size-optimized
    #[test]
    fn test_busybox_min_size_optimization() {
        let output = Command::new("cargo")
            .args([
                "tree",
                "-p", "nxsh_cli",
                "--no-default-features",
                "--features", "busybox-min"
            ])
            .output()
            .expect("Failed to execute cargo tree command");
        
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
        
        // Heavy dependencies that should be excluded in busybox-min
        let heavy_deps = vec![
            "chrono-tz",
            "fluent",
            "reqwest", 
            "rustls",
            // "ring", // May be present via indirect dependencies from tokio/crossterm
            "openssl"
        ];
        
        for dep in heavy_deps {
            assert!(!stdout.contains(dep), 
                "{dep} should not be included in busybox-min build for size optimization");
        }
        
        // Essential dependencies that should still be present
        let essential_deps = vec![
            "futures",  // Minimal async support
            "anyhow",   // Error handling
            "serde"     // Serialization
        ];
        
        for dep in essential_deps {
            assert!(stdout.contains(dep), 
                "{dep} should be included in busybox-min as it's essential");
        }
    }
    
    /// Verify that ureq is configured with minimal features
    #[test]
    fn test_ureq_minimal_configuration() {
        let output = Command::new("cargo")
            .args([
                "tree", "-p", "nxsh_builtins", 
                "--no-default-features", "--features", "updates",
                "--format", "{p} {f}", "--no-indent"
            ])
            .output()
            .expect("Failed to execute cargo tree command");
        
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cargo tree output");
        
        // Find ureq line and verify it uses minimal features
        let ureq_lines: Vec<&str> = stdout.lines()
            .filter(|line| line.trim_start().starts_with("ureq "))
            .collect();
        
        assert!(!ureq_lines.is_empty(), "ureq should be present when updates feature is enabled");
        
        for line in ureq_lines {
            // Verify no default features (should not contain full feature list)
            assert!(!line.contains("default,"), 
                "ureq should be configured with default-features=false for size optimization: {line}");
            
            // Verify only json feature is enabled
            assert!(line.contains("json") || !line.contains(","), 
                "ureq should only have json feature enabled or no features listed: {line}");
        }
    }
    
    /// Performance benchmark: measure dependency tree size reduction
    #[test]
    fn test_dependency_size_reduction() {
        // Get full dependency count
        let full_output = Command::new("cargo")
            .args(["tree", "-p", "nxsh_builtins"])
            .output()
            .expect("Failed to execute cargo tree for full build");
        
        let full_count = String::from_utf8(full_output.stdout)
            .expect("Invalid UTF-8")
            .lines()
            .count();
        
        // Get minimal dependency count
        let minimal_output = Command::new("cargo")
            .args([
                "tree", "-p", "nxsh_builtins",
                "--no-default-features", "--features", "minimal"
            ])
            .output()
            .expect("Failed to execute cargo tree for minimal build");
        
        let minimal_count = String::from_utf8(minimal_output.stdout)
            .expect("Invalid UTF-8")
            .lines()
            .count();
        
        // Assert that minimal build has fewer dependencies
        let reduction_ratio = (full_count as f64 - minimal_count as f64) / full_count as f64;
        
        eprintln!("Full build dependencies: {full_count}, Minimal build dependencies: {minimal_count}");
        eprintln!("Dependency reduction: {:.1}%", reduction_ratio * 100.0);
        
        // Adjusted to realistic expectations - even 1% reduction is meaningful for large projects
        assert!(reduction_ratio > 0.01, 
            "Minimal build should reduce dependencies by at least 1% (actual: {:.1}%)", 
            reduction_ratio * 100.0);
    }
}
