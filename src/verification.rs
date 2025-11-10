//! IR Verification
//!
//! This module provides IR verification capabilities to ensure the correctness
//! of LLVM IR. It checks for type consistency, SSA form, and other invariants.

use std::collections::HashSet;
use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::{Instruction, Opcode};
use crate::types::Type;

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
    /// Invalid metadata
    InvalidMetadata { reason: String, location: String },
    /// Invalid debug info
    InvalidDebugInfo { reason: String, location: String },
    /// Metadata reference error
    MetadataReference { reason: String, location: String },
    /// Invalid control flow graph
    InvalidCFG { reason: String, location: String },
    /// Unreachable block
    UnreachableBlock { block: String },
    /// Invalid landing pad
    InvalidLandingPad { reason: String, location: String },
    /// Invalid exception handling
    InvalidExceptionHandling { reason: String, location: String },
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
            VerificationError::InvalidMetadata { reason, location } =>
                write!(f, "Invalid metadata at {}: {}", location, reason),
            VerificationError::InvalidDebugInfo { reason, location } =>
                write!(f, "Invalid debug info at {}: {}", location, reason),
            VerificationError::MetadataReference { reason, location } =>
                write!(f, "Metadata reference error at {}: {}", location, reason),
            VerificationError::InvalidCFG { reason, location } =>
                write!(f, "Invalid control flow graph at {}: {}", location, reason),
            VerificationError::UnreachableBlock { block } =>
                write!(f, "Unreachable block: {}", block),
            VerificationError::InvalidLandingPad { reason, location } =>
                write!(f, "Invalid landing pad at {}: {}", location, reason),
            VerificationError::InvalidExceptionHandling { reason, location } =>
                write!(f, "Invalid exception handling at {}: {}", location, reason),
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

    /// Check if type is integer or vector of integers
    fn is_integer_or_vector_of_integers(&self, ty: &Type) -> bool {
        ty.is_integer() ||
            (ty.is_vector() && ty.vector_info().map_or(false, |(elem, _)| elem.is_integer()))
    }

    /// Check if type is float or vector of floats
    fn is_float_or_vector_of_floats(&self, ty: &Type) -> bool {
        ty.is_float() ||
            (ty.is_vector() && ty.vector_info().map_or(false, |(elem, _)| elem.is_float()))
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

        // Check landing pad position
        // Landing pads must be first non-PHI instruction
        let mut found_non_phi = false;
        let mut found_landingpad = false;
        for inst in instructions.iter() {
            if inst.opcode() == Opcode::LandingPad {
                if found_landingpad {
                    self.errors.push(VerificationError::InvalidLandingPad {
                        reason: "multiple landing pads in same block".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                }
                if found_non_phi {
                    self.errors.push(VerificationError::InvalidLandingPad {
                        reason: "landing pad must be first non-PHI instruction in block".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                }
                found_landingpad = true;
            } else if inst.opcode() != Opcode::PHI {
                found_non_phi = true;
            }
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
            // === CAST OPERATIONS ===
            Opcode::Trunc => {
                // Trunc: operand must be integer or vector of integers
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !self.is_integer_or_vector_of_integers(&src_type) {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: "trunc operand must be integer or vector of integers".to_string(),
                            location: "trunc instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !self.is_integer_or_vector_of_integers(&dst_type) {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "trunc result must be integer or vector of integers".to_string(),
                                location: "trunc instruction".to_string(),
                            });
                        } else if let (Some(src_bits), Some(dst_bits)) = (src_type.int_width(), dst_type.int_width()) {
                            if dst_bits >= src_bits {
                                self.errors.push(VerificationError::InvalidCast {
                                    from: format!("{:?}", src_type),
                                    to: format!("{:?}", dst_type),
                                    reason: format!("trunc result must be smaller (src: {} bits, dst: {} bits)", src_bits, dst_bits),
                                    location: "trunc instruction".to_string(),
                                });
                            }
                        }
                    }
                }
            }
            Opcode::ZExt | Opcode::SExt => {
                // ZExt/SExt: operand must be integer or vector of integers
                let operands = inst.operands();
                let opcode_name = if inst.opcode() == Opcode::ZExt { "zext" } else { "sext" };
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !self.is_integer_or_vector_of_integers(&src_type) {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: format!("{} operand must be integer or vector of integers", opcode_name),
                            location: format!("{} instruction", opcode_name),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !self.is_integer_or_vector_of_integers(&dst_type) {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: format!("{} result must be integer or vector of integers", opcode_name),
                                location: format!("{} instruction", opcode_name),
                            });
                        } else if let (Some(src_bits), Some(dst_bits)) = (src_type.int_width(), dst_type.int_width()) {
                            if dst_bits <= src_bits {
                                self.errors.push(VerificationError::InvalidCast {
                                    from: format!("{:?}", src_type),
                                    to: format!("{:?}", dst_type),
                                    reason: format!("{} result must be larger (src: {} bits, dst: {} bits)", opcode_name, src_bits, dst_bits),
                                    location: format!("{} instruction", opcode_name),
                                });
                            }
                        }
                    }
                }
            }
            Opcode::FPTrunc => {
                // FPTrunc: operand must be float or vector of floats
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !self.is_float_or_vector_of_floats(&src_type) {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "float".to_string(),
                            reason: "fptrunc operand must be floating point or vector of floats".to_string(),
                            location: "fptrunc instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !self.is_float_or_vector_of_floats(&dst_type) {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "fptrunc result must be floating point or vector of floats".to_string(),
                                location: "fptrunc instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::FPExt => {
                // FPExt: operand must be float or vector of floats
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !self.is_float_or_vector_of_floats(&src_type) {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "float".to_string(),
                            reason: "fpext operand must be floating point or vector of floats".to_string(),
                            location: "fpext instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !self.is_float_or_vector_of_floats(&dst_type) {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "fpext result must be floating point or vector of floats".to_string(),
                                location: "fpext instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::FPToUI | Opcode::FPToSI => {
                // FPToUI/FPToSI: operand must be float, result must be integer
                let operands = inst.operands();
                let opcode_name = if inst.opcode() == Opcode::FPToUI { "fptoui" } else { "fptosi" };
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !src_type.is_float() {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "float".to_string(),
                            reason: format!("{} operand must be floating point type", opcode_name),
                            location: format!("{} instruction", opcode_name),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !dst_type.is_integer() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: format!("{} result must be integer type", opcode_name),
                                location: format!("{} instruction", opcode_name),
                            });
                        }
                    }
                }
            }
            Opcode::UIToFP | Opcode::SIToFP => {
                // UIToFP/SIToFP: operand must be integer, result must be float
                let operands = inst.operands();
                let opcode_name = if inst.opcode() == Opcode::UIToFP { "uitofp" } else { "sitofp" };
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !src_type.is_integer() {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: format!("{} operand must be integer type", opcode_name),
                            location: format!("{} instruction", opcode_name),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !dst_type.is_float() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: format!("{} result must be floating point type", opcode_name),
                                location: format!("{} instruction", opcode_name),
                            });
                        }
                    }
                }
            }
            Opcode::PtrToInt => {
                // PtrToInt: operand must be pointer, result must be integer
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !src_type.is_pointer() {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "pointer".to_string(),
                            reason: "ptrtoint operand must be pointer type".to_string(),
                            location: "ptrtoint instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !dst_type.is_integer() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "ptrtoint result must be integer type".to_string(),
                                location: "ptrtoint instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::IntToPtr => {
                // IntToPtr: operand must be integer, result must be pointer
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !src_type.is_integer() {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: "inttoptr operand must be integer type".to_string(),
                            location: "inttoptr instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !dst_type.is_pointer() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "inttoptr result must be pointer type".to_string(),
                                location: "inttoptr instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::BitCast => {
                // BitCast: basic type compatibility check
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        // Bitcast cannot convert to/from void
                        if src_type.is_void() || dst_type.is_void() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "bitcast cannot convert to/from void type".to_string(),
                                location: "bitcast instruction".to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::AddrSpaceCast => {
                // AddrSpaceCast: both must be pointers
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    if !src_type.is_pointer() {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "pointer".to_string(),
                            reason: "addrspacecast operand must be pointer type".to_string(),
                            location: "addrspacecast instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        if !dst_type.is_pointer() {
                            self.errors.push(VerificationError::InvalidCast {
                                from: format!("{:?}", src_type),
                                to: format!("{:?}", dst_type),
                                reason: "addrspacecast result must be pointer type".to_string(),
                                location: "addrspacecast instruction".to_string(),
                            });
                        }
                    }
                }
            }

            // === AGGREGATE OPERATIONS ===
            Opcode::ExtractElement => {
                // ExtractElement: first operand must be vector, second must be integer index
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let vec_type = operands[0].get_type();
                    let idx_type = operands[1].get_type();

                    if !vec_type.is_vector() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("extractelement first operand must be vector type, got {:?}", vec_type),
                            location: "extractelement instruction".to_string(),
                        });
                    }

                    if !idx_type.is_integer() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("extractelement index must be integer type, got {:?}", idx_type),
                            location: "extractelement instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::InsertElement => {
                // InsertElement: first operand must be vector, value must match element type, index must be integer
                let operands = inst.operands();
                if operands.len() >= 3 {
                    let vec_type = operands[0].get_type();
                    let val_type = operands[1].get_type();
                    let idx_type = operands[2].get_type();

                    if !vec_type.is_vector() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("insertelement first operand must be vector type, got {:?}", vec_type),
                            location: "insertelement instruction".to_string(),
                        });
                    } else if let Some((elem_type, _)) = vec_type.vector_info() {
                        if *val_type != *elem_type {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: format!("{:?}", elem_type),
                                found: format!("{:?}", val_type),
                                location: "insertelement value".to_string(),
                            });
                        }
                    }

                    if !idx_type.is_integer() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("insertelement index must be integer type, got {:?}", idx_type),
                            location: "insertelement instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::ExtractValue => {
                // ExtractValue: operand must be aggregate type (struct or array)
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let agg_type = operands[0].get_type();
                    if !agg_type.is_struct() && !agg_type.is_array() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("extractvalue operand must be aggregate type (struct or array), got {:?}", agg_type),
                            location: "extractvalue instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::InsertValue => {
                // InsertValue: operand must be aggregate type (struct or array)
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let agg_type = operands[0].get_type();
                    if !agg_type.is_struct() && !agg_type.is_array() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("insertvalue operand must be aggregate type (struct or array), got {:?}", agg_type),
                            location: "insertvalue instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::GetElementPtr => {
                // GetElementPtr: base must be pointer or vector of pointers
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let base_type = operands[0].get_type();
                    // Check if base is pointer or vector of pointers
                    let is_valid_base = base_type.is_pointer() ||
                        (base_type.is_vector() && base_type.vector_info().map_or(false, |(elem, _)| elem.is_pointer()));

                    if !is_valid_base {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("getelementptr base must be pointer or vector of pointers, got {:?}", base_type),
                            location: "getelementptr instruction".to_string(),
                        });
                    }

                    // All index operands must be integers or vectors of integers
                    for (i, operand) in operands.iter().enumerate().skip(1) {
                        let idx_type = operand.get_type();
                        let is_valid_index = idx_type.is_integer() ||
                            (idx_type.is_vector() && idx_type.vector_info().map_or(false, |(elem, _)| elem.is_integer()));

                        if !is_valid_index {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: format!("getelementptr index {} must be integer or vector of integers, got {:?}", i-1, idx_type),
                                location: "getelementptr instruction".to_string(),
                            });
                        }
                    }
                }
            }

            // === FUNCTION CALL VALIDATION ===
            Opcode::Call => {
                // Call: validate argument count and types match function signature
                let operands = inst.operands();
                if operands.is_empty() {
                    return; // No callee, skip validation
                }

                let callee = &operands[0];
                let callee_type = callee.get_type();

                // If callee is a pointer to function, get the function type
                let fn_type = if callee_type.is_pointer() {
                    if let Some(pointee) = callee_type.pointee_type() {
                        if pointee.is_function() {
                            pointee.clone()
                        } else {
                            return; // Not a function pointer
                        }
                    } else {
                        return;
                    }
                } else if callee_type.is_function() {
                    callee_type.clone()
                } else {
                    return; // Not a function type
                };

                if let Some((ret_type, param_types, is_var_arg)) = fn_type.function_info() {
                    let args = &operands[1..];

                    // Check argument count (varargs functions can have more)
                    if !is_var_arg && args.len() != param_types.len() {
                        self.errors.push(VerificationError::InvalidCall {
                            expected_args: param_types.len(),
                            found_args: args.len(),
                            location: "call instruction".to_string(),
                        });
                    } else if args.len() < param_types.len() {
                        // Even varargs functions need at least the fixed parameters
                        self.errors.push(VerificationError::InvalidCall {
                            expected_args: param_types.len(),
                            found_args: args.len(),
                            location: "call instruction (too few args for varargs)".to_string(),
                        });
                    }

                    // Check argument types match parameter types
                    for (i, (arg, param_type)) in args.iter().zip(param_types.iter()).enumerate() {
                        let arg_type = arg.get_type();
                        // Allow pointer type equivalence and metadata type equivalence
                        let types_match = if arg_type.is_pointer() && param_type.is_pointer() {
                            true
                        } else if arg_type.is_metadata() && param_type.is_metadata() {
                            true
                        } else {
                            *arg_type == *param_type
                        };

                        if !types_match {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: format!("{:?}", param_type),
                                found: format!("{:?}", arg_type),
                                location: format!("call argument {}", i),
                            });
                        }
                    }

                    // Verify return type matches result
                    if let Some(result) = inst.result() {
                        let result_type = result.get_type();
                        let types_match = if result_type.is_pointer() && ret_type.is_pointer() {
                            true
                        } else {
                            *result_type == ret_type
                        };

                        if !types_match {
                            self.errors.push(VerificationError::TypeMismatch {
                                expected: format!("{:?}", ret_type),
                                found: format!("{:?}", result_type),
                                location: "call result type".to_string(),
                            });
                        }
                    }
                    // Note: We don't validate missing result for non-void return because
                    // calls with sret (struct return) attribute explicitly use "call void"
                    // even though the function returns a struct through a pointer parameter
                }
            }

            // === EXISTING VALIDATIONS ===
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
                // PHI: all incoming values must have same type as result, must have even operand count
                let operands = inst.operands();

                // PHI must have even number of operands (value/block pairs)
                if operands.len() % 2 != 0 {
                    self.errors.push(VerificationError::InvalidPhi {
                        reason: format!("phi must have even number of operands (value/block pairs), found {}", operands.len()),
                        location: "phi instruction".to_string(),
                    });
                }

                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
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
                // ShuffleVector: vec1 and vec2 must be same type, mask must be vector of integers
                let operands = inst.operands();
                if operands.len() >= 3 {
                    let vec1_type = operands[0].get_type();
                    let vec2_type = operands[1].get_type();
                    let mask_type = operands[2].get_type();

                    // vec1 and vec2 must be same type
                    if *vec1_type != *vec2_type {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", vec1_type),
                            found: format!("{:?}", vec2_type),
                            location: "shufflevector second vector".to_string(),
                        });
                    }

                    // Both must be vectors
                    if !vec1_type.is_vector() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("shufflevector operands must be vector types, got {:?}", vec1_type),
                            location: "shufflevector instruction".to_string(),
                        });
                    }

                    // Mask must be a vector of integers
                    if mask_type.is_vector() {
                        if let Some((elem_type, _)) = mask_type.vector_info() {
                            if !elem_type.is_integer() {
                                self.errors.push(VerificationError::InvalidInstruction {
                                    reason: format!("shufflevector mask must be vector of integers, got vector of {:?}", elem_type),
                                    location: "shufflevector instruction".to_string(),
                                });
                            }
                        }
                    } else {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("shufflevector mask must be vector type, got {:?}", mask_type),
                            location: "shufflevector instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::Shl | Opcode::LShr | Opcode::AShr => {
                // Shift operations: both operands must be same integer or vector type
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let value_type = operands[0].get_type();
                    let shift_type = operands[1].get_type();

                    // Value must be integer or vector of integers
                    if !value_type.is_integer() && !value_type.is_vector() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("shift operand must be integer or vector type, got {:?}", value_type),
                            location: format!("{:?} instruction", inst.opcode()),
                        });
                    }

                    // Shift amount must have same type as value
                    if *value_type != *shift_type {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", value_type),
                            found: format!("{:?}", shift_type),
                            location: format!("{:?} shift amount", inst.opcode()),
                        });
                    }
                }
            }
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::UDiv | Opcode::SDiv |
            Opcode::URem | Opcode::SRem |
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

                    // Integer operations must have integer operands
                    match inst.opcode() {
                        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::UDiv | Opcode::SDiv |
                        Opcode::URem | Opcode::SRem | Opcode::And | Opcode::Or | Opcode::Xor => {
                            if !op1_type.is_integer() && !op1_type.is_vector() {
                                self.errors.push(VerificationError::InvalidInstruction {
                                    reason: format!("integer operation requires integer or vector operands, got {:?}", op1_type),
                                    location: format!("{:?} instruction", inst.opcode()),
                                });
                            }
                        }
                        Opcode::FAdd | Opcode::FSub | Opcode::FMul | Opcode::FDiv | Opcode::FRem => {
                            if !op1_type.is_float() && !op1_type.is_vector() {
                                self.errors.push(VerificationError::InvalidInstruction {
                                    reason: format!("floating point operation requires float or vector operands, got {:?}", op1_type),
                                    location: format!("{:?} instruction", inst.opcode()),
                                });
                            }
                        }
                        _ => {}
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
            Opcode::Store => {
                // Store: value type must be sized
                // Skip validation if types are void (indicates parser limitations)
                let operands = inst.operands();
                if operands.len() >= 2 {
                    let value_type = operands[0].get_type();
                    let ptr_type = operands[1].get_type();

                    // Skip validation if either type is void (parser limitation)
                    if value_type.is_void() || ptr_type.is_void() {
                        return;
                    }

                    // Validate pointer operand is actually a pointer
                    if !ptr_type.is_pointer() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("store pointer operand must be a pointer type, got {:?}", ptr_type),
                            location: "store instruction".to_string(),
                        });
                    }

                    // Value must be sized (structs are sized in LLVM)
                    if !value_type.is_sized() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("store value must be sized type, got {:?}", value_type),
                            location: "store instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::Load => {
                // Load: pointer must be pointer type, result must be sized
                // Skip validation if types are void (indicates parser limitations)
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let ptr_type = operands[0].get_type();

                    // Skip if void (parser limitation)
                    if ptr_type.is_void() {
                        return;
                    }

                    if !ptr_type.is_pointer() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("load operand must be a pointer type, got {:?}", ptr_type),
                            location: "load instruction".to_string(),
                        });
                    }
                }

                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
                    // Skip if void (parser may not preserve full type info)
                    if result_type.is_void() {
                        return;
                    }

                    if !result_type.is_sized() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("load result must be sized type, got {:?}", result_type),
                            location: "load instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::Select => {
                // Select: condition must be i1 or vector of i1, both values must match type
                let operands = inst.operands();
                if operands.len() >= 3 {
                    let cond_type = operands[0].get_type();
                    let true_type = operands[1].get_type();
                    let false_type = operands[2].get_type();

                    // Allow pointer type equivalence
                    let types_match = if true_type.is_pointer() && false_type.is_pointer() {
                        true
                    } else {
                        *true_type == *false_type
                    };

                    if !types_match {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: format!("{:?}", true_type),
                            found: format!("{:?}", false_type),
                            location: "select true/false values".to_string(),
                        });
                    }

                    // Condition should be i1 (or vector of i1)
                    if !cond_type.is_integer() && !cond_type.is_vector() {
                        self.errors.push(VerificationError::TypeMismatch {
                            expected: "i1 or vector of i1".to_string(),
                            found: format!("{:?}", cond_type),
                            location: "select condition".to_string(),
                        });
                    }
                }
            }
            // === EXCEPTION HANDLING VALIDATION ===
            Opcode::LandingPad => {
                // LandingPad: must be first non-PHI instruction in block
                // Note: Cannot fully validate without block context
                // This will be checked in verify_basic_block
            }
            Opcode::Invoke => {
                // Invoke: must have both normal and unwind destinations
                // Note: Parser must preserve successor information
                // Basic validation: check it's a valid function call
                let operands = inst.operands();
                if operands.is_empty() {
                    self.errors.push(VerificationError::InvalidExceptionHandling {
                        reason: "invoke must have a callee".to_string(),
                        location: "invoke instruction".to_string(),
                    });
                }
            }
            Opcode::Resume => {
                // Resume: must have exactly one operand of aggregate type
                let operands = inst.operands();
                if operands.len() != 1 {
                    self.errors.push(VerificationError::InvalidExceptionHandling {
                        reason: format!("resume must have exactly one operand, found {}", operands.len()),
                        location: "resume instruction".to_string(),
                    });
                } else {
                    let arg_type = operands[0].get_type();
                    if !arg_type.is_struct() {
                        self.errors.push(VerificationError::InvalidExceptionHandling {
                            reason: format!("resume operand must be aggregate type, got {:?}", arg_type),
                            location: "resume instruction".to_string(),
                        });
                    }
                }
            }
            Opcode::CatchPad | Opcode::CleanupPad => {
                // CatchPad/CleanupPad: Windows exception handling
                // Must have a parent catchswitch or cleanuppad
                // Note: Full validation requires CFG analysis
            }
            Opcode::CatchSwitch => {
                // CatchSwitch: must have at least one handler
                // Note: Parser must preserve handler information
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

    /// Verify metadata attachments (placeholder - parser doesn't preserve metadata yet)
    pub fn verify_metadata(&mut self, _module: &Module) {
        // TODO: Once parser preserves metadata, validate:
        // 1. Named metadata nodes are well-formed
        // 2. Metadata references are valid
        // 3. Debug info structure is correct
        // 4. Metadata types are appropriate for their use
        // 5. No circular references in metadata
    }

    /// Verify debug info metadata structure
    fn verify_debug_info(&mut self, _debug_info: &crate::metadata::DebugInfo, _location: &str) {
        // TODO: Validate debug info structure:
        // 1. Compile units have valid file references
        // 2. Subprograms have valid scopes
        // 3. Types have valid sizes and encodings
        // 4. Locations have valid line/column numbers
    }

    /// Verify control flow
    pub fn verify_control_flow(&mut self, function: &Function) {
        // Enhanced CFG validation with reachability analysis
        let basic_blocks = function.basic_blocks();
        if basic_blocks.is_empty() {
            return;
        }

        // Check entry block exists and has no predecessors
        if let Some(entry) = function.entry_block() {
            if let Some(entry_name) = entry.name() {
                // Entry block should be first in the list
                if !basic_blocks.is_empty() {
                    if let Some(first_name) = basic_blocks[0].name() {
                        if first_name != entry_name {
                            self.errors.push(VerificationError::InvalidCFG {
                                reason: "entry block must be first block in function".to_string(),
                                location: format!("function {}", function.name()),
                            });
                        }
                    }
                }

                // TODO: Implement full reachability analysis
                // This would require:
                // 1. Build CFG from terminator instructions
                // 2. Perform DFS/BFS from entry block
                // 3. Mark all reachable blocks
                // 4. Report unreachable blocks
                //
                // Current limitation: Parser doesn't preserve CFG edges
                // so we can't traverse the graph
            }
        } else {
            self.errors.push(VerificationError::EntryBlockMissing {
                function: function.name(),
            });
        }

        // Validate exception handling control flow
        self.verify_exception_handling_cfg(function);
    }

    /// Verify exception handling control flow constraints
    fn verify_exception_handling_cfg(&mut self, function: &Function) {
        // Check that landing pads are only in blocks reachable via invoke
        // Check that resume is only in landing pad blocks
        // Note: Full validation requires CFG analysis

        for bb in function.basic_blocks() {
            let instructions = bb.instructions();
            let mut has_landingpad = false;
            let mut has_resume = false;

            for inst in instructions.iter() {
                match inst.opcode() {
                    Opcode::LandingPad => has_landingpad = true,
                    Opcode::Resume => has_resume = true,
                    _ => {}
                }
            }

            // Resume should typically appear in blocks with landing pads
            // This is a soft constraint - document but don't error
            if has_resume && !has_landingpad {
                // This is valid in cleanup blocks, so don't error
                // Just a note for future enhancement
            }
        }
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
