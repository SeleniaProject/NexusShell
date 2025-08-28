//! Comprehensive tests for Zstandard dictionary training functionality
//!
//! This test suite validates all aspects of the dictionary training implementation:
//! - Suffix array construction and pattern extraction
//! - Dictionary training from sample data
//! - File I/O operations for dictionaries
//! - Command-line interface integration
//! - Edge cases and error handling

#[cfg(test)]
mod dictionary_training_tests {
    use super::super::super::{ZstdOptions, train_dictionary, zstd_cli};
    use super::super::dictionary_trainer::*;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use std::path::Path;
    use tempfile::{TempDir, NamedTempFile};
    use anyhow::Result;

    /// Test suffix array construction with various input patterns
    #[test]
    fn test_suffix_array_comprehensive() {
        use super::super::dictionary_trainer::SuffixArray;
        
        // Test empty input
        let empty_sa = SuffixArray::new(Vec::new());
        assert_eq!(empty_sa.sa.len(), 0);
        assert_eq!(empty_sa.lcp.len(), 0);

        // Test single character
        let single_sa = SuffixArray::new(b"a".to_vec());
        assert_eq!(single_sa.sa.len(), 1);
        assert_eq!(single_sa.sa[0], 0);

        // Test repeated pattern
        let repeated_sa = SuffixArray::new(b"abcabcabc".to_vec());
        assert_eq!(repeated_sa.sa.len(), 9);
        
        // Verify suffixes are sorted lexicographically
        for i in 1..repeated_sa.sa.len() {
            let suffix1 = &repeated_sa.text[repeated_sa.sa[i-1]..];
            let suffix2 = &repeated_sa.text[repeated_sa.sa[i]..];
            assert!(suffix1 <= suffix2, "Suffixes not properly sorted");
        }

        // Test with special characters and unicode
        let special_sa = SuffixArray::new("Hello 世界! 🌍".as_bytes().to_vec());
        assert_eq!(special_sa.sa.len(), "Hello 世界! 🌍".as_bytes().len());
    }

    /// Test pattern extraction with various frequencies and lengths
    #[test]
    fn test_pattern_extraction_comprehensive() {
        use super::super::dictionary_trainer::SuffixArray;
        
        // Create text with known patterns
        let text = b"abcabcabcdefdefdefghighighi".to_vec();
        let sa = SuffixArray::new(text);

        // Extract 3-character patterns with frequency >= 3
        let patterns = sa.extract_patterns(3, 3);
        
        // Should find "abc", "def", "ghi" patterns
        let pattern_data: std::collections::HashSet<_> = patterns.iter()
            .map(|p| p.data.as_slice())
            .collect();
        
        assert!(pattern_data.contains(&b"abc"[..]), "Should find 'abc' pattern");
        assert!(pattern_data.contains(&b"def"[..]), "Should find 'def' pattern");
        assert!(pattern_data.contains(&b"ghi"[..]), "Should find 'ghi' pattern");

        // Verify pattern frequencies
        for pattern in &patterns {
            if pattern.data == b"abc" || pattern.data == b"def" || pattern.data == b"ghi" {
                assert!(pattern.frequency >= 3, "Pattern frequency should be >= 3");
            }
        }

        // Test minimum frequency filtering
        let rare_patterns = sa.extract_patterns(3, 10);
        assert!(rare_patterns.is_empty(), "Should not find patterns with frequency >= 10");
    }

    /// Test entropy calculation for various byte distributions
    #[test]
    fn test_entropy_calculation() {
        use super::super::dictionary_trainer::SuffixArray;
        
        // Test uniform distribution (high entropy, low factor)
        let uniform: Vec<u8> = (0..=255).collect();
        let entropy_factor = SuffixArray::calculate_entropy_factor(&uniform);
        assert!(entropy_factor < 0.2, "Uniform distribution should have low entropy factor");

        // Test single repeated byte (low entropy, high factor)
        let repeated = vec![65u8; 100];
        let entropy_factor = SuffixArray::calculate_entropy_factor(&repeated);
        assert!(entropy_factor > 0.9, "Repeated bytes should have high entropy factor");

        // Test binary distribution
        let binary = [vec![0u8; 50], vec![255u8; 50]].concat();
        let entropy_factor = SuffixArray::calculate_entropy_factor(&binary);
        assert!(entropy_factor > 0.3 && entropy_factor < 0.8, "Binary distribution should have medium entropy factor");

        // Test empty input
        let empty_entropy = SuffixArray::calculate_entropy_factor(&[]);
        assert_eq!(empty_entropy, 0.0, "Empty input should have zero entropy factor");
    }

    /// Test dictionary trainer configuration and basic functionality
    #[test]
    fn test_dictionary_trainer_configuration() {
        // Test default configuration
        let default_config = DictionaryTrainerConfig::default();
        assert_eq!(default_config.dict_size, 65536);
        assert_eq!(default_config.min_pattern_length, 4);
        assert_eq!(default_config.max_pattern_length, 256);
        assert_eq!(default_config.min_frequency, 10);

        // Test custom configuration
        let custom_config = DictionaryTrainerConfig {
            dict_size: 32768,
            min_pattern_length: 2,
            max_pattern_length: 128,
            min_frequency: 5,
            patterns_per_length: 500,
            max_sample_size: 512 * 1024,
        };

        let trainer = DictionaryTrainer::new(custom_config.clone());
        assert_eq!(trainer.config.dict_size, 32768);
        assert_eq!(trainer.config.min_pattern_length, 2);
    }

    /// Test dictionary training with realistic sample data
    #[test]
    fn test_dictionary_training_realistic() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create realistic sample files with common patterns
        let samples = vec![
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1234\r\n\r\n{\"status\":\"success\",\"data\":{\"user\":\"john\",\"id\":12345}}",
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 5678\r\n\r\n{\"status\":\"success\",\"data\":{\"user\":\"jane\",\"id\":67890}}",
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: 567\r\n\r\n{\"status\":\"error\",\"message\":\"Not found\"}",
            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: 890\r\n\r\n{\"status\":\"error\",\"message\":\"Internal error\"}",
        ];

        let mut sample_files = Vec::new();
        for (i, sample) in samples.iter().enumerate() {
            let file_path = temp_dir.path().join(format!("sample_{}.txt", i));
            let mut file = File::create(&file_path)?;
            file.write_all(sample.as_bytes())?;
            sample_files.push(file_path.to_string_lossy().to_string());
        }

        // Configure trainer for HTTP-like data
        let config = DictionaryTrainerConfig {
            dict_size: 1024,
            min_pattern_length: 4,
            max_pattern_length: 64,
            min_frequency: 2, // Lower threshold for test data
            patterns_per_length: 100,
            max_sample_size: 0, // No size limit
        };

        let dictionary = train_dictionary_from_files(&sample_files, config)?;

        // Validate dictionary properties
        assert!(dictionary.is_valid(), "Dictionary should be valid");
        assert!(dictionary.size() <= 1024, "Dictionary should respect size limit");
        assert!(dictionary.id != 0, "Dictionary should have non-zero ID");

        // Check that common HTTP patterns are likely included
        let dict_str = String::from_utf8_lossy(&dictionary.data);
        assert!(
            dict_str.contains("HTTP") || dict_str.contains("Content-Type") || dict_str.contains("status"),
            "Dictionary should contain common HTTP patterns"
        );

        Ok(())
    }

    /// Test dictionary file I/O operations
    #[test]
    fn test_dictionary_file_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test dictionary
        let original_dict = ZstdDictionary {
            id: 0x12345678,
            data: b"Test dictionary content with various patterns".to_vec(),
        };

        // Save to file
        let dict_path = temp_dir.path().join("test.dict");
        original_dict.save_to_file(&dict_path)?;

        // Verify file exists and has expected structure
        assert!(dict_path.exists(), "Dictionary file should be created");
        
        let file_data = std::fs::read(&dict_path)?;
        assert!(file_data.len() > 8, "File should contain header + data");
        assert_eq!(&file_data[0..4], b"\x37\xa4\x30\xec", "Should have Zstandard magic number");

        // Load from file
        let loaded_dict = ZstdDictionary::from_file(&dict_path)?;

        // Verify loaded dictionary matches original
        assert_eq!(loaded_dict.id, original_dict.id, "Dictionary ID should match");
        assert_eq!(loaded_dict.data, original_dict.data, "Dictionary data should match");
        assert_eq!(loaded_dict.size(), original_dict.size(), "Dictionary size should match");

        // Test loading raw dictionary (without magic number)
        let raw_dict_path = temp_dir.path().join("raw.dict");
        std::fs::write(&raw_dict_path, &original_dict.data)?;
        
        let loaded_raw = ZstdDictionary::from_file(&raw_dict_path)?;
        assert_eq!(loaded_raw.data, original_dict.data, "Raw dictionary data should match");
        assert_ne!(loaded_raw.id, 0, "Raw dictionary should get generated ID");

        Ok(())
    }

    /// Test command-line interface integration
    #[test]
    fn test_dictionary_training_cli() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create sample files
        let sample1_path = temp_dir.path().join("sample1.txt");
        let sample2_path = temp_dir.path().join("sample2.txt");
        
        std::fs::write(&sample1_path, "The quick brown fox jumps over the lazy dog. The quick brown fox is fast.")?;
        std::fs::write(&sample2_path, "The lazy dog sleeps under the tree. The lazy dog is tired.")?;

        let dict_path = temp_dir.path().join("output.dict");

        // Test training via CLI
        let args = vec![
            "--train".to_string(),
            "--maxdict".to_string(),
            "512".to_string(),
            "-o".to_string(),
            dict_path.to_string_lossy().to_string(),
            sample1_path.to_string_lossy().to_string(),
            sample2_path.to_string_lossy().to_string(),
        ];

        let result = zstd_cli(&args);
        assert!(result.is_ok(), "Dictionary training CLI should succeed: {:?}", result);

        // Verify output file was created
        assert!(dict_path.exists(), "Dictionary file should be created");

        // Verify dictionary can be loaded
        let dictionary = ZstdDictionary::from_file(&dict_path)?;
        assert!(dictionary.is_valid(), "Generated dictionary should be valid");
        assert!(dictionary.size() <= 512, "Dictionary should respect size limit");

        Ok(())
    }

    /// Test edge cases and error handling
    #[test]
    fn test_dictionary_training_edge_cases() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test empty sample file
        let empty_file = temp_dir.path().join("empty.txt");
        std::fs::write(&empty_file, "")?;

        let mut trainer = DictionaryTrainer::new(DictionaryTrainerConfig::default());
        let result = trainer.add_sample_file(&empty_file);
        assert!(result.is_ok(), "Adding empty file should not error");

        // Test non-existent file
        let non_existent = temp_dir.path().join("does_not_exist.txt");
        let result = trainer.add_sample_file(&non_existent);
        assert!(result.is_err(), "Adding non-existent file should error");

        // Test training with no samples
        let empty_trainer = DictionaryTrainer::new(DictionaryTrainerConfig::default());
        let result = empty_trainer.train_dictionary();
        assert!(result.is_err(), "Training with no samples should error");

        // Test training with very small dictionary size
        let small_file = temp_dir.path().join("small.txt");
        std::fs::write(&small_file, "abcdefghijklmnopqrstuvwxyz")?;

        let config = DictionaryTrainerConfig {
            dict_size: 4, // Very small
            min_pattern_length: 2,
            max_pattern_length: 8,
            min_frequency: 1,
            patterns_per_length: 10,
            max_sample_size: 0,
        };

        let result = train_dictionary_from_files(&[&small_file], config);
        assert!(result.is_ok(), "Training with small dictionary size should work");

        if let Ok(dict) = result {
            assert!(dict.size() <= 4, "Dictionary should respect tiny size limit");
        }

        // Test with maximum sample size limit
        let large_file = temp_dir.path().join("large.txt");
        let large_content = "x".repeat(1024 * 1024); // 1MB
        std::fs::write(&large_file, &large_content)?;

        let config = DictionaryTrainerConfig {
            dict_size: 1024,
            min_pattern_length: 2,
            max_pattern_length: 16,
            min_frequency: 10,
            patterns_per_length: 100,
            max_sample_size: 1024, // 1KB limit
        };

        let result = train_dictionary_from_files(&[&large_file], config);
        assert!(result.is_ok(), "Training with sample size limit should work");

        Ok(())
    }

    /// Test pattern scoring algorithm
    #[test]
    fn test_pattern_scoring() {
        use super::super::dictionary_trainer::SuffixArray;
        
        // Test patterns with different characteristics
        let patterns = vec![
            (b"abcd".to_vec(), 10, 4), // Medium frequency, short
            (b"abcdefgh".to_vec(), 5, 8), // Lower frequency, longer
            (b"xxxx".to_vec(), 20, 4), // High frequency, low entropy
            (b"abcd".to_vec(), 100, 4), // Very high frequency
        ];

        let mut scored_patterns = Vec::new();
        for (data, frequency, length) in patterns {
            let score = SuffixArray::calculate_pattern_score(&data, frequency, length);
            scored_patterns.push((data, frequency, length, score));
        }

        // Verify scoring makes sense
        let high_freq_score = scored_patterns.iter()
            .find(|(data, freq, _, _)| *freq == 100 && data == b"abcd")
            .map(|(_, _, _, score)| *score)
            .unwrap();

        let low_freq_score = scored_patterns.iter()
            .find(|(data, freq, _, _)| *freq == 10 && data == b"abcd")
            .map(|(_, _, _, score)| *score)
            .unwrap();

        assert!(high_freq_score > low_freq_score, "Higher frequency should result in higher score");

        // Longer patterns with reasonable frequency should score well
        let long_pattern_score = scored_patterns.iter()
            .find(|(data, _, _, _)| data.len() == 8)
            .map(|(_, _, _, score)| *score)
            .unwrap();

        assert!(long_pattern_score > 0.0, "Long patterns should have positive scores");
    }

    /// Test dictionary ID generation consistency
    #[test]
    fn test_dictionary_id_generation() {
        let data1 = b"test dictionary content".to_vec();
        let data2 = b"test dictionary content".to_vec(); // Same content
        let data3 = b"different dictionary content".to_vec();

        let id1 = ZstdDictionary::generate_id_for_data(&data1);
        let id2 = ZstdDictionary::generate_id_for_data(&data2);
        let id3 = ZstdDictionary::generate_id_for_data(&data3);

        assert_eq!(id1, id2, "Same content should generate same ID");
        assert_ne!(id1, id3, "Different content should generate different ID");
        assert_ne!(id1, 0, "Generated ID should not be zero");
    }

    /// Performance test for large datasets
    #[test]
    #[ignore] // Long-running test, run with --ignored
    fn test_dictionary_training_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create large sample file (1MB)
        let large_sample = temp_dir.path().join("large_sample.txt");
        let mut file = File::create(&large_sample)?;
        
        // Generate realistic text with repeated patterns
        for i in 0..10000 {
            writeln!(file, "Log entry {}: User action performed successfully with timestamp {}", i, i * 1000)?;
            writeln!(file, "Request ID: {}, Status: OK, Response time: {}ms", i, (i % 100) + 10)?;
            if i % 100 == 0 {
                writeln!(file, "Checkpoint reached at iteration {}", i)?;
            }
        }

        let config = DictionaryTrainerConfig {
            dict_size: 64 * 1024, // 64KB
            min_pattern_length: 4,
            max_pattern_length: 64,
            min_frequency: 50,
            patterns_per_length: 1000,
            max_sample_size: 0,
        };

        let start_time = std::time::Instant::now();
        let result = train_dictionary_from_files(&[&large_sample], config);
        let duration = start_time.elapsed();

        assert!(result.is_ok(), "Large dataset training should succeed");
        assert!(duration.as_secs() < 30, "Training should complete within 30 seconds");

        if let Ok(dict) = result {
            assert!(dict.is_valid(), "Large dataset dictionary should be valid");
            assert!(dict.size() <= 64 * 1024, "Dictionary should respect size limit");
        }

        Ok(())
    }
}
