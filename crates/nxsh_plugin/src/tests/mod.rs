//! Plugin System Tests
//!
//! This module contains comprehensive tests for the NexusShell plugin system,
//! including native plugins, WASI plugins, and hybrid scenarios.

pub mod wasi_integration;

// Re-export test utilities for other modules
pub use wasi_integration::*;
