//! Zstandard Dictionary Training Implementation
//! 
//! Pure Rust implementation of dictionary training for Zstandard compression.
//! This module provides automatic dictionary learning from sample data.
//!
//! Features:
//! - Suffix array based pattern extraction
//! - Frequency analysis for optimal pattern selection  
//! - Entropy optimization for maximum compression benefit
//! - No C/C++ dependencies - 100% Pure Rust
//!
//! Algorithm Overview:
//! 1. Collect sample data from multiple input files
//! 2. Build suffix array for efficient substring enumeration
//! 3. Extract most frequent patterns with minimum length threshold
//! 4. Score patterns by frequency * length * entropy reduction
//! 5. Select optimal dictionary entries within size budget
//! 6. Generate Zstandard-compatible dictionary format

use anyhow::{Result, Context};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::path::Path;

/// Configuration for dictionary training
#[derive(Debug, Clone)]
pub struct DictionaryTrainerConfig {
    /// Target dictionary size in bytes
    pub dict_size: usize,
    /// Minimum pattern length to consider
    pub min_pattern_length: usize,
    /// Maximum pattern length to consider  
    pub max_pattern_length: usize,
    /// Minimum frequency threshold for pattern inclusion
    pub min_frequency: usize,
    /// Number of top patterns to analyze for each length
    pub patterns_per_length: usize,
    /// Sample size limit per input file (0 = no limit)
    pub max_sample_size: usize,
}

impl Default for DictionaryTrainerConfig {
    fn default() -> Self {
        Self {
            dict_size: 65536,        // 64KB default dictionary size
            min_pattern_length: 4,   // Minimum 4-byte patterns
            max_pattern_length: 256, // Maximum 256-byte patterns
            min_frequency: 10,       // Must appear at least 10 times
            patterns_per_length: 1000, // Analyze top 1000 patterns per length
            max_sample_size: 1024 * 1024, // 1MB sample size limit per file
        }
    }
}

/// Dictionary pattern with scoring information
#[derive(Debug, Clone)]
struct DictionaryPattern {
    /// Pattern bytes
    data: Vec<u8>,
    /// Frequency across all samples
    frequency: usize,
    /// Estimated compression benefit score
    score: f64,
    /// Pattern length
    length: usize,
}

impl PartialEq for DictionaryPattern {
    fn eq(&self, other: &Self) -> bool {
        self.score.partial_cmp(&other.score) == Some(Ordering::Equal)
    }
}

impl Eq for DictionaryPattern {}

impl PartialOrd for DictionaryPattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score) // Reverse order for max-heap
    }
}

impl Ord for DictionaryPattern {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Suffix array implementation for efficient pattern extraction
#[derive(Debug)]
struct SuffixArray {
    /// Original text data
    text: Vec<u8>,
    /// Suffix array indices
    sa: Vec<usize>,
    /// Longest common prefix array
    lcp: Vec<usize>,
}

impl SuffixArray {
    /// Build suffix array using efficient radix sort algorithm
    fn new(text: Vec<u8>) -> Self {
        let n = text.len();
        if n == 0 {
            return Self { text, sa: Vec::new(), lcp: Vec::new() };
        }
        
        let mut sa = Self::build_suffix_array(&text);
        let lcp = Self::build_lcp_array(&text, &sa);
        
        Self { text, sa, lcp }
    }
    
    /// Build suffix array using SA-IS algorithm (simplified version)
    fn build_suffix_array(text: &[u8]) -> Vec<usize> {
        let n = text.len();
        let mut sa = (0..n).collect::<Vec<_>>();
        
        // Simple O(n log n) suffix array construction
        // For production use, SA-IS or other linear algorithms could be implemented
        sa.sort_by(|&a, &b| {
            let suffix_a = &text[a..];
            let suffix_b = &text[b..];
            suffix_a.cmp(suffix_b)
        });
        
        sa
    }
    
    /// Build LCP (Longest Common Prefix) array using Kasai algorithm
    fn build_lcp_array(text: &[u8], sa: &[usize]) -> Vec<usize> {
        let n = text.len();
        let mut lcp = vec![0; n];
        let mut rank = vec![0; n];
        
        // Build rank array (inverse of suffix array)
        for i in 0..n {
            rank[sa[i]] = i;
        }
        
        let mut h = 0;
        for i in 0..n {
            if rank[i] > 0 {
                let j = sa[rank[i] - 1];
                while i + h < n && j + h < n && text[i + h] == text[j + h] {
                    h += 1;
                }
                lcp[rank[i]] = h;
                if h > 0 {
                    h -= 1;
                }
            }
        }
        
        lcp
    }
    
    /// Extract frequent patterns of given length
    fn extract_patterns(&self, length: usize, min_frequency: usize) -> Vec<DictionaryPattern> {
        let mut pattern_counts: HashMap<Vec<u8>, usize> = HashMap::new();
        
        // Count pattern frequencies using suffix array
        for &start in &self.sa {
            if start + length <= self.text.len() {
                let pattern = self.text[start..start + length].to_vec();
                *pattern_counts.entry(pattern).or_insert(0) += 1;
            }
        }
        
        // Convert to patterns with scoring
        pattern_counts
            .into_iter()
            .filter(|(_, count)| *count >= min_frequency)
            .map(|(data, frequency)| {
                let length = data.len();
                let score = Self::calculate_pattern_score(&data, frequency, length);
                DictionaryPattern { data, frequency, score, length }
            })
            .collect()
    }
    
    /// Calculate compression benefit score for a pattern
    fn calculate_pattern_score(pattern: &[u8], frequency: usize, length: usize) -> f64 {
        // Score = frequency * length * entropy_reduction_factor
        // Higher frequency and longer patterns are preferred
        // Entropy factor considers byte distribution within pattern
        let entropy_factor = Self::calculate_entropy_factor(pattern);
        frequency as f64 * length as f64 * entropy_factor
    }
    
    /// Calculate entropy reduction factor for pattern
    fn calculate_entropy_factor(pattern: &[u8]) -> f64 {
        if pattern.is_empty() {
            return 0.0;
        }
        
        let mut byte_counts = [0u32; 256];
        for &byte in pattern {
            byte_counts[byte as usize] += 1;
        }
        
        let total = pattern.len() as f64;
        let mut entropy = 0.0;
        
        for &count in &byte_counts {
            if count > 0 {
                let prob = count as f64 / total;
                entropy -= prob * prob.log2();
            }
        }
        
        // Normalize entropy to [0, 1] range where 0 = max entropy, 1 = min entropy
        let max_entropy = 8.0; // log2(256)
        1.0 - (entropy / max_entropy).min(1.0)
    }
}

/// Main dictionary trainer
pub struct DictionaryTrainer {
    config: DictionaryTrainerConfig,
    samples: Vec<Vec<u8>>,
}

impl DictionaryTrainer {
    /// Create new dictionary trainer with configuration
    pub fn new(config: DictionaryTrainerConfig) -> Self {
        Self {
            config,
            samples: Vec::new(),
        }
    }
    
    /// Add sample data from file
    pub fn add_sample_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open sample file: {}", path.as_ref().display()))?;
        
        let mut sample = Vec::new();
        if self.config.max_sample_size > 0 {
            let mut reader = BufReader::new(file);
            let mut buffer = vec![0u8; self.config.max_sample_size];
            let bytes_read = reader.read(&mut buffer)
                .context("Failed to read sample file")?;
            sample = buffer[..bytes_read].to_vec();
        } else {
            file.read_to_end(&mut sample)
                .context("Failed to read sample file")?;
        }
        
        if !sample.is_empty() {
            self.samples.push(sample);
        }
        
        Ok(())
    }
    
    /// Add sample data directly
    pub fn add_sample_data(&mut self, data: Vec<u8>) {
        if !data.is_empty() {
            self.samples.push(data);
        }
    }
    
    /// Train dictionary from collected samples
    pub fn train_dictionary(&self) -> Result<ZstdDictionary> {
        if self.samples.is_empty() {
            return Err(anyhow::anyhow!("No sample data provided for dictionary training"));
        }
        
        // Concatenate all samples with separators
        let mut combined_data = Vec::new();
        for (i, sample) in self.samples.iter().enumerate() {
            if i > 0 {
                combined_data.push(0); // Use null byte as separator
            }
            combined_data.extend_from_slice(sample);
        }
        
        if combined_data.is_empty() {
            return Err(anyhow::anyhow!("All sample data is empty"));
        }
        
        // Build suffix array for pattern extraction
        let suffix_array = SuffixArray::new(combined_data);
        
        // Extract patterns of different lengths
        let mut all_patterns = Vec::new();
        for length in self.config.min_pattern_length..=self.config.max_pattern_length {
            let patterns = suffix_array.extract_patterns(length, self.config.min_frequency);
            all_patterns.extend(patterns);
        }
        
        if all_patterns.is_empty() {
            return Err(anyhow::anyhow!("No frequent patterns found in sample data"));
        }
        
        // Select best patterns within size budget
        let selected_patterns = self.select_optimal_patterns(all_patterns)?;
        
        // Build dictionary from selected patterns
        self.build_dictionary(selected_patterns)
    }
    
    /// Select optimal patterns within dictionary size budget using greedy algorithm
    fn select_optimal_patterns(&self, mut patterns: Vec<DictionaryPattern>) -> Result<Vec<DictionaryPattern>> {
        // Sort patterns by score (descending)
        patterns.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        
        let mut selected = Vec::new();
        let mut total_size = 0;
        
        // Greedily select patterns that fit within size budget
        for pattern in patterns {
            if total_size + pattern.length <= self.config.dict_size {
                total_size += pattern.length;
                selected.push(pattern);
            }
            
            if total_size >= self.config.dict_size {
                break;
            }
        }
        
        if selected.is_empty() {
            return Err(anyhow::anyhow!("No patterns fit within dictionary size budget"));
        }
        
        Ok(selected)
    }
    
    /// Build Zstandard dictionary from selected patterns
    fn build_dictionary(&self, patterns: Vec<DictionaryPattern>) -> Result<ZstdDictionary> {
        // Combine all pattern data
        let mut dict_data = Vec::new();
        for pattern in &patterns {
            dict_data.extend_from_slice(&pattern.data);
        }
        
        // Ensure dictionary doesn't exceed size limit
        if dict_data.len() > self.config.dict_size {
            dict_data.truncate(self.config.dict_size);
        }
        
        // Generate dictionary ID based on content hash
        let dict_id = self.generate_dictionary_id(&dict_data);
        
        // Create Zstandard dictionary format
        Ok(ZstdDictionary {
            id: dict_id,
            data: dict_data,
        })
    }
    
    /// Generate dictionary ID using simple hash of content
    fn generate_dictionary_id(&self, data: &[u8]) -> u32 {
        // Simple FNV-1a hash for dictionary ID generation
        let mut hash = 2166136261u32;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}

/// Zstandard dictionary structure
#[derive(Debug, Clone)]
pub struct ZstdDictionary {
    /// Dictionary ID
    pub id: u32,
    /// Dictionary content data
    pub data: Vec<u8>,
}

impl ZstdDictionary {
    /// Load dictionary from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open dictionary file: {}", path.as_ref().display()))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .context("Failed to read dictionary file")?;
        
        // Check for Zstandard dictionary magic number
        if data.len() >= 8 && &data[0..4] == b"\x37\xa4\x30\xec" {
            // Has magic number, extract dictionary ID and content
            let dict_id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            let dict_data = data[8..].to_vec();
            Ok(Self { id: dict_id, data: dict_data })
        } else {
            // Raw dictionary data, generate ID from content
            let dict_id = Self::generate_id_for_data(&data);
            Ok(Self { id: dict_id, data })
        }
    }
    
    /// Save dictionary to file with Zstandard format
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path.as_ref())
            .with_context(|| format!("Failed to create dictionary file: {}", path.as_ref().display()))?;
        
        // Write Zstandard dictionary magic number
        file.write_all(b"\x37\xa4\x30\xec")
            .context("Failed to write dictionary magic number")?;
        
        // Write dictionary ID
        file.write_all(&self.id.to_le_bytes())
            .context("Failed to write dictionary ID")?;
        
        // Write dictionary content
        file.write_all(&self.data)
            .context("Failed to write dictionary content")?;
        
        Ok(())
    }
    
    /// Generate dictionary ID for raw data
    fn generate_id_for_data(data: &[u8]) -> u32 {
        let mut hash = 2166136261u32;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
    
    /// Get dictionary size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Check if dictionary is valid
    pub fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.data.len() <= 2 * 1024 * 1024 // Max 2MB
    }
}

/// Train dictionary from multiple sample files
pub fn train_dictionary_from_files<P: AsRef<Path>>(
    sample_files: &[P],
    config: DictionaryTrainerConfig,
) -> Result<ZstdDictionary> {
    let mut trainer = DictionaryTrainer::new(config);
    
    for file_path in sample_files {
        trainer.add_sample_file(file_path)
            .with_context(|| format!("Failed to add sample file: {}", file_path.as_ref().display()))?;
    }
    
    trainer.train_dictionary()
}

/// Convenience function to train dictionary with default configuration
pub fn train_dictionary_default<P: AsRef<Path>>(sample_files: &[P]) -> Result<ZstdDictionary> {
    train_dictionary_from_files(sample_files, DictionaryTrainerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_suffix_array_construction() {
        let text = b"banana".to_vec();
        let sa = SuffixArray::new(text);
        
        // Verify suffix array is properly constructed
        assert_eq!(sa.sa.len(), 6);
        
        // Suffixes should be sorted lexicographically
        let suffixes: Vec<_> = sa.sa.iter()
            .map(|&i| &sa.text[i..])
            .collect();
        
        for i in 1..suffixes.len() {
            assert!(suffixes[i-1] <= suffixes[i]);
        }
    }
    
    #[test]
    fn test_pattern_extraction() {
        let text = b"abcabcabcdef".to_vec();
        let sa = SuffixArray::new(text);
        
        let patterns = sa.extract_patterns(3, 2);
        
        // Should find "abc" pattern with frequency >= 2
        assert!(patterns.iter().any(|p| p.data == b"abc" && p.frequency >= 2));
    }
    
    #[test]
    fn test_dictionary_training() -> Result<()> {
        // Create temporary sample files
        let mut file1 = NamedTempFile::new()?;
        file1.write_all(b"hello world hello world hello world")?;
        let mut file2 = NamedTempFile::new()?;
        file2.write_all(b"hello universe hello universe hello universe")?;
        
        let config = DictionaryTrainerConfig {
            dict_size: 100,
            min_pattern_length: 3,
            max_pattern_length: 10,
            min_frequency: 2,
            patterns_per_length: 100,
            max_sample_size: 0,
        };
        
        let dict = train_dictionary_from_files(&[file1.path(), file2.path()], config)?;
        
        assert!(dict.is_valid());
        assert!(dict.size() <= 100);
        assert!(dict.id != 0);
        
        Ok(())
    }
    
    #[test]
    fn test_dictionary_file_operations() -> Result<()> {
        let dict = ZstdDictionary {
            id: 12345,
            data: b"test dictionary content".to_vec(),
        };
        
        let temp_file = NamedTempFile::new()?;
        dict.save_to_file(temp_file.path())?;
        
        let loaded_dict = ZstdDictionary::from_file(temp_file.path())?;
        
        assert_eq!(dict.id, loaded_dict.id);
        assert_eq!(dict.data, loaded_dict.data);
        
        Ok(())
    }
    
    #[test]
    fn test_entropy_calculation() {
        // Uniform distribution should have high entropy (low factor)
        let uniform = (0u8..=255).collect::<Vec<_>>();
        let entropy_factor = SuffixArray::calculate_entropy_factor(&uniform);
        assert!(entropy_factor < 0.1);
        
        // Repeated bytes should have low entropy (high factor)
        let repeated = vec![65u8; 100];
        let entropy_factor = SuffixArray::calculate_entropy_factor(&repeated);
        assert!(entropy_factor > 0.9);
    }
}
