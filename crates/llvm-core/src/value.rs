//! LLVM Value System
//!
//! This module defines the value hierarchy in LLVM IR:
//! - Constants (integers, floats, arrays, structs, etc.)
//! - Instructions (add, sub, load, store, etc.)
//! - Arguments (function parameters)
//! - Basic blocks
//! - Functions

use crate::types::Type;

/// A value in LLVM IR.
///
/// All instructions, constants, and arguments are values.
#[derive(Debug, Clone)]
pub struct Value {
    /// The type of this value
    ty: Type,
    /// The kind of value
    kind: ValueKind,
    /// Optional name for this value
    name: Option<String>,
}

impl Value {
    /// Creates a new value.
    pub fn new(ty: Type, kind: ValueKind) -> Self {
        Value {
            ty,
            kind,
            name: None,
        }
    }

    /// Creates a new value with a name.
    pub fn with_name(ty: Type, kind: ValueKind, name: String) -> Self {
        Value {
            ty,
            kind,
            name: Some(name),
        }
    }

    /// Returns the type of this value.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Returns the kind of this value.
    pub fn kind(&self) -> &ValueKind {
        &self.kind
    }

    /// Returns the name of this value, if it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the name of this value.
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Returns true if this is a constant.
    pub fn is_constant(&self) -> bool {
        matches!(self.kind, ValueKind::Constant(_))
    }

    /// Returns true if this is an instruction.
    pub fn is_instruction(&self) -> bool {
        matches!(self.kind, ValueKind::Instruction(_))
    }
}

/// The kind of value.
#[derive(Debug, Clone)]
pub enum ValueKind {
    /// A constant value
    Constant(Constant),
    /// An instruction
    Instruction(Instruction),
    /// A function argument
    Argument(usize), // argument index
    /// A basic block (as a value)
    BasicBlock,
    /// A function
    Function,
}

/// Constant values.
#[derive(Debug, Clone)]
pub enum Constant {
    /// Integer constant
    Int { value: i128, bit_width: u32 },
    /// Floating-point constant
    Float(f64),
    /// Null pointer constant
    Null,
    /// Undefined value
    Undef,
    /// Poison value (for optimization)
    Poison,
    /// Array constant
    Array(Vec<Box<Value>>),
    /// Struct constant
    Struct { fields: Vec<Box<Value>>, packed: bool },
    /// Vector constant
    Vector(Vec<Box<Value>>),
    /// Zero initializer
    ZeroInitializer,
}

impl Constant {
    /// Creates an integer constant.
    pub fn int(value: i128, bit_width: u32) -> Self {
        Constant::Int { value, bit_width }
    }

    /// Creates a floating-point constant.
    pub fn float(value: f64) -> Self {
        Constant::Float(value)
    }

    /// Creates a null pointer constant.
    pub fn null() -> Self {
        Constant::Null
    }

    /// Creates an undefined value.
    pub fn undef() -> Self {
        Constant::Undef
    }

    /// Creates a poison value.
    pub fn poison() -> Self {
        Constant::Poison
    }

    /// Creates a zero initializer.
    pub fn zero_initializer() -> Self {
        Constant::ZeroInitializer
    }

    /// Creates an i32 constant.
    pub fn i32(value: i32) -> Self {
        Constant::int(value as i128, 32)
    }

    /// Creates an i64 constant.
    pub fn i64(value: i64) -> Self {
        Constant::int(value as i128, 64)
    }
}

/// LLVM Instructions.
#[derive(Debug, Clone)]
pub enum Instruction {
    // Binary operations
    Add {
        lhs: Box<Value>,
        rhs: Box<Value>,
        nuw: bool, // no unsigned wrap
        nsw: bool, // no signed wrap
    },
    Sub {
        lhs: Box<Value>,
        rhs: Box<Value>,
        nuw: bool,
        nsw: bool,
    },
    Mul {
        lhs: Box<Value>,
        rhs: Box<Value>,
        nuw: bool,
        nsw: bool,
    },
    UDiv {
        lhs: Box<Value>,
        rhs: Box<Value>,
        exact: bool,
    },
    SDiv {
        lhs: Box<Value>,
        rhs: Box<Value>,
        exact: bool,
    },
    URem {
        lhs: Box<Value>,
        rhs: Box<Value>,
    },
    SRem {
        lhs: Box<Value>,
        rhs: Box<Value>,
    },

    // Bitwise operations
    And {
        lhs: Box<Value>,
        rhs: Box<Value>,
    },
    Or {
        lhs: Box<Value>,
        rhs: Box<Value>,
    },
    Xor {
        lhs: Box<Value>,
        rhs: Box<Value>,
    },
    Shl {
        lhs: Box<Value>,
        rhs: Box<Value>,
        nuw: bool,
        nsw: bool,
    },
    LShr {
        lhs: Box<Value>,
        rhs: Box<Value>,
        exact: bool,
    },
    AShr {
        lhs: Box<Value>,
        rhs: Box<Value>,
        exact: bool,
    },

    // Memory operations
    Alloca {
        allocated_type: Type,
        num_elements: Option<Box<Value>>,
        align: u32,
    },
    Load {
        pointer: Box<Value>,
        align: u32,
        volatile: bool,
    },
    Store {
        value: Box<Value>,
        pointer: Box<Value>,
        align: u32,
        volatile: bool,
    },
    GetElementPtr {
        base: Box<Value>,
        indices: Vec<Box<Value>>,
        in_bounds: bool,
    },

    // Conversion operations
    Trunc {
        value: Box<Value>,
        dest_ty: Type,
    },
    ZExt {
        value: Box<Value>,
        dest_ty: Type,
    },
    SExt {
        value: Box<Value>,
        dest_ty: Type,
    },
    FPToUI {
        value: Box<Value>,
        dest_ty: Type,
    },
    FPToSI {
        value: Box<Value>,
        dest_ty: Type,
    },
    UIToFP {
        value: Box<Value>,
        dest_ty: Type,
    },
    SIToFP {
        value: Box<Value>,
        dest_ty: Type,
    },
    PtrToInt {
        value: Box<Value>,
        dest_ty: Type,
    },
    IntToPtr {
        value: Box<Value>,
        dest_ty: Type,
    },
    BitCast {
        value: Box<Value>,
        dest_ty: Type,
    },

    // Comparison operations
    ICmp {
        predicate: IntPredicate,
        lhs: Box<Value>,
        rhs: Box<Value>,
    },
    FCmp {
        predicate: FloatPredicate,
        lhs: Box<Value>,
        rhs: Box<Value>,
    },

    // Control flow
    Br {
        dest: Box<Value>,
    },
    CondBr {
        cond: Box<Value>,
        true_dest: Box<Value>,
        false_dest: Box<Value>,
    },
    Switch {
        value: Box<Value>,
        default: Box<Value>,
        cases: Vec<(Constant, Box<Value>)>,
    },
    Ret {
        value: Option<Box<Value>>,
    },
    Unreachable,

    // Function calls
    Call {
        function: Box<Value>,
        args: Vec<Box<Value>>,
        tail_call: bool,
    },

    // Other
    Phi {
        incoming: Vec<(Box<Value>, Box<Value>)>, // (value, block)
    },
    Select {
        cond: Box<Value>,
        true_val: Box<Value>,
        false_val: Box<Value>,
    },
}

/// Integer comparison predicates.
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

/// Floating-point comparison predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatPredicate {
    False, // always false
    OEQ,   // ordered and equal
    OGT,   // ordered and greater than
    OGE,   // ordered and greater or equal
    OLT,   // ordered and less than
    OLE,   // ordered and less or equal
    ONE,   // ordered and not equal
    ORD,   // ordered (no NaNs)
    UEQ,   // unordered or equal
    UGT,   // unordered or greater than
    UGE,   // unordered or greater or equal
    ULT,   // unordered or less than
    ULE,   // unordered or less or equal
    UNE,   // unordered or not equal
    UNO,   // unordered (has NaNs)
    True,  // always true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_creation() {
        let c1 = Constant::i32(42);
        assert!(matches!(c1, Constant::Int { value: 42, bit_width: 32 }));

        let c2 = Constant::i64(100);
        assert!(matches!(c2, Constant::Int { value: 100, bit_width: 64 }));

        let c3 = Constant::float(3.14);
        assert!(matches!(c3, Constant::Float(_)));
    }

    #[test]
    fn test_value_creation() {
        let ty = Type::i32();
        let const_val = Constant::i32(42);
        let value = Value::new(ty, ValueKind::Constant(const_val));

        assert!(value.is_constant());
        assert!(!value.is_instruction());
    }

    #[test]
    fn test_value_with_name() {
        let ty = Type::i32();
        let const_val = Constant::i32(42);
        let value = Value::with_name(ty, ValueKind::Constant(const_val), "my_value".to_string());

        assert_eq!(value.name(), Some("my_value"));
    }
}
