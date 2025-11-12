//! x86-64 Calling Conventions
//!
//! Implements the System V AMD64 ABI calling convention.

use super::registers::X86Register;

/// System V AMD64 ABI calling convention
pub struct SystemVCallingConvention;

impl SystemVCallingConvention {
    /// Get the register for integer argument N
    pub fn int_arg_register(n: usize) -> Option<X86Register> {
        match n {
            0 => Some(X86Register::RDI),
            1 => Some(X86Register::RSI),
            2 => Some(X86Register::RDX),
            3 => Some(X86Register::RCX),
            4 => Some(X86Register::R8),
            5 => Some(X86Register::R9),
            _ => None, // Remaining arguments go on stack
        }
    }

    /// Get the return value register
    pub fn return_register() -> X86Register {
        X86Register::RAX
    }

    /// Get the stack pointer register
    pub fn stack_pointer() -> X86Register {
        X86Register::RSP
    }

    /// Get the base pointer register
    pub fn base_pointer() -> X86Register {
        X86Register::RBP
    }

    /// Check if a register needs to be preserved across calls
    pub fn is_callee_saved(reg: X86Register) -> bool {
        reg.is_callee_saved()
    }

    /// Get all callee-saved registers that need to be preserved
    pub fn callee_saved_registers() -> Vec<X86Register> {
        vec![
            X86Register::RBX,
            X86Register::RBP,
            X86Register::R12,
            X86Register::R13,
            X86Register::R14,
            X86Register::R15,
        ]
    }
}
