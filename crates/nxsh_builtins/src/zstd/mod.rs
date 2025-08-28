//! Zstandard compression module for NexusShell
//! 
//! This module provides Pure Rust implementation of Zstandard compression
//! with dictionary support and training capabilities.

pub mod zstd_impl;
pub mod dictionary_trainer;

#[cfg(test)]
pub mod dictionary_trainer_tests;

// Re-export public types
pub use dictionary_trainer::{DictionaryTrainer, DictionaryTrainerConfig, ZstdDictionary};
