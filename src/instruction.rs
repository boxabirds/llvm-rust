//! LLVM Instructions
//!
//! Instructions are the basic operations in LLVM IR, such as arithmetic,
//! memory access, control flow, etc.

use std::fmt;
use crate::value::Value;
use crate::types::Type;

/// Represents an LLVM instruction
#[derive(Clone)]
pub struct Instruction {
    opcode: Opcode,
    operands: Vec<Value>,
    result: Option<Value>,
    metadata_attachments: Vec<String>, // e.g., ["dbg", "llvm.access.group", "align"]

    // GEP-specific metadata
    gep_source_type: Option<Type>,

    // Atomic/volatile flags
    is_atomic: bool,
    is_volatile: bool,
    atomic_ordering: Option<AtomicOrdering>,
}

/// Instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Terminator instructions (7)
    Ret,
    Br,
    CondBr,
    Switch,
    IndirectBr,
    Invoke,
    Resume,
    Unreachable,
    CleanupRet,
    CatchRet,
    CatchSwitch,
    CallBr,

    // Unary operations (1)
    FNeg,

    // Binary operations (28)
    Add,
    FAdd,
    Sub,
    FSub,
    Mul,
    FMul,
    UDiv,
    SDiv,
    FDiv,
    URem,
    SRem,
    FRem,

    // Bitwise binary operations (6)
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,

    // Vector operations (3)
    ExtractElement,
    InsertElement,
    ShuffleVector,

    // Aggregate operations (4)
    ExtractValue,
    InsertValue,

    // Memory addressing and access (5)
    Alloca,
    Load,
    Store,
    GetElementPtr,
    Fence,
    AtomicCmpXchg,
    AtomicRMW,

    // Conversion operations (14)
    Trunc,
    ZExt,
    SExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
    FPTrunc,
    FPExt,
    PtrToInt,
    IntToPtr,
    PtrToAddr,
    AddrToPtr,
    BitCast,
    AddrSpaceCast,

    // Other operations (11)
    ICmp,
    FCmp,
    PHI,
    Call,
    Select,
    UserOp1,
    UserOp2,
    VAArg,
    LandingPad,
    CleanupPad,
    CatchPad,
    Freeze,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(opcode: Opcode, operands: Vec<Value>, result: Option<Value>) -> Self {
        Self {
            opcode,
            operands,
            result,
            metadata_attachments: Vec::new(),
            gep_source_type: None,
            is_atomic: false,
            is_volatile: false,
            atomic_ordering: None,
        }
    }

    /// Add a metadata attachment
    pub fn add_metadata_attachment(&mut self, name: String) {
        self.metadata_attachments.push(name);
    }

    /// Get metadata attachments
    pub fn metadata_attachments(&self) -> &[String] {
        &self.metadata_attachments
    }

    /// Check if instruction has a specific metadata attachment
    pub fn has_metadata(&self, name: &str) -> bool {
        self.metadata_attachments.iter().any(|m| m == name)
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
            Opcode::Invoke |
            Opcode::Resume |
            Opcode::Unreachable |
            Opcode::CleanupRet |
            Opcode::CatchRet |
            Opcode::CatchSwitch |
            Opcode::CallBr
        )
    }

    /// Check if this is a binary operation
    pub fn is_binary_op(&self) -> bool {
        matches!(self.opcode,
            Opcode::Add | Opcode::FAdd |
            Opcode::Sub | Opcode::FSub |
            Opcode::Mul | Opcode::FMul |
            Opcode::UDiv | Opcode::SDiv | Opcode::FDiv |
            Opcode::URem | Opcode::SRem | Opcode::FRem |
            Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor
        )
    }

    /// Check if this is a unary operation
    pub fn is_unary_op(&self) -> bool {
        matches!(self.opcode, Opcode::FNeg)
    }

    /// Check if this is a memory operation
    pub fn is_memory_op(&self) -> bool {
        matches!(self.opcode,
            Opcode::Alloca | Opcode::Load | Opcode::Store |
            Opcode::GetElementPtr | Opcode::Fence |
            Opcode::AtomicCmpXchg | Opcode::AtomicRMW
        )
    }

    /// Check if this is a cast/conversion operation
    pub fn is_cast(&self) -> bool {
        matches!(self.opcode,
            Opcode::Trunc | Opcode::ZExt | Opcode::SExt |
            Opcode::FPToUI | Opcode::FPToSI | Opcode::UIToFP | Opcode::SIToFP |
            Opcode::FPTrunc | Opcode::FPExt |
            Opcode::PtrToInt | Opcode::IntToPtr |
            Opcode::BitCast | Opcode::AddrSpaceCast
        )
    }

    /// Check if this is a comparison operation
    pub fn is_comparison(&self) -> bool {
        matches!(self.opcode, Opcode::ICmp | Opcode::FCmp)
    }

    /// Check if this is a vector operation
    pub fn is_vector_op(&self) -> bool {
        matches!(self.opcode,
            Opcode::ExtractElement | Opcode::InsertElement | Opcode::ShuffleVector
        )
    }

    /// Check if this is an aggregate operation
    pub fn is_aggregate_op(&self) -> bool {
        matches!(self.opcode, Opcode::ExtractValue | Opcode::InsertValue)
    }

    // GEP-specific accessors

    /// Set the GEP source type for GetElementPtr instructions
    pub fn set_gep_source_type(&mut self, ty: Type) {
        self.gep_source_type = Some(ty);
    }

    /// Get the GEP source type if this is a GetElementPtr instruction
    pub fn gep_source_type(&self) -> Option<&Type> {
        self.gep_source_type.as_ref()
    }

    // Atomic/volatile accessors

    /// Set the atomic flag
    pub fn set_atomic(&mut self, is_atomic: bool) {
        self.is_atomic = is_atomic;
    }

    /// Check if this is an atomic operation
    pub fn is_atomic(&self) -> bool {
        self.is_atomic
    }

    /// Set the volatile flag
    pub fn set_volatile(&mut self, is_volatile: bool) {
        self.is_volatile = is_volatile;
    }

    /// Check if this is a volatile operation
    pub fn is_volatile(&self) -> bool {
        self.is_volatile
    }

    /// Set the atomic ordering
    pub fn set_atomic_ordering(&mut self, ordering: AtomicOrdering) {
        self.atomic_ordering = Some(ordering);
    }

    /// Get the atomic ordering
    pub fn atomic_ordering(&self) -> Option<AtomicOrdering> {
        self.atomic_ordering
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

/// Atomic ordering constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicOrdering {
    NotAtomic,
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}

/// Atomic read-modify-write operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicRMWBinOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    UMax,
    UMin,
    FAdd,
    FSub,
}

/// Fast math flags for floating point operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastMathFlags {
    pub allow_reassoc: bool,
    pub no_nans: bool,
    pub no_infs: bool,
    pub no_signed_zeros: bool,
    pub allow_reciprocal: bool,
    pub allow_contract: bool,
    pub approx_func: bool,
}

impl Default for FastMathFlags {
    fn default() -> Self {
        Self {
            allow_reassoc: false,
            no_nans: false,
            no_infs: false,
            no_signed_zeros: false,
            allow_reciprocal: false,
            allow_contract: false,
            approx_func: false,
        }
    }
}

impl FastMathFlags {
    pub fn fast() -> Self {
        Self {
            allow_reassoc: true,
            no_nans: true,
            no_infs: true,
            no_signed_zeros: true,
            allow_reciprocal: true,
            allow_contract: true,
            approx_func: true,
        }
    }
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
