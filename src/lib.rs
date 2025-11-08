//! # LLVM-Rust
//!
//! A Rust implementation of LLVM's core components, providing a type-safe
//! and idiomatic Rust API for constructing and manipulating LLVM IR.
//!
//! This library aims to provide the fundamental building blocks of LLVM:
//! - Type system (integers, floats, pointers, arrays, structs, functions)
//! - Values and instructions
//! - Basic blocks and control flow
//! - Functions and modules
//! - IR builder for programmatic construction

pub mod types;
pub mod value;
pub mod instruction;
pub mod basic_block;
pub mod function;
pub mod module;
pub mod builder;
pub mod context;

pub use context::Context;
pub use types::Type;
pub use value::Value;
pub use instruction::Instruction;
pub use basic_block::BasicBlock;
pub use function::Function;
pub use module::Module;
pub use builder::Builder;
