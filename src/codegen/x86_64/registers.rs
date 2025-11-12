//! x86-64 Register Definitions

use std::fmt;

/// x86-64 general purpose registers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86Register {
    // 64-bit registers
    RAX, RBX, RCX, RDX,
    RSI, RDI, RBP, RSP,
    R8, R9, R10, R11,
    R12, R13, R14, R15,

    // 32-bit registers
    EAX, EBX, ECX, EDX,
    ESI, EDI, EBP, ESP,

    // 16-bit registers
    AX, BX, CX, DX,

    // 8-bit registers
    AL, BL, CL, DL,
}

impl X86Register {
    /// Check if this is a caller-saved register (System V ABI)
    pub fn is_caller_saved(&self) -> bool {
        matches!(self,
            X86Register::RAX | X86Register::RCX | X86Register::RDX |
            X86Register::RSI | X86Register::RDI |
            X86Register::R8 | X86Register::R9 | X86Register::R10 | X86Register::R11
        )
    }

    /// Check if this is a callee-saved register (System V ABI)
    pub fn is_callee_saved(&self) -> bool {
        matches!(self,
            X86Register::RBX | X86Register::RBP |
            X86Register::R12 | X86Register::R13 | X86Register::R14 | X86Register::R15
        )
    }

    /// Get all allocatable general-purpose registers
    pub fn allocatable_gp_regs() -> Vec<X86Register> {
        vec![
            X86Register::RAX, X86Register::RBX, X86Register::RCX, X86Register::RDX,
            X86Register::RSI, X86Register::RDI,
            X86Register::R8, X86Register::R9, X86Register::R10, X86Register::R11,
            X86Register::R12, X86Register::R13, X86Register::R14, X86Register::R15,
        ]
    }

    /// Get argument registers (System V ABI)
    pub fn argument_registers() -> Vec<X86Register> {
        vec![
            X86Register::RDI,
            X86Register::RSI,
            X86Register::RDX,
            X86Register::RCX,
            X86Register::R8,
            X86Register::R9,
        ]
    }
}

impl fmt::Display for X86Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            X86Register::RAX => "%rax",
            X86Register::RBX => "%rbx",
            X86Register::RCX => "%rcx",
            X86Register::RDX => "%rdx",
            X86Register::RSI => "%rsi",
            X86Register::RDI => "%rdi",
            X86Register::RBP => "%rbp",
            X86Register::RSP => "%rsp",
            X86Register::R8 => "%r8",
            X86Register::R9 => "%r9",
            X86Register::R10 => "%r10",
            X86Register::R11 => "%r11",
            X86Register::R12 => "%r12",
            X86Register::R13 => "%r13",
            X86Register::R14 => "%r14",
            X86Register::R15 => "%r15",
            X86Register::EAX => "%eax",
            X86Register::EBX => "%ebx",
            X86Register::ECX => "%ecx",
            X86Register::EDX => "%edx",
            X86Register::ESI => "%esi",
            X86Register::EDI => "%edi",
            X86Register::EBP => "%ebp",
            X86Register::ESP => "%esp",
            X86Register::AX => "%ax",
            X86Register::BX => "%bx",
            X86Register::CX => "%cx",
            X86Register::DX => "%dx",
            X86Register::AL => "%al",
            X86Register::BL => "%bl",
            X86Register::CL => "%cl",
            X86Register::DL => "%dl",
        };
        write!(f, "{}", name)
    }
}

/// Register class for register allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterClass {
    /// General purpose 64-bit
    GPR64,
    /// General purpose 32-bit
    GPR32,
    /// General purpose 16-bit
    GPR16,
    /// General purpose 8-bit
    GPR8,
}

impl X86Register {
    /// Get the register class
    pub fn register_class(&self) -> RegisterClass {
        match self {
            X86Register::RAX | X86Register::RBX | X86Register::RCX | X86Register::RDX |
            X86Register::RSI | X86Register::RDI | X86Register::RBP | X86Register::RSP |
            X86Register::R8 | X86Register::R9 | X86Register::R10 | X86Register::R11 |
            X86Register::R12 | X86Register::R13 | X86Register::R14 | X86Register::R15 => RegisterClass::GPR64,
            X86Register::EAX | X86Register::EBX | X86Register::ECX | X86Register::EDX |
            X86Register::ESI | X86Register::EDI | X86Register::EBP | X86Register::ESP => RegisterClass::GPR32,
            X86Register::AX | X86Register::BX | X86Register::CX | X86Register::DX => RegisterClass::GPR16,
            X86Register::AL | X86Register::BL | X86Register::CL | X86Register::DL => RegisterClass::GPR8,
        }
    }
}
