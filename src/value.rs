//! LLVM Values
//!
//! In LLVM, a Value represents any entity that can be used as an operand.
//! This includes constants, instructions, function arguments, etc.

use std::sync::Arc;
use std::fmt;
use crate::types::Type;

/// A value in LLVM IR
#[derive(Clone)]
pub struct Value {
    data: Arc<ValueData>,
}

pub(crate) struct ValueData {
    ty: Type,
    kind: ValueKind,
    name: Option<String>,
}

pub(crate) enum ValueKind {
    /// A constant integer value
    ConstantInt { value: i64 },
    /// A constant floating point value
    ConstantFloat { value: f64 },
    /// A constant null pointer
    ConstantNull,
    /// An undefined value
    Undef,
    /// A poison value
    Poison,
    /// A constant array
    ConstantArray { elements: Vec<Value> },
    /// A constant struct
    ConstantStruct { fields: Vec<Value> },
    /// A constant vector
    ConstantVector { elements: Vec<Value> },
    /// Zero initializer
    ZeroInitializer,
    /// A constant expression
    ConstantExpr { opcode: crate::instruction::Opcode, operands: Vec<Value> },
    /// A function argument
    Argument { index: usize },
    /// An instruction (reference to instruction data)
    Instruction { opcode: crate::instruction::Opcode },
    /// A basic block
    BasicBlock,
    /// A function
    Function,
    /// A global variable
    GlobalVariable { is_constant: bool },
    /// Block address
    BlockAddress { function: Box<Value>, block: Box<Value> },
}

impl Value {
    pub(crate) fn new(ty: Type, kind: ValueKind, name: Option<String>) -> Self {
        Self {
            data: Arc::new(ValueData { ty, kind, name }),
        }
    }

    /// Get the type of this value
    pub fn get_type(&self) -> &Type {
        &self.data.ty
    }

    /// Get the name of this value, if it has one
    pub fn name(&self) -> Option<&str> {
        self.data.name.as_deref()
    }

    /// Check if this value is a constant
    pub fn is_constant(&self) -> bool {
        matches!(&self.data.kind,
            ValueKind::ConstantInt { .. } |
            ValueKind::ConstantFloat { .. } |
            ValueKind::ConstantNull |
            ValueKind::Undef |
            ValueKind::Poison |
            ValueKind::ConstantArray { .. } |
            ValueKind::ConstantStruct { .. } |
            ValueKind::ConstantVector { .. } |
            ValueKind::ZeroInitializer |
            ValueKind::ConstantExpr { .. } |
            ValueKind::BlockAddress { .. }
        )
    }

    /// Check if this value is an instruction
    pub fn is_instruction(&self) -> bool {
        matches!(&self.data.kind, ValueKind::Instruction { .. })
    }

    // Constant constructors

    /// Create a constant integer value
    pub fn const_int(ty: Type, value: i64, name: Option<String>) -> Self {
        assert!(ty.is_integer(), "const_int requires an integer type");
        Self::new(ty, ValueKind::ConstantInt { value }, name)
    }

    /// Create a constant floating point value
    pub fn const_float(ty: Type, value: f64, name: Option<String>) -> Self {
        assert!(ty.is_float(), "const_float requires a floating point type");
        Self::new(ty, ValueKind::ConstantFloat { value }, name)
    }

    /// Create a null pointer constant
    pub fn const_null(ty: Type) -> Self {
        assert!(ty.is_pointer(), "const_null requires a pointer type");
        Self::new(ty, ValueKind::ConstantNull, None)
    }

    /// Create an undefined value
    pub fn undef(ty: Type) -> Self {
        Self::new(ty, ValueKind::Undef, None)
    }

    /// Create a poison value
    pub fn poison(ty: Type) -> Self {
        Self::new(ty, ValueKind::Poison, None)
    }

    /// Create a constant array
    pub fn const_array(ty: Type, elements: Vec<Value>) -> Self {
        assert!(ty.is_array(), "const_array requires an array type");
        Self::new(ty, ValueKind::ConstantArray { elements }, None)
    }

    /// Create a constant struct
    pub fn const_struct(ty: Type, fields: Vec<Value>) -> Self {
        assert!(ty.is_struct(), "const_struct requires a struct type");
        Self::new(ty, ValueKind::ConstantStruct { fields }, None)
    }

    /// Create a constant vector
    pub fn const_vector(ty: Type, elements: Vec<Value>) -> Self {
        assert!(ty.is_vector(), "const_vector requires a vector type");
        Self::new(ty, ValueKind::ConstantVector { elements }, None)
    }

    /// Create a zero initializer
    pub fn zero_initializer(ty: Type) -> Self {
        Self::new(ty, ValueKind::ZeroInitializer, None)
    }

    /// Create a constant expression
    pub fn const_expr(ty: Type, opcode: crate::instruction::Opcode, operands: Vec<Value>) -> Self {
        Self::new(ty, ValueKind::ConstantExpr { opcode, operands }, None)
    }

    /// Create a block address constant
    pub fn block_address(ty: Type, function: Value, block: Value) -> Self {
        assert!(ty.is_pointer(), "block_address requires a pointer type");
        Self::new(ty, ValueKind::BlockAddress {
            function: Box::new(function),
            block: Box::new(block),
        }, None)
    }

    /// Create a function argument value
    pub fn argument(ty: Type, index: usize, name: Option<String>) -> Self {
        Self::new(ty, ValueKind::Argument { index }, name)
    }

    pub(crate) fn instruction(ty: Type, opcode: crate::instruction::Opcode, name: Option<String>) -> Self {
        Self::new(ty, ValueKind::Instruction { opcode }, name)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data.kind {
            ValueKind::ConstantInt { value } => {
                if let Some(name) = &self.data.name {
                    write!(f, "%{} = {}", name, value)
                } else {
                    write!(f, "{}", value)
                }
            }
            ValueKind::ConstantFloat { value } => {
                if let Some(name) = &self.data.name {
                    write!(f, "%{} = {}", name, value)
                } else {
                    write!(f, "{}", value)
                }
            }
            ValueKind::ConstantNull => write!(f, "null"),
            ValueKind::Undef => write!(f, "undef"),
            ValueKind::Poison => write!(f, "poison"),
            ValueKind::ZeroInitializer => write!(f, "zeroinitializer"),
            ValueKind::ConstantArray { elements } => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", elem.get_type(), elem)?;
                }
                write!(f, "]")
            }
            ValueKind::ConstantStruct { fields } => {
                write!(f, "{{ ")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", field.get_type(), field)?;
                }
                write!(f, " }}")
            }
            ValueKind::ConstantVector { elements } => {
                write!(f, "<")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", elem.get_type(), elem)?;
                }
                write!(f, ">")
            }
            ValueKind::ConstantExpr { opcode, operands } => {
                write!(f, "{:?}(", opcode)?;
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", op.get_type(), op)?;
                }
                write!(f, ")")
            }
            ValueKind::BlockAddress { function, block } => {
                write!(f, "blockaddress({}, {})", function, block)
            }
            ValueKind::Argument { index } => {
                if let Some(name) = &self.data.name {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "%arg{}", index)
                }
            }
            ValueKind::Instruction { .. } => {
                if let Some(name) = &self.data.name {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "%tmp")
                }
            }
            ValueKind::BasicBlock => {
                if let Some(name) = &self.data.name {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "%bb")
                }
            }
            ValueKind::Function => {
                if let Some(name) = &self.data.name {
                    write!(f, "@{}", name)
                } else {
                    write!(f, "@func")
                }
            }
            ValueKind::GlobalVariable { .. } => {
                if let Some(name) = &self.data.name {
                    write!(f, "@{}", name)
                } else {
                    write!(f, "@global")
                }
            }
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value({}: {})", self, self.data.ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_constant_int() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let val = Value::const_int(i32_type, 42, Some("answer".to_string()));
        assert!(val.is_constant());
        assert_eq!(val.name(), Some("answer"));
    }

    #[test]
    fn test_constant_float() {
        let ctx = Context::new();
        let float_type = ctx.float_type();
        let val = Value::const_float(float_type, 3.14, None);
        assert!(val.is_constant());
    }

    #[test]
    fn test_null_pointer() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let ptr_type = ctx.ptr_type(i32_type);
        let val = Value::const_null(ptr_type);
        assert!(val.is_constant());
    }
}
