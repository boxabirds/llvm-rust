//! LLVM Basic Blocks
//!
//! A basic block is a sequence of instructions with a single entry point
//! and a single exit point. Control flow can only enter at the beginning
//! and leave at the end.

use std::sync::{Arc, RwLock};
use std::fmt;
use crate::instruction::Instruction;

/// A basic block in LLVM IR
#[derive(Clone)]
pub struct BasicBlock {
    data: Arc<RwLock<BasicBlockData>>,
}

struct BasicBlockData {
    name: Option<String>,
    instructions: Vec<Instruction>,
    terminated: bool,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(name: Option<String>) -> Self {
        Self {
            data: Arc::new(RwLock::new(BasicBlockData {
                name,
                instructions: Vec::new(),
                terminated: false,
            })),
        }
    }

    /// Get the name of this basic block
    pub fn name(&self) -> Option<String> {
        self.data.read().unwrap().name.clone()
    }

    /// Add an instruction to this basic block
    pub fn add_instruction(&self, instruction: Instruction) {
        let mut data = self.data.write().unwrap();
        assert!(!data.terminated, "Cannot add instruction to terminated basic block");

        if instruction.is_terminator() {
            data.terminated = true;
        }

        data.instructions.push(instruction);
    }

    /// Get the instructions in this basic block
    pub fn instructions(&self) -> Vec<Instruction> {
        self.data.read().unwrap().instructions.clone()
    }

    /// Check if this basic block is terminated
    pub fn is_terminated(&self) -> bool {
        self.data.read().unwrap().terminated
    }

    /// Get the terminator instruction, if any
    pub fn terminator(&self) -> Option<Instruction> {
        let data = self.data.read().unwrap();
        data.instructions.iter()
            .filter(|inst| inst.is_terminator())
            .last()
            .cloned()
    }

    /// Get the number of instructions in this basic block
    pub fn instruction_count(&self) -> usize {
        self.data.read().unwrap().instructions.len()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();

        if let Some(ref name) = data.name {
            writeln!(f, "{}:", name)?;
        } else {
            writeln!(f, "bb:")?;
        }

        for inst in &data.instructions {
            writeln!(f, "  {}", inst)?;
        }

        Ok(())
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();
        write!(f, "BasicBlock({:?}, {} instructions)", data.name, data.instructions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Opcode;

    #[test]
    fn test_basic_block_creation() {
        let bb = BasicBlock::new(Some("entry".to_string()));
        assert_eq!(bb.name(), Some("entry".to_string()));
        assert_eq!(bb.instruction_count(), 0);
        assert!(!bb.is_terminated());
    }

    #[test]
    fn test_add_instruction() {
        let bb = BasicBlock::new(Some("entry".to_string()));
        let inst = Instruction::new(Opcode::Add, vec![], None);
        bb.add_instruction(inst);
        assert_eq!(bb.instruction_count(), 1);
    }

    #[test]
    fn test_terminator() {
        let bb = BasicBlock::new(Some("entry".to_string()));
        let inst = Instruction::new(Opcode::Ret, vec![], None);
        bb.add_instruction(inst);
        assert!(bb.is_terminated());
        assert!(bb.terminator().is_some());
    }
}
