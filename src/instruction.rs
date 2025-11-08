//! LLVM Instructions
//!
//! Instructions are the basic operations in LLVM IR, such as arithmetic,
//! memory access, control flow, etc.

use std::fmt;
use crate::value::Value;

/// Represents an LLVM instruction
#[derive(Clone)]
pub struct Instruction {
    opcode: Opcode,
    operands: Vec<Value>,
    result: Option<Value>,
}

/// Instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Terminator instructions
    Ret,
    Br,
    CondBr,
    Switch,
    IndirectBr,
    Unreachable,

    // Binary operations
    Add,
    Sub,
    Mul,
    UDiv,
    SDiv,
    URem,
    SRem,

    // Bitwise operations
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,

    // Floating point operations
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,

    // Memory operations
    Alloca,
    Load,
    Store,
    GetElementPtr,

    // Comparison operations
    ICmp,
    FCmp,

    // Conversion operations
    Trunc,
    ZExt,
    SExt,
    FPTrunc,
    FPExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
    PtrToInt,
    IntToPtr,
    BitCast,

    // Other operations
    PHI,
    Call,
    Select,
    ExtractValue,
    InsertValue,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(opcode: Opcode, operands: Vec<Value>, result: Option<Value>) -> Self {
        Self {
            opcode,
            operands,
            result,
        }
    }

    /// Get the opcode of this instruction
    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    /// Get the operands of this instruction
    pub fn operands(&self) -> &[Value] {
        &self.operands
    }

    /// Get the result value of this instruction, if any
    pub fn result(&self) -> Option<&Value> {
        self.result.as_ref()
    }

    /// Check if this is a terminator instruction
    pub fn is_terminator(&self) -> bool {
        matches!(self.opcode,
            Opcode::Ret |
            Opcode::Br |
            Opcode::CondBr |
            Opcode::Switch |
            Opcode::IndirectBr |
            Opcode::Unreachable
        )
    }

    /// Check if this is a binary operation
    pub fn is_binary_op(&self) -> bool {
        matches!(self.opcode,
            Opcode::Add | Opcode::Sub | Opcode::Mul |
            Opcode::UDiv | Opcode::SDiv | Opcode::URem | Opcode::SRem |
            Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul |
            Opcode::FDiv | Opcode::FRem
        )
    }

    /// Check if this is a memory operation
    pub fn is_memory_op(&self) -> bool {
        matches!(self.opcode,
            Opcode::Alloca | Opcode::Load | Opcode::Store | Opcode::GetElementPtr
        )
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(result) = &self.result {
            write!(f, "{} = ", result)?;
        }

        write!(f, "{:?}", self.opcode)?;

        if !self.operands.is_empty() {
            write!(f, " ")?;
            for (i, operand) in self.operands.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", operand)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instruction({:?}, {} operands)", self.opcode, self.operands.len())
    }
}

/// Integer comparison predicates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntPredicate {
    EQ,  // equal
    NE,  // not equal
    UGT, // unsigned greater than
    UGE, // unsigned greater or equal
    ULT, // unsigned less than
    ULE, // unsigned less or equal
    SGT, // signed greater than
    SGE, // signed greater or equal
    SLT, // signed less than
    SLE, // signed less or equal
}

/// Floating point comparison predicates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPredicate {
    OEQ, // ordered and equal
    OGT, // ordered and greater than
    OGE, // ordered and greater than or equal
    OLT, // ordered and less than
    OLE, // ordered and less than or equal
    ONE, // ordered and not equal
    ORD, // ordered (no NaNs)
    UNO, // unordered (has NaNs)
    UEQ, // unordered or equal
    UGT, // unordered or greater than
    UGE, // unordered or greater than or equal
    ULT, // unordered or less than
    ULE, // unordered or less than or equal
    UNE, // unordered or not equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_is_terminator() {
        let ret_inst = Instruction::new(Opcode::Ret, vec![], None);
        assert!(ret_inst.is_terminator());

        let add_inst = Instruction::new(Opcode::Add, vec![], None);
        assert!(!add_inst.is_terminator());
    }

    #[test]
    fn test_instruction_is_binary_op() {
        let add_inst = Instruction::new(Opcode::Add, vec![], None);
        assert!(add_inst.is_binary_op());

        let ret_inst = Instruction::new(Opcode::Ret, vec![], None);
        assert!(!ret_inst.is_binary_op());
    }
}
