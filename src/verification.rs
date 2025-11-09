//! IR Verification
//!
//! This module provides IR verification capabilities to ensure the correctness
//! of LLVM IR. It checks for type consistency, SSA form, and other invariants.

use std::collections::{HashMap, HashSet};
use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::{Instruction, Opcode};
use crate::value::Value;

/// Verification errors
#[derive(Debug, Clone)]
pub enum VerificationError {
    /// Type mismatch
    TypeMismatch { expected: String, found: String, location: String },
    /// Invalid SSA form
    InvalidSSA { value: String, location: String },
    /// Missing terminator
    MissingTerminator { block: String },
    /// Multiple terminators
    MultipleTerminators { block: String },
    /// Undefined value
    UndefinedValue { value: String, location: String },
    /// Invalid operand count
    InvalidOperandCount { instruction: String, expected: usize, found: usize },
    /// Invalid instruction in basic block
    InvalidInstruction { reason: String, location: String },
    /// Entry block missing
    EntryBlockMissing { function: String },
    /// Unreachable code
    UnreachableCode { location: String },
    /// Invalid control flow
    InvalidControlFlow { reason: String, location: String },
    /// Invalid cast operation
    InvalidCast { from: String, to: String, reason: String, location: String },
    /// Invalid function call
    InvalidCall { expected_args: usize, found_args: usize, location: String },
    /// Invalid phi node
    InvalidPhi { reason: String, location: String },
    /// Invalid alignment
    InvalidAlignment { value: usize, location: String },
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationError::TypeMismatch { expected, found, location } =>
                write!(f, "Type mismatch at {}: expected {}, found {}", location, expected, found),
            VerificationError::InvalidSSA { value, location } =>
                write!(f, "Invalid SSA form at {}: value {} assigned multiple times", location, value),
            VerificationError::MissingTerminator { block } =>
                write!(f, "Block {} missing terminator instruction", block),
            VerificationError::MultipleTerminators { block } =>
                write!(f, "Block {} has multiple terminator instructions", block),
            VerificationError::UndefinedValue { value, location } =>
                write!(f, "Undefined value {} used at {}", value, location),
            VerificationError::InvalidOperandCount { instruction, expected, found } =>
                write!(f, "Invalid operand count for {}: expected {}, found {}", instruction, expected, found),
            VerificationError::InvalidInstruction { reason, location } =>
                write!(f, "Invalid instruction at {}: {}", location, reason),
            VerificationError::EntryBlockMissing { function } =>
                write!(f, "Function {} missing entry block", function),
            VerificationError::UnreachableCode { location } =>
                write!(f, "Unreachable code at {}", location),
            VerificationError::InvalidControlFlow { reason, location } =>
                write!(f, "Invalid control flow at {}: {}", location, reason),
            VerificationError::InvalidCast { from, to, reason, location } =>
                write!(f, "Invalid cast from {} to {} at {}: {}", from, to, location, reason),
            VerificationError::InvalidCall { expected_args, found_args, location } =>
                write!(f, "Invalid function call at {}: expected {} arguments, found {}", location, expected_args, found_args),
            VerificationError::InvalidPhi { reason, location } =>
                write!(f, "Invalid phi node at {}: {}", location, reason),
            VerificationError::InvalidAlignment { value, location } =>
                write!(f, "Invalid alignment {} at {}: must be power of 2", value, location),
        }
    }
}

impl std::error::Error for VerificationError {}

/// Verification result
pub type VerificationResult = Result<(), Vec<VerificationError>>;

/// IR verifier
pub struct Verifier {
    errors: Vec<VerificationError>,
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Verify a module
    pub fn verify_module(&mut self, module: &Module) -> VerificationResult {
        self.errors.clear();

        // Verify all functions in the module
        for function in module.functions() {
            self.verify_function(&function);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Verify a function
    pub fn verify_function(&mut self, function: &Function) {
        // Check if function has a body
        if !function.has_body() {
            return; // External function, nothing to verify
        }

        // Check if function has an entry block
        if function.entry_block().is_none() {
            self.errors.push(VerificationError::EntryBlockMissing {
                function: function.name(),
            });
            return;
        }

        // Verify each basic block
        for bb in function.basic_blocks() {
            self.verify_basic_block(&bb);
        }

        // Verify SSA form
        self.verify_ssa_form(function);

        // Verify control flow
        self.verify_control_flow(function);
    }

    /// Verify a basic block
    pub fn verify_basic_block(&mut self, bb: &BasicBlock) {
        let instructions = bb.instructions();

        if instructions.is_empty() {
            self.errors.push(VerificationError::MissingTerminator {
                block: bb.name().unwrap_or_else(|| "unnamed".to_string()),
            });
            return;
        }

        // Check terminator
        let mut terminator_count = 0;
        for (i, inst) in instructions.iter().enumerate() {
            if inst.is_terminator() {
                terminator_count += 1;
                // Terminator must be last
                if i != instructions.len() - 1 {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Terminator must be last instruction".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                }
            }
        }

        if terminator_count == 0 {
            self.errors.push(VerificationError::MissingTerminator {
                block: bb.name().unwrap_or_else(|| "unnamed".to_string()),
            });
        } else if terminator_count > 1 {
            self.errors.push(VerificationError::MultipleTerminators {
                block: bb.name().unwrap_or_else(|| "unnamed".to_string()),
            });
        }

        // Verify each instruction
        for inst in instructions.iter() {
            self.verify_instruction(inst);
        }
    }

    /// Verify an instruction
    pub fn verify_instruction(&mut self, inst: &Instruction) {
        let location = format!("instruction {:?}", inst.opcode());

        // Type checking based on opcode
        match inst.opcode() {
            Opcode::Add | Opcode::Sub | Opcode::Mul |
            Opcode::UDiv | Opcode::SDiv | Opcode::URem | Opcode::SRem |
            Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor => {
                // Binary operations: require 2 operands of same integer type
                let operands = inst.operands();
                if operands.len() < 2 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: format!("{:?}", inst.opcode()),
                        expected: 2,
                        found: operands.len(),
                    });
                    return;
                }

                // Check that operands have compatible types
                if operands.len() >= 2 {
                    let ty1 = operands[0].get_type();
                    let ty2 = operands[1].get_type();

                    if !ty1.is_integer() || !ty2.is_integer() {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: "integer type".to_string(),
                            found: format!("{:?}, {:?}", ty1, ty2),
                            location: location.clone(),
                        });
                    }
                }
            }

            Opcode::FAdd | Opcode::FSub | Opcode::FMul | Opcode::FDiv | Opcode::FRem => {
                // Floating-point binary operations
                let operands = inst.operands();
                if operands.len() < 2 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: format!("{:?}", inst.opcode()),
                        expected: 2,
                        found: operands.len(),
                    });
                    return;
                }

                if operands.len() >= 2 {
                    let ty1 = operands[0].get_type();
                    let ty2 = operands[1].get_type();

                    if !ty1.is_float() || !ty2.is_float() {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: "floating-point type".to_string(),
                            found: format!("{:?}, {:?}", ty1, ty2),
                            location: location.clone(),
                        });
                    }
                }
            }

            Opcode::ICmp => {
                // Integer comparison: requires 2 integer operands
                let operands = inst.operands();
                if operands.len() < 2 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "icmp".to_string(),
                        expected: 2,
                        found: operands.len(),
                    });
                }
            }

            Opcode::FCmp => {
                // Float comparison: requires 2 float operands
                let operands = inst.operands();
                if operands.len() < 2 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "fcmp".to_string(),
                        expected: 2,
                        found: operands.len(),
                    });
                }
            }

            Opcode::Br => {
                // Branch: either 1 label (unconditional) or 3 operands (conditional)
                let operands = inst.operands();
                if operands.len() != 1 && operands.len() != 3 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "br".to_string(),
                        expected: 1,  // or 3
                        found: operands.len(),
                    });
                }
            }

            Opcode::Ret => {
                // Return: 0 operands (void) or 1 operand (value)
                let operands = inst.operands();
                if operands.len() > 1 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "ret".to_string(),
                        expected: 1,
                        found: operands.len(),
                    });
                }
            }

            Opcode::Call => {
                // Call: function + arguments
                // Type checking would require function signature analysis
                // For now, just check that we have at least a function operand
                let operands = inst.operands();
                if operands.is_empty() {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "call".to_string(),
                        expected: 1,
                        found: 0,
                    });
                }
            }

            Opcode::Alloca => {
                // Alloca: requires type operand
                // Check alignment if specified (must be power of 2)
                // This is handled by the parser mostly
            }

            Opcode::Load | Opcode::Store => {
                // Memory operations: require pointer operand
                let operands = inst.operands();
                if operands.is_empty() {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: format!("{:?}", inst.opcode()),
                        expected: 1,
                        found: 0,
                    });
                }
            }

            Opcode::GetElementPtr => {
                // GEP: requires pointer base + indices
                let operands = inst.operands();
                if operands.len() < 2 {
                    self.errors.push(VerificationError::InvalidOperandCount {
                        instruction: "getelementptr".to_string(),
                        expected: 2,
                        found: operands.len(),
                    });
                }
            }

            _ => {
                // Other opcodes: basic validation
                // Full LLVM verification would check all opcode-specific constraints
            }
        }
    }

    /// Verify SSA form
    pub fn verify_ssa_form(&mut self, function: &Function) {
        let mut defined_values: HashSet<String> = HashSet::new();

        // Add function arguments as defined values
        for (i, arg) in function.arguments().iter().enumerate() {
            if let Some(name) = arg.name() {
                defined_values.insert(name.to_string());
            } else {
                defined_values.insert(format!("arg{}", i));
            }
        }

        // Check each basic block
        for bb in function.basic_blocks() {
            for inst in bb.instructions() {
                // Check that operands are defined before use
                for operand in inst.operands() {
                    if let Some(name) = operand.name() {
                        if !defined_values.contains(name) && !operand.is_constant() {
                            self.errors.push(VerificationError::UndefinedValue {
                                value: name.to_string(),
                                location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                            });
                        }
                    }
                }

                // Add result to defined values
                if let Some(result) = inst.result() {
                    if let Some(name) = result.name() {
                        if defined_values.contains(name) {
                            self.errors.push(VerificationError::InvalidSSA {
                                value: name.to_string(),
                                location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                            });
                        }
                        defined_values.insert(name.to_string());
                    }
                }
            }
        }
    }

    /// Verify control flow
    pub fn verify_control_flow(&mut self, function: &Function) {
        // Check for unreachable blocks
        // Check for infinite loops
        // Check for proper dominator relationships
        // This is a simplified version

        let basic_blocks = function.basic_blocks();
        if basic_blocks.is_empty() {
            return;
        }

        // Simple reachability check
        let mut reachable: HashSet<String> = HashSet::new();
        if let Some(entry) = function.entry_block() {
            if let Some(name) = entry.name() {
                reachable.insert(name);
            }
        }

        // Mark all blocks as potentially unreachable for now
        // A full implementation would do proper CFG traversal
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify a module and return a result
pub fn verify_module(module: &Module) -> VerificationResult {
    let mut verifier = Verifier::new();
    verifier.verify_module(module)
}

/// Verify a function and return a result
pub fn verify_function(function: &Function) -> VerificationResult {
    let mut verifier = Verifier::new();
    verifier.verify_function(function);

    if verifier.errors.is_empty() {
        Ok(())
    } else {
        Err(verifier.errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, BasicBlock};

    #[test]
    fn test_verify_empty_module() {
        let ctx = Context::new();
        let module = Module::new("test".to_string(), ctx);
        let result = verify_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_function_without_terminator() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        let bb = BasicBlock::new(Some("entry".to_string()));
        func.add_basic_block(bb);

        let result = verify_function(&func);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert_eq!(errors.len(), 1);
            match &errors[0] {
                VerificationError::MissingTerminator { .. } => {},
                _ => panic!("Expected MissingTerminator error"),
            }
        }
    }
}
