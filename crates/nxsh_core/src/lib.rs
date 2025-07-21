#![doc = "Core runtime of NexusShell, responsible for context management, execution flow, and feature gates."]

pub mod context;
pub mod executor;
pub mod mir;
pub mod stream;
pub mod job;

#[cfg(feature = "jit")]
mod jit; // JIT compilation backend (Cranelift)

#[cfg(feature = "object-pipe")]
mod object_pipe; // Object pipeline implementation 