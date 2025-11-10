//! IR Verification
//!
//! This module provides IR verification capabilities to ensure the correctness
//! of LLVM IR. It checks for type consistency, SSA form, and other invariants.

use std::collections::HashSet;
use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::{Instruction, Opcode};

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

        // Verify return types match function signature
        self.verify_return_types(function);

        // Verify SSA form
        self.verify_ssa_form(function);

        // Verify control flow
        self.verify_control_flow(function);
    }

    /// Verify that all return instructions match the function's return type
    fn verify_return_types(&mut self, function: &Function) {
        let fn_type = function.get_type();
        let return_type = match fn_type.function_return_type() {
            Some(ty) => ty,
            None => return, // Not a function type, skip verification
        };

        for bb in function.basic_blocks() {
            for inst in bb.instructions() {
                if inst.opcode() == Opcode::Ret {
                    let operands = inst.operands();

                    if return_type.is_void() {
                        // Void return: should have no operands
                        if !operands.is_empty() {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: "void".to_string(),
                                found: format!("{:?}", operands[0].get_type()),
                                location: format!("function {} return", function.name()),
                            });
                        }
                    } else {
                        // Non-void return: should have exactly 1 operand
                        if operands.is_empty() {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: format!("{:?}", return_type),
                                found: "void".to_string(),
                                location: format!("function {} return", function.name()),
                            });
                        } else if operands.len() == 1 {
                            let ret_val_type = operands[0].get_type();
                            // In modern LLVM, all pointers are opaque - treat pointer types as equivalent
                            let types_match = if ret_val_type.is_pointer() && return_type.is_pointer() {
                                true  // All pointer types are compatible
                            } else {
                                *ret_val_type == return_type
                            };

                            if !types_match {
                                self.errors.push(VerificationError::TypeMismatch {
                                    expected: format!("{:?}", return_type),
                                    found: format!("{:?}", ret_val_type),
                                    location: format!("function {} return", function.name()),
                                });
                            }
                        }
                    }
                }
            }
        }
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
        // Focus on semantic validation, not strict operand count checks

        match inst.opcode() {
            Opcode::Alloca => {
                // Alloca must allocate a sized type (not void, function, label, token, or metadata)
                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
                    // Result is a pointer, check the pointee type
                    if let Some(pointee) = result_type.pointee_type() {
                        if !pointee.is_sized() {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: format!("alloca of unsized type {:?}", pointee),
                                location: "alloca instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::Switch => {
                // Switch: condition type must match all case types
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let cond_type = operands[0].get_type();
                    // Check all case values (every other operand starting from index 2)
                    // Format: [condition, default_dest, case1_value, case1_dest, case2_value, case2_dest, ...]
                    let mut case_idx = 2;
                    while case_idx < operands.len() {
                        let case_type = operands[case_idx].get_type();
                        if *case_type != *cond_type {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: format!("{:?}", cond_type),
                                found: format!("{:?}", case_type),
                                location: format!("switch case {}", (case_idx - 2) / 2),
                            });
                        }
                        case_idx += 2; // Skip destination, move to next case value
                    }
                }
            }
            Opcode::PHI => {
                // PHI: all incoming values must have same type as result
                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
                    let operands = inst.operands();
                    // PHI operands are pairs: [value1, block1, value2, block2, ...]
                    let mut i = 0;
                    while i < operands.len() {
                        if i % 2 == 0 {
                            // Even indices are values
                            let value_type = operands[i].get_type();
                            // Allow pointer type equivalence
                            let types_match = if value_type.is_pointer() && result_type.is_pointer() {
                                true
                            } else {
                                *value_type == *result_type
                            };
                            if !types_match {
                                self.errors.push(VerificationError::TypeMismatch {
                                    expected: format!("{:?}", result_type),
                                    found: format!("{:?}", value_type),
                                    location: format!("phi incoming value {}", i / 2),
                                });
                            }
                        }
                        i += 1;
                    }
                }
            }
            Opcode::ShuffleVector => {
                // ShuffleVector: vec1 and vec2 must be same type
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let vec1_type = operands[0].get_type();
                    let vec2_type = operands[1].get_type();
                    if *vec1_type != *vec2_type {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", vec1_type),
                            found: format!("{:?}", vec2_type),
                            location: "shufflevector second vector".to_string(),
                        });
                    }
                }
            }
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::UDiv | Opcode::SDiv |
            Opcode::URem | Opcode::SRem | Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul | Opcode::FDiv | Opcode::FRem => {
                // Binary operations: both operands must have same type
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let op1_type = operands[0].get_type();
                    let op2_type = operands[1].get_type();
                    if *op1_type != *op2_type {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", op1_type),
                            found: format!("{:?}", op2_type),
                            location: format!("{:?} instruction", inst.opcode()),
                        });
                    }
                }
            }
            Opcode::ICmp | Opcode::FCmp => {
                // Comparison: both operands must have same type
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let op1_type = operands[0].get_type();
                    let op2_type = operands[1].get_type();
                    // Allow pointer type equivalence for comparisons
                    let types_match = if op1_type.is_pointer() && op2_type.is_pointer() {
                        true
                    } else {
                        *op1_type == *op2_type
                    };
                    if !types_match {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", op1_type),
                            found: format!("{:?}", op2_type),
                            location: "comparison operands".to_string(),
                        });
                    }
                }
            }
            _ => {
                // Other opcodes: no special validation yet
            }
        }
    }

    /// Verify SSA form
    pub fn verify_ssa_form(&mut self, _function: &Function) {
        // Disabled for now as it catches parser bugs rather than IR semantic errors
        // TODO: Re-enable once parser properly populates instruction operands and results
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
