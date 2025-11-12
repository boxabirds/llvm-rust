//! Code Generation Infrastructure
//!
//! This module provides the code generation backend for translating LLVM IR
//! into machine code.

pub mod x86_64;
pub mod machine_instr;
pub mod register_allocator;
pub mod stack_frame;
pub mod elf;
pub mod external_functions;
pub mod runtime;

use crate::module::Module;
use crate::function::Function;

/// Target machine abstraction
pub trait TargetMachine {
    /// Generate assembly code for a module
    fn emit_assembly(&mut self, module: &Module) -> Result<String, CodegenError>;

    /// Generate machine code for a function
    fn emit_function(&mut self, function: &Function) -> Result<Vec<u8>, CodegenError>;
}

/// Code generation error types
#[derive(Debug, Clone)]
pub enum CodegenError {
    /// Unsupported instruction
    UnsupportedInstruction(String),
    /// Register allocation failed
    RegisterAllocationFailed(String),
    /// Invalid operand
    InvalidOperand(String),
    /// General error
    General(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UnsupportedInstruction(s) => write!(f, "Unsupported instruction: {}", s),
            CodegenError::RegisterAllocationFailed(s) => write!(f, "Register allocation failed: {}", s),
            CodegenError::InvalidOperand(s) => write!(f, "Invalid operand: {}", s),
            CodegenError::General(s) => write!(f, "Codegen error: {}", s),
        }
    }
}

impl std::error::Error for CodegenError {}
