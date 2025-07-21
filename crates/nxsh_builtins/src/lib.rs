//! Collection of built-in commands re-exported for convenient linking.

pub mod jobs;

pub use jobs::{fg, bg};

pub mod common;

pub use common::logging;

pub fn cd(path: &str) {
    println!("Changed directory to {path} (stub)");
} 