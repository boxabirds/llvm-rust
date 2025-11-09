//! LLVM Functions
//!
//! Functions are the top-level callable entities in LLVM IR.
//! They consist of a signature (return type and parameters) and
//! a body (basic blocks).

use std::sync::{Arc, RwLock};
use std::fmt;
use crate::types::Type;
use crate::basic_block::BasicBlock;
use crate::value::Value;

/// A function in LLVM IR
#[derive(Clone)]
pub struct Function {
    data: Arc<RwLock<FunctionData>>,
}

struct FunctionData {
    name: String,
    ty: Type,
    basic_blocks: Vec<BasicBlock>,
    arguments: Vec<Value>,
}

impl Function {
    /// Create a new function
    pub fn new(name: String, ty: Type) -> Self {
        assert!(ty.is_function(), "Function must have function type");

        Self {
            data: Arc::new(RwLock::new(FunctionData {
                name,
                ty,
                basic_blocks: Vec::new(),
                arguments: Vec::new(),
            })),
        }
    }

    /// Get the name of this function
    pub fn name(&self) -> String {
        self.data.read().unwrap().name.clone()
    }

    /// Get the type of this function
    pub fn get_type(&self) -> Type {
        self.data.read().unwrap().ty.clone()
    }

    /// Add a basic block to this function
    pub fn add_basic_block(&self, bb: BasicBlock) {
        let mut data = self.data.write().unwrap();
        data.basic_blocks.push(bb);
    }

    /// Get the basic blocks in this function
    pub fn basic_blocks(&self) -> Vec<BasicBlock> {
        self.data.read().unwrap().basic_blocks.clone()
    }

    /// Get the entry basic block
    pub fn entry_block(&self) -> Option<BasicBlock> {
        self.data.read().unwrap().basic_blocks.first().cloned()
    }

    /// Get the number of basic blocks in this function
    pub fn basic_block_count(&self) -> usize {
        self.data.read().unwrap().basic_blocks.len()
    }

    /// Set the function arguments
    pub fn set_arguments(&self, arguments: Vec<Value>) {
        let mut data = self.data.write().unwrap();
        data.arguments = arguments;
    }

    /// Get the function arguments
    pub fn arguments(&self) -> Vec<Value> {
        self.data.read().unwrap().arguments.clone()
    }

    /// Get a specific argument by index
    pub fn argument(&self, index: usize) -> Option<Value> {
        self.data.read().unwrap().arguments.get(index).cloned()
    }

    /// Check if this function has a body
    pub fn has_body(&self) -> bool {
        !self.data.read().unwrap().basic_blocks.is_empty()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();

        write!(f, "define {} @{}(", data.ty, data.name)?;

        for (i, arg) in data.arguments.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", arg.get_type(), arg)?;
        }

        writeln!(f, ") {{")?;

        for bb in &data.basic_blocks {
            write!(f, "{}", bb)?;
        }

        writeln!(f, "}}")
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();
        write!(f, "Function(@{}, {} basic blocks)", data.name, data.basic_blocks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_function_creation() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);

        let func = Function::new("add".to_string(), fn_type);
        assert_eq!(func.name(), "add");
        assert!(!func.has_body());
    }

    #[test]
    fn test_add_basic_block() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);

        let func = Function::new("test".to_string(), fn_type);
        let bb = BasicBlock::new(Some("entry".to_string()));

        func.add_basic_block(bb);
        assert_eq!(func.basic_block_count(), 1);
        assert!(func.has_body());
    }
}
