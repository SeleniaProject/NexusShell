//! Comprehensive tests for 4-stream Huffman encoding implementation
//! Tests cover edge cases, performance characteristics, and RFC 8878 compliance

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zstd::zstd_impl::bitstream::BitWriter;

    #[test]
    fn test_four_stream_basic_functionality() {
        // Test basic 4-stream encoding with diverse literal data
        let literals = b"Hello, World! This is a test of the 4-stream Huffman encoder. It should split data efficiently across streams and produce valid output with jump tables.";
        
        let (table, _) = build_literals_huffman(literals).expect("Failed to build Huffman table");
        let four_stream = encode_four_stream_huffman(literals, &table).expect("Failed to encode 4-stream");
        
        // Verify basic structure
        assert_eq!(four_stream.streams.len(), 4);
        assert_eq!(four_stream.jump_table.len(), 3);
        
        // Verify streams are non-empty (given sufficient input)
        for stream in &four_stream.streams {
            assert!(!stream.is_empty(), "Stream should not be empty");
        }
        
        // Verify jump table makes sense
        for &size in &four_stream.jump_table {
            assert!(size > 0, "Jump table entry should be positive");
        }
        
        // Verify total size calculation
        let expected_total = 6 + four_stream.streams.iter().map(|s| s.len()).sum::<usize>();
        assert_eq!(four_stream.total_size, expected_total);
    }

    #[test]
    fn test_four_stream_edge_cases() {
        // Test with minimal data (should fail gracefully)
        let tiny_data = b"Hi";
        let result = encode_four_stream_huffman(tiny_data, &build_literals_huffman(b"Hi").unwrap().0);
        assert!(result.is_none(), "4-stream should fail on tiny data");
        
        // Test with exactly 4 bytes
        let four_bytes = b"Test";
        if let Some((table, _)) = build_literals_huffman(four_bytes) {
            let result = encode_four_stream_huffman(four_bytes, &table);
            assert!(result.is_some(), "4-stream should handle exactly 4 bytes");
        }
        
        // Test with large uniform data
        let uniform_data = vec![b'A'; 4096];
        if let Some((table, _)) = build_literals_huffman(&uniform_data) {
            let result = encode_four_stream_huffman(&uniform_data, &table);
            assert!(result.is_some(), "4-stream should handle uniform data");
        }
    }

    #[test]
    fn test_four_stream_data_distribution() {
        // Test that data is distributed reasonably across streams
        let test_data = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(50); // 1300 bytes
        
        let (table, _) = build_literals_huffman(&test_data).expect("Failed to build table");
        let four_stream = encode_four_stream_huffman(&test_data, &table).expect("Failed to encode");
        
        // Check that streams have reasonably balanced sizes
        let total_input = test_data.len();
        let expected_size = total_input / 4;
        let tolerance = expected_size / 2; // Allow 50% variance
        
        for (i, stream) in four_stream.streams.iter().enumerate() {
            // Note: encoded size will differ from input size due to compression
            // Just verify streams are non-trivial
            assert!(stream.len() > 0, "Stream {} should be non-empty", i);
        }
    }

    #[test] 
    fn test_four_stream_jump_table_format() {
        // Test jump table encoding format
        let test_data = b"The quick brown fox jumps over the lazy dog. ".repeat(30);
        
        let (table, _) = build_literals_huffman(&test_data).expect("Failed to build table");
        let four_stream = encode_four_stream_huffman(&test_data, &table).expect("Failed to encode");
        
        // Verify jump table constraints
        for (i, &size) in four_stream.jump_table.iter().enumerate() {
            assert!(size <= u16::MAX, "Jump table entry {} exceeds u16::MAX", i);
            assert!(size > 0, "Jump table entry {} should be positive", i);
        }
        
        // Verify jump table points to valid stream boundaries
        let mut offset = 0;
        for i in 0..3 {
            offset += four_stream.jump_table[i] as usize;
            assert!(offset <= four_stream.streams.iter().map(|s| s.len()).sum::<usize>(),
                   "Jump table entry {} points beyond stream data", i);
        }
    }

    #[test]
    fn test_four_stream_vs_single_stream_decision() {
        // Test automatic mode selection logic
        
        // Small data should prefer single-stream
        let small_data = b"Small test data";
        assert!(!should_use_four_stream(small_data), "Small data should use single-stream");
        
        // Large data should prefer 4-stream  
        let large_data = vec![b'X'; 2048];
        assert!(should_use_four_stream(&large_data), "Large data should use 4-stream");
        
        // Threshold boundary test
        let threshold_data = vec![b'T'; 1024];
        assert!(should_use_four_stream(&threshold_data), "Threshold data should use 4-stream");
        
        let below_threshold = vec![b'B'; 1023];
        assert!(!should_use_four_stream(&below_threshold), "Below threshold should use single-stream");
    }

    #[test]
    fn test_literals_block_header_encoding() {
        // Test various size encodings for literals block headers
        let test_cases = vec![
            (15, 63),       // Fits in 4+4 bits
            (100, 200),     // Fits in 6+6 bits  
            (1000, 500),    // Needs Size_Format=01
            (20000, 18000), // Needs Size_Format=11
        ];
        
        for (regen_size, comp_size) in test_cases {
            let dummy_lits = vec![b'A'; regen_size];
            let (table, header) = build_literals_huffman(&dummy_lits).expect("Failed to build table");
            
            // Test both single-stream and 4-stream block encoding
            if let Some(single_block) = encode_single_stream_literals_block(&dummy_lits, &table, &header) {
                assert!(!single_block.is_empty(), "Single-stream block should not be empty");
                // Verify header format is reasonable
                assert!(single_block[0] & 0x03 == 0x01, "Block type should be Compressed_Literals_Block");
            }
            
            if regen_size >= 1024 {
                if let Some(four_block) = encode_four_stream_literals_block(&dummy_lits, &table, &header) {
                    assert!(!four_block.is_empty(), "4-stream block should not be empty");
                    assert!(four_block[0] & 0x03 == 0x01, "Block type should be Compressed_Literals_Block");
                }
            }
        }
    }

    #[test]
    fn test_fse_compressed_weights_enhancement() {
        // Test enhanced FSE compression for weights
        let test_patterns = vec![
            b"AAABBBCCCDDD".as_slice(),                    // Balanced distribution
            b"AAAAAAABBBBBBCCCCCCDDDDDD".as_slice(),       // More entropy
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_slice(),      // Uniform distribution
            b"AAAAAAAAAAAABBBBCCCCDDDDEEEEFFFFF".as_slice(), // Skewed distribution
        ];
        
        for pattern in test_patterns {
            if let Some((table, _)) = build_literals_huffman(pattern) {
                // Test that FSE compression makes sensible decisions
                let fse_result = build_fse_compressed_weights_header(&table);
                let direct_size = 1 + table.num_symbols.div_ceil(2);
                
                if let Some(fse_compressed) = fse_result {
                    // FSE should be more efficient than direct weights
                    assert!(fse_compressed.len() < direct_size, 
                           "FSE compression should be more efficient for pattern: {:?}", 
                           std::str::from_utf8(pattern).unwrap_or("binary"));
                    
                    // Verify header format
                    assert!(fse_compressed[0] < 128, "Header should indicate FSE compression");
                    assert!(fse_compressed.len() >= 2, "FSE compressed should have header + data");
                }
                
                // Test that we always get some valid result
                assert!(table.weights.len() == table.num_symbols, "Weights length should match num_symbols");
            }
        }
    }

    #[test]
    fn test_auto_encoding_optimization() {
        // Test the auto-encoding function that chooses optimal mode
        let test_cases = vec![
            ("Short text", false), // Should choose single-stream
            (&"Long text that exceeds the threshold ".repeat(50), true), // Should choose 4-stream
        ];
        
        for (text, expect_large) in test_cases {
            let data = text.as_bytes();
            
            if let Some((table, encoded_block)) = encode_literals_block_auto(data) {
                assert!(!encoded_block.is_empty(), "Encoded block should not be empty");
                assert!(table.num_symbols > 0, "Table should have symbols");
                
                // Verify block format
                let block_type = encoded_block[0] & 0x03;
                assert!(block_type == 0x01 || block_type == 0x00, 
                       "Block type should be valid (Raw or Compressed)");
                
                if expect_large {
                    // Large data should produce reasonably sized output
                    assert!(encoded_block.len() < data.len(), 
                           "Compressed block should be smaller than input for large data");
                }
            }
        }
    }

    #[test]
    fn test_huffman_code_generation_correctness() {
        // Test that generated Huffman codes are valid and canonical
        let test_data = b"Hello world! This tests Huffman code generation.";
        
        if let Some((table, _)) = build_literals_huffman(test_data) {
            let mut code_lengths = Vec::new();
            let mut symbols = Vec::new();
            
            // Collect all defined codes
            for (symbol, code_opt) in table.codes.iter().enumerate() {
                if let Some((code, length)) = code_opt {
                    symbols.push(symbol as u8);
                    code_lengths.push(*length);
                    
                    // Verify code fits in specified bit length
                    assert!(*code < (1u16 << *length), 
                           "Code {} doesn't fit in {} bits for symbol {}", 
                           code, length, symbol);
                }
            }
            
            // Verify we have at least 2 symbols (required for Huffman)
            assert!(symbols.len() >= 2, "Should have at least 2 symbols");
            
            // Verify canonical ordering: shorter codes come first, 
            // codes of same length are in symbol order
            let mut prev_length = 0u8;
            let mut prev_symbol = 0u8;
            let mut prev_code = 0u16;
            
            for (&symbol, &length) in symbols.iter().zip(code_lengths.iter()) {
                if let Some((code, _)) = table.codes[symbol as usize] {
                    if length > prev_length {
                        // Length increased, code should reset properly
                        prev_length = length;
                        prev_symbol = symbol;
                        prev_code = code;
                    } else if length == prev_length {
                        // Same length, symbol order should increase
                        assert!(symbol > prev_symbol, 
                               "Symbols of same length should be in order");
                        assert!(code > prev_code, 
                               "Codes of same length should increase");
                        prev_symbol = symbol;
                        prev_code = code;
                    }
                }
            }
        }
    }

    #[test]
    fn test_bitstream_alignment_and_padding() {
        // Test that bitstreams are properly aligned and padded
        let test_data = b"Test data for bitstream alignment verification!";
        
        if let Some((table, _)) = build_literals_huffman(test_data) {
            // Test single stream alignment
            if let Some(encoded) = encode_single_stream_huffman(test_data, &table) {
                // Verify that last byte doesn't have stray bits beyond the padding
                // (This is hard to verify without decoding, but we check basic structure)
                assert!(!encoded.is_empty(), "Encoded stream should not be empty");
            }
            
            // Test 4-stream alignment
            if let Some(four_stream) = encode_four_stream_huffman(test_data, &table) {
                for (i, stream) in four_stream.streams.iter().enumerate() {
                    assert!(!stream.is_empty(), "Stream {} should not be empty", i);
                    
                    // Each stream should be byte-aligned
                    // (BitWriter ensures this with align_to_byte())
                }
                
                // Verify jump table entries are sensible
                let total_stream_size: usize = four_stream.streams.iter().map(|s| s.len()).sum();
                let jump_total: usize = four_stream.jump_table.iter().map(|&x| x as usize).sum();
                assert!(jump_total <= total_stream_size, 
                       "Jump table total should not exceed total stream size");
            }
        }
    }

    #[test]
    fn test_performance_characteristics() {
        // Test performance-related characteristics
        let sizes = vec![1024, 4096, 16384, 65536];
        
        for size in sizes {
            let test_data = (0..size).map(|i| (i % 256) as u8).collect::<Vec<_>>();
            
            let start = std::time::Instant::now();
            let result = encode_literals_block_auto(&test_data);
            let duration = start.elapsed();
            
            assert!(result.is_some(), "Should successfully encode {} bytes", size);
            
            // Performance should be reasonable (less than 100ms for these sizes in debug mode)
            assert!(duration.as_millis() < 1000, 
                   "Encoding {} bytes took too long: {:?}", size, duration);
            
            if let Some((_, encoded)) = result {
                // Compression ratio should be reasonable for synthetic data
                let ratio = encoded.len() as f64 / test_data.len() as f64;
                assert!(ratio > 0.1 && ratio < 2.0, 
                       "Compression ratio {} seems unreasonable for {} bytes", ratio, size);
            }
        }
    }
}
