//! LLVM Core - Fundamental IR data structures
//!
//! This crate provides the core LLVM Intermediate Representation (IR) data structures
//! implemented in safe Rust. It includes:
//!
//! - Type system (integers, floats, pointers, arrays, structs, functions)
//! - Value hierarchy (constants, instructions, arguments)
//! - Basic blocks and functions
//! - Modules and contexts
//! - Metadata system
//!
//! # Example
//!
//! ```
//! use llvm_core::{Context, Type};
//!
//! let ctx = Context::new();
//! let i32_ty = ctx.i32_type();
//! let ptr_ty = Type::pointer(Type::i32(), 0);
//! ```

pub mod context;
pub mod metadata;
pub mod module;
pub mod types;
pub mod value;

// Re-export commonly used items
pub use context::Context;
pub use module::Module;
pub use types::Type;
pub use value::Value;

/// Type alias for LLVM results
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for LLVM operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid type: {0}")]
    InvalidType(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Context mismatch")]
    ContextMismatch,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
