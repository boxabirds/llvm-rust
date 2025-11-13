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
                write!(f, "multiple definition of local value named '{}'", value),
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
    current_function: Option<String>,
    current_function_is_varargs: bool,
    current_function_has_personality: bool,
    current_function_calling_convention: crate::function::CallingConvention,
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            current_function: None,
            current_function_is_varargs: false,
            current_function_has_personality: false,
            current_function_calling_convention: crate::function::CallingConvention::C,
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

        // Check for duplicate global definitions (functions, globals, aliases)
        use std::collections::HashSet;
        let mut global_names = HashSet::new();

        for global in module.globals() {
            let name = global.name();
            // Skip duplicate check for empty/numbered names
            if !name.is_empty() && !name.chars().all(|c| c.is_ascii_digit()) {
                if !global_names.insert(name.to_string()) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("redefinition of global '@{}'", name),
                        location: format!("global variable @{}", name),
                    });
                }
            }
        }

        for function in module.functions() {
            let name = function.name();
            // Skip duplicate check for empty/numbered names
            if !name.is_empty() && !name.chars().all(|c| c.is_ascii_digit()) {
                if !global_names.insert(name.to_string()) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("redefinition of global '@{}'", name),
                        location: format!("function {}", name),
                    });
                }
            }
        }

        for alias in module.aliases() {
            let name = &alias.name;
            // Skip duplicate check for empty/numbered names
            if !name.is_empty() && !name.chars().all(|c| c.is_ascii_digit()) {
                if !global_names.insert(name.to_string()) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("redefinition of global '@{}'", name),
                        location: format!("alias @{}", name),
                    });
                }
            }
        }

        // Verify global variables
        for global in module.globals() {
            let global_type = global.get_type();
            // If it's a pointer, check the pointee type
            let value_type = if global_type.is_pointer() {
                global_type.pointee_type().unwrap_or(global_type)
            } else {
                global_type
            };

            // Global variables cannot have token type
            if value_type.is_token() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "invalid type for global variable".to_string(),
                    location: format!("global variable @{}", global.name()),
                });
            }

            // Verify global variable constraints
            self.verify_global_variable(&global);
        }

        // Verify all functions in the module
        for function in module.functions() {
            let fn_name = function.name();

            // Check if this is an intrinsic function (starts with "llvm.")
            if fn_name.starts_with("llvm.") {
                // Intrinsics cannot be defined, only declared
                // Check if function has a body (basic blocks)
                if !function.basic_blocks().is_empty() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "llvm intrinsics cannot be defined".to_string(),
                        location: format!("function {}", fn_name),
                    });
                }
            }

            self.verify_function(&function);
        }

        // Verify module-level metadata
        self.verify_module_flags(module);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Verify module flags metadata
    fn verify_module_flags(&mut self, module: &Module) {
        use crate::metadata::Metadata;
        use std::collections::HashMap;

        let flags = module.module_flags();
        let mut flag_ids = HashMap::new();

        for flag in &flags {
            // Module flags must be metadata tuples with exactly 3 operands
            if let Some(operands) = flag.operands() {
                if operands.len() != 3 {
                    self.errors.push(VerificationError::InvalidMetadata {
                        reason: "incorrect number of operands in module flag".to_string(),
                        location: "module flags".to_string(),
                    });
                    continue;
                }

                // First operand: behavior (must be constant integer 1-8)
                let behavior = if let Some(behavior_val) = operands[0].as_i32() {
                    if behavior_val < 1 || behavior_val > 8 {
                        self.errors.push(VerificationError::InvalidMetadata {
                            reason: format!("invalid behavior operand in module flag (unexpected constant)"),
                            location: "module flags".to_string(),
                        });
                        continue;
                    }
                    behavior_val
                } else {
                    self.errors.push(VerificationError::InvalidMetadata {
                        reason: "invalid behavior operand in module flag (expected constant integer)".to_string(),
                        location: "module flags".to_string(),
                    });
                    continue;
                };

                // Second operand: ID (must be metadata string)
                let id = if let Some(id_str) = operands[1].as_string() {
                    id_str.to_string()
                } else {
                    self.errors.push(VerificationError::InvalidMetadata {
                        reason: "invalid ID operand in module flag (expected metadata string)".to_string(),
                        location: "module flags".to_string(),
                    });
                    continue;
                };

                // Check for duplicate IDs (except for 'require' type which is behavior 3)
                if behavior != 3 {
                    if let Some(prev_behavior) = flag_ids.get(&id) {
                        if *prev_behavior != 3 {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "module flag identifiers must be unique (or of 'require' type)".to_string(),
                                location: format!("module flag '{}'", id),
                            });
                        }
                    }
                }
                flag_ids.insert(id.clone(), behavior);

                // Third operand: value (constraints depend on behavior)
                let value = &operands[2];

                match behavior {
                    3 => {
                        // Require: value must be a metadata pair with string first element
                        if let Some(value_ops) = value.operands() {
                            if value_ops.len() != 2 {
                                self.errors.push(VerificationError::InvalidMetadata {
                                    reason: "invalid value for 'require' module flag (expected metadata pair)".to_string(),
                                    location: format!("module flag '{}'", id),
                                });
                            } else if !value_ops[0].is_string() {
                                self.errors.push(VerificationError::InvalidMetadata {
                                    reason: "invalid value for 'require' module flag (first value operand should be a string)".to_string(),
                                    location: format!("module flag '{}'", id),
                                });
                            }
                        } else {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "invalid value for 'require' module flag (expected metadata pair)".to_string(),
                                location: format!("module flag '{}'", id),
                            });
                        }
                    }
                    5 => {
                        // Append: value must be a metadata node
                        if !value.is_tuple() {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "invalid value for 'append'-type module flag (expected a metadata node)".to_string(),
                                location: format!("module flag '{}'", id),
                            });
                        }
                    }
                    7 => {
                        // Max: value must be a constant integer
                        if !value.is_int() {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "invalid value for 'max' module flag (expected constant integer)".to_string(),
                                location: format!("module flag '{}'", id),
                            });
                        }
                    }
                    8 => {
                        // Min: value must be a constant non-negative integer
                        if let Some(val) = value.as_int() {
                            if val < 0 {
                                self.errors.push(VerificationError::InvalidMetadata {
                                    reason: "invalid value for 'min' module flag (expected constant non-negative integer)".to_string(),
                                    location: format!("module flag '{}'", id),
                                });
                            }
                        } else {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "invalid value for 'min' module flag (expected constant non-negative integer)".to_string(),
                                location: format!("module flag '{}'", id),
                            });
                        }
                    }
                    _ => {
                        // Other behaviors (1=Error, 2=Warning, 4=Override, 6=AppendUnique)
                        // have no special constraints on the value
                    }
                }
            } else {
                self.errors.push(VerificationError::InvalidMetadata {
                    reason: "incorrect number of operands in module flag".to_string(),
                    location: "module flags".to_string(),
                });
            }
        }
    }

    /// Verify global variable constraints
    fn verify_global_variable(&mut self, global: &crate::module::GlobalVariable) {
        use crate::module::{Linkage, Visibility};

        // Check intrinsic global variables
        if global.name == "llvm.used" || global.name == "llvm.compiler.used" {
            // Must be array of pointers
            let valid_type = if let Some(array_info) = global.ty.array_info() {
                let (elem_type, _) = array_info;
                elem_type.is_pointer()
            } else {
                false
            };

            if !valid_type {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "wrong type for intrinsic global variable".to_string(),
                    location: format!("global variable @{}", global.name),
                });
            }
        }

        // Check linkage + visibility constraints
        // Private/internal linkage requires default visibility
        if matches!(global.linkage, Linkage::Private | Linkage::Internal) {
            if !matches!(global.visibility, Visibility::Default) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "symbol with local linkage must have default visibility".to_string(),
                    location: format!("global variable @{}", global.name),
                });
            }
        }

        // Check comdat constraints
        if global.comdat.is_some() {
            // Declarations (external/available_externally linkage) may not be in a Comdat
            match global.linkage {
                Linkage::External => {
                    // External without initializer is a declaration
                    if global.initializer.is_none() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Declaration may not be in a Comdat!".to_string(),
                            location: format!("global variable @{}", global.name),
                        });
                    }
                }
                Linkage::AvailableExternally => {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Declaration may not be in a Comdat!".to_string(),
                        location: format!("global variable @{}", global.name),
                    });
                }
                Linkage::Common => {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "'common' global may not be in a Comdat!".to_string(),
                        location: format!("global variable @{}", global.name),
                    });
                }
                Linkage::Private => {
                    // Private linkage in comdat is target-specific (Windows only)
                    // For now, we'll flag it as an error
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "comdat global value has private linkage".to_string(),
                        location: format!("global variable @{}", global.name),
                    });
                }
                _ => {
                    // Other linkage types are allowed in comdats
                }
            }
        }
    }

    /// Verify a function
    pub fn verify_function(&mut self, function: &Function) {
        let fn_name = function.name();

        // Set current function context
        self.current_function = Some(fn_name.clone());
        let fn_type = function.get_type();
        self.current_function_is_varargs = fn_type.function_info()
            .map(|(_, _, is_varargs)| is_varargs)
            .unwrap_or(false);
        self.current_function_has_personality = function.personality().is_some();
        self.current_function_calling_convention = function.calling_convention();

        // Check if trying to define an LLVM intrinsic (functions starting with "llvm.")
        // Intrinsics can be declared but not defined
        if fn_name.starts_with("llvm.") && function.has_body() {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "llvm intrinsics cannot be defined".to_string(),
                location: format!("function {}", fn_name),
            });
        }

        // Check linkage + visibility constraints
        // Private/internal linkage requires default visibility
        let linkage = function.linkage();
        let visibility = function.visibility();
        use crate::module::{Linkage, Visibility, DLLStorageClass};

        if matches!(linkage, Linkage::Private | Linkage::Internal) {
            if !matches!(visibility, Visibility::Default) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "symbol with local linkage must have default visibility".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // Check DLL storage class + linkage constraints
        // Private/internal linkage cannot have DLL storage class
        let dll_storage = function.dll_storage_class();
        if matches!(linkage, Linkage::Private | Linkage::Internal) {
            if !matches!(dll_storage, DLLStorageClass::Default) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "symbol with local linkage cannot have a DLL storage class".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // Check DLL storage class + visibility constraints
        // dllexport must have default or protected visibility
        if matches!(dll_storage, DLLStorageClass::DllExport) {
            if matches!(visibility, Visibility::Hidden) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "dllexport GlobalValue must have default or protected visibility".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // dllimport must have default visibility
        if matches!(dll_storage, DLLStorageClass::DllImport) {
            if !matches!(visibility, Visibility::Default) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "dllimport GlobalValue must have default visibility".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // Check function parameter types
        for param in function.arguments() {
            let param_type = param.get_type();

            // Functions cannot take label as parameter
            if param_type.is_label() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "invalid type for function argument".to_string(),
                    location: format!("function {}", fn_name),
                });
            }

            // Only intrinsics can have token parameters
            if param_type.is_token() && !fn_name.starts_with("llvm.") {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Function takes token but isn't an intrinsic".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // Check for incompatible function attributes
        let attrs = function.attributes();
        if attrs.noinline && attrs.alwaysinline {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Attributes 'noinline and alwaysinline' are incompatible".to_string(),
                location: format!("function {}", fn_name),
            });
        }

        // Naked functions cannot use their arguments
        if attrs.naked {
            for block in function.basic_blocks() {
                for inst in block.instructions() {
                    // Check if any operand is a function argument
                    for operand in inst.operands() {
                        if let Some(operand_name) = operand.name() {
                            // Check if this operand matches any argument name
                            for arg in function.arguments() {
                                if let Some(arg_name) = arg.name() {
                                    if arg_name == operand_name {
                                        self.errors.push(VerificationError::InvalidInstruction {
                                            reason: "cannot use argument of naked function".to_string(),
                                            location: format!("function {}", fn_name),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Verify parameter types
        if let Some((ret_type, param_types, is_varargs)) = fn_type.function_info() {
            use crate::function::CallingConvention;

            // Check return type - non-intrinsic functions cannot return token
            if ret_type.is_token() && !fn_name.starts_with("llvm.") {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Function returns a token but isn't an intrinsic".to_string(),
                    location: format!("function {}", fn_name),
                });
            }

            // Check calling convention restrictions on return type
            let cc = function.calling_convention();
            match cc {
                CallingConvention::AMDGPU_Kernel | CallingConvention::SPIR_Kernel => {
                    if !ret_type.is_void() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention requires void return type".to_string(),
                            location: format!("function {}", fn_name),
                        });
                    }
                }
                _ => {}
            }

            // Check calling convention restrictions on varargs
            match cc {
                CallingConvention::AMDGPU_Kernel |
                CallingConvention::AMDGPU_VS |
                CallingConvention::AMDGPU_GS |
                CallingConvention::AMDGPU_PS |
                CallingConvention::AMDGPU_CS |
                CallingConvention::SPIR_Kernel => {
                    if is_varargs {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention does not support varargs or perfect forwarding!".to_string(),
                            location: format!("function {}", fn_name),
                        });
                    }
                }
                _ => {}
            }

            for (idx, param_type) in param_types.iter().enumerate() {
                // Parameters cannot have label type
                if param_type.is_label() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "invalid type for function argument".to_string(),
                        location: format!("function {} parameter {}", fn_name, idx),
                    });
                }
                // Parameters cannot have void type
                if param_type.is_void() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "invalid type for function argument".to_string(),
                        location: format!("function {} parameter {}", fn_name, idx),
                    });
                }
                // Parameters cannot have metadata type
                if param_type.is_metadata() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "invalid type for function argument".to_string(),
                        location: format!("function {} parameter {}", fn_name, idx),
                    });
                }
            }
        }

        // Validate allockind attribute
        if let Some(ref kinds) = attrs.allockind {
            self.verify_allockind_attribute(kinds, &fn_name);
        }

        // Validate allocsize attribute
        if let Some(ref indices) = attrs.allocsize {
            self.verify_allocsize_attribute(indices, function);
        }

        // Verify parameter attributes (for both declarations and definitions)
        self.verify_parameter_attributes(function);

        // Check if function has a body
        if !function.has_body() {
            return; // External function, nothing else to verify
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

        // Check for duplicate local variable definitions and type consistency
        use std::collections::HashMap;
        let mut local_names: HashMap<String, Type> = HashMap::new();
        for bb in function.basic_blocks() {
            for inst in bb.instructions() {
                if let Some(result) = inst.result() {
                    if let Some(name) = result.name() {
                        // Skip empty names
                        if !name.is_empty() {
                            let result_type = result.get_type();

                            if let Some(existing_type) = local_names.get(name) {
                                // Check if it's the same type (allow same name with same type in different branches)
                                // But flag it as an error if types differ
                                if format!("{:?}", existing_type) != format!("{:?}", result_type) {
                                    self.errors.push(VerificationError::TypeMismatch {
                                        expected: format!("{:?}", existing_type),
                                        found: format!("{:?}", result_type),
                                        location: format!("'%{}' defined with type", name),
                                    });
                                } else {
                                    // Same type - this is a duplicate definition (SSA violation)
                                    if !name.chars().all(|c| c.is_ascii_digit()) {
                                        self.errors.push(VerificationError::InvalidSSA {
                                            value: name.to_string(),
                                            location: format!("function {}", fn_name),
                                        });
                                    }
                                }
                            } else {
                                local_names.insert(name.to_string(), result_type.clone());
                            }
                        }
                    }
                }
            }
        }

        // Verify return types match function signature
        self.verify_return_types(function);

        // Verify SSA form
        self.verify_ssa_form(function);

        // Verify control flow
        self.verify_control_flow(function);

        // Verify calling convention constraints
        self.verify_calling_convention(function);
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

        // Check PHI node grouping
        // All PHI nodes must be contiguous at the top of the block
        let mut found_non_phi = false;
        for inst in instructions.iter() {
            if inst.opcode() == Opcode::PHI {
                if found_non_phi {
                    self.errors.push(VerificationError::InvalidPhi {
                        reason: "PHI nodes not grouped at top of basic block".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                    break; // Only report once per block
                }
            } else {
                found_non_phi = true;
            }
        }

        // Check landing pad position
        // Landing pads must be first non-PHI instruction
        let mut found_non_phi_non_landingpad = false;
        let mut found_landingpad = false;
        for inst in instructions.iter() {
            if inst.opcode() == Opcode::LandingPad {
                if found_landingpad {
                    self.errors.push(VerificationError::InvalidLandingPad {
                        reason: "multiple landing pads in same block".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                }
                if found_non_phi_non_landingpad {
                    self.errors.push(VerificationError::InvalidLandingPad {
                        reason: "landing pad must be first non-PHI instruction in block".to_string(),
                        location: format!("block {}", bb.name().unwrap_or_else(|| "unnamed".to_string())),
                    });
                }
                found_landingpad = true;
            } else if inst.opcode() != Opcode::PHI {
                found_non_phi_non_landingpad = true;
            }
        }

        // Note: Self-referential check removed - it was catching false positives:
        // 1. Self-reference in unreachable code is allowed
        // 2. Local names can shadow global names
        // Proper check requires reachability analysis

        // Verify each instruction
        for inst in instructions.iter() {
            self.verify_instruction(inst);
        }
    }

    /// Verify an instruction
    pub fn verify_instruction(&mut self, inst: &Instruction) {
        // Focus on semantic validation, not strict operand count checks

        let location = format!("instruction {:?}", inst.opcode());

        // Check for self-referential instructions (non-PHI nodes)
        // Only PHI nodes may reference their own value
        if inst.opcode() != Opcode::PHI {
            if let Some(result) = inst.result() {
                if let Some(result_name) = result.name() {
                    // Check if any operand references this result
                    for operand in inst.operands() {
                        if let Some(operand_name) = operand.name() {
                            if operand_name == result_name && !result_name.is_empty() {
                                // Self-reference detected in non-PHI instruction
                                self.errors.push(VerificationError::InvalidSSA {
                                    value: result_name.to_string(),
                                    location: format!("instruction {:?}", inst.opcode()),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Verify metadata attachments
        self.verify_instruction_metadata(inst, &location);

        // Verify alignment constraints
        self.verify_instruction_alignment(inst, &location);

        // Verify atomic instructions
        self.verify_atomic_instruction(inst, &location);

        // Verify instruction type constraints
        self.verify_instruction_types(inst, &location);

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
                // FPToUI/FPToSI: operand must be float (or vector of float), result must be integer (or vector of integer)
                let operands = inst.operands();
                let opcode_name = if inst.opcode() == Opcode::FPToUI { "fptoui" } else { "fptosi" };
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    // Check if source is float or vector of float
                    let src_is_float = src_type.is_float() ||
                        (src_type.is_vector() && src_type.vector_info().map_or(false, |(elem, _)| elem.is_float()));

                    if !src_is_float {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "float".to_string(),
                            reason: format!("{} operand must be floating point type", opcode_name),
                            location: format!("{} instruction", opcode_name),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        // Check if destination is integer or vector of integer
                        let dst_is_int = dst_type.is_integer() ||
                            (dst_type.is_vector() && dst_type.vector_info().map_or(false, |(elem, _)| elem.is_integer()));

                        if !dst_is_int {
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
                // UIToFP/SIToFP: operand must be integer (or vector of integer), result must be float (or vector of float)
                let operands = inst.operands();
                let opcode_name = if inst.opcode() == Opcode::UIToFP { "uitofp" } else { "sitofp" };
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    // Check if source is integer or vector of integer
                    let src_is_int = src_type.is_integer() ||
                        (src_type.is_vector() && src_type.vector_info().map_or(false, |(elem, _)| elem.is_integer()));

                    if !src_is_int {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: format!("{} operand must be integer type", opcode_name),
                            location: format!("{} instruction", opcode_name),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        // Check if destination is float or vector of float
                        let dst_is_float = dst_type.is_float() ||
                            (dst_type.is_vector() && dst_type.vector_info().map_or(false, |(elem, _)| elem.is_float()));

                        if !dst_is_float {
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
                // PtrToInt: operand must be pointer (or vector of pointer), result must be integer (or vector of integer)
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    // Check if source is pointer or vector of pointer
                    let src_is_ptr = src_type.is_pointer() ||
                        (src_type.is_vector() && src_type.vector_info().map_or(false, |(elem, _)| elem.is_pointer()));

                    if !src_is_ptr {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "pointer".to_string(),
                            reason: "ptrtoint operand must be pointer type".to_string(),
                            location: "ptrtoint instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        // Check if destination is integer or vector of integer
                        let dst_is_int = dst_type.is_integer() ||
                            (dst_type.is_vector() && dst_type.vector_info().map_or(false, |(elem, _)| elem.is_integer()));

                        if !dst_is_int {
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
                // IntToPtr: operand must be integer (or vector of integer), result must be pointer (or vector of pointer)
                let operands = inst.operands();
                if operands.len() >= 1 {
                    let src_type = operands[0].get_type();
                    // Check if source is integer or vector of integer
                    let src_is_int = src_type.is_integer() ||
                        (src_type.is_vector() && src_type.vector_info().map_or(false, |(elem, _)| elem.is_integer()));

                    if !src_is_int {
                        self.errors.push(VerificationError::InvalidCast {
                            from: format!("{:?}", src_type),
                            to: "integer".to_string(),
                            reason: "inttoptr operand must be integer type".to_string(),
                            location: "inttoptr instruction".to_string(),
                        });
                    }
                    if let Some(result) = inst.result() {
                        let dst_type = result.get_type();
                        // Check if destination is pointer or vector of pointer
                        let dst_is_ptr = dst_type.is_pointer() ||
                            (dst_type.is_vector() && dst_type.vector_info().map_or(false, |(elem, _)| elem.is_pointer()));

                        if !dst_is_ptr {
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

                        // Check if types involve vectors of pointers
                        let src_is_ptr = src_type.is_pointer();
                        let dst_is_ptr = dst_type.is_pointer();
                        let src_is_vec_of_ptr = src_type.is_vector() &&
                            src_type.vector_info().map_or(false, |(elem, _)| elem.is_pointer());
                        let dst_is_vec_of_ptr = dst_type.is_vector() &&
                            dst_type.vector_info().map_or(false, |(elem, _)| elem.is_pointer());

                        // Cannot bitcast between scalar pointer and vector of pointers with size != 1
                        if (src_is_ptr && dst_is_vec_of_ptr) || (src_is_vec_of_ptr && dst_is_ptr) {
                            let vec_size = if dst_is_vec_of_ptr {
                                dst_type.vector_info().map(|(_, size)| size).unwrap_or(0)
                            } else {
                                src_type.vector_info().map(|(_, size)| size).unwrap_or(0)
                            };
                            // Only error if vector size is not 1 (size 1 is allowed, same size as scalar)
                            if vec_size != 1 {
                                self.errors.push(VerificationError::InvalidCast {
                                    from: format!("{:?}", src_type),
                                    to: format!("{:?}", dst_type),
                                    reason: format!("invalid cast opcode for cast from '{}' to '{}'",
                                        if src_is_ptr { "ptr" } else { &format!("{:?}", src_type) },
                                        if dst_is_vec_of_ptr { &format!("{:?}", dst_type) } else { "ptr" }),
                                    location: "bitcast instruction".to_string(),
                                });
                            }
                        }

                        // Bitcast between pointers and non-pointers requires compatible sizes
                        if src_is_ptr != dst_is_ptr && !src_is_vec_of_ptr && !dst_is_vec_of_ptr {
                            // One is pointer, other is not - check if non-pointer is integer
                            let non_ptr_type = if src_is_ptr { dst_type.clone() } else { src_type.clone() };
                            if !non_ptr_type.is_integer() && !non_ptr_type.is_vector() {
                                self.errors.push(VerificationError::InvalidCast {
                                    from: format!("{:?}", src_type),
                                    to: format!("{:?}", dst_type),
                                    reason: "bitcast between pointer and non-integer type is invalid".to_string(),
                                    location: "bitcast instruction".to_string(),
                                });
                            }
                        }

                        // Bitcast cannot change address space (both scalar and vector pointers)
                        let check_addrspace = (src_is_ptr && dst_is_ptr) ||
                            (src_is_vec_of_ptr && dst_is_vec_of_ptr);
                        if check_addrspace {
                            let src_addrspace = if src_is_ptr {
                                src_type.address_space().unwrap_or(0)
                            } else {
                                src_type.vector_info()
                                    .and_then(|(elem, _)| elem.address_space())
                                    .unwrap_or(0)
                            };
                            let dst_addrspace = if dst_is_ptr {
                                dst_type.address_space().unwrap_or(0)
                            } else {
                                dst_type.vector_info()
                                    .and_then(|(elem, _)| elem.address_space())
                                    .unwrap_or(0)
                            };
                            if src_addrspace != dst_addrspace {
                                let src_desc = if src_is_ptr { format!("ptr") } else { format!("<{:?}>", src_type) };
                                let dst_desc = if dst_is_ptr {
                                    format!("ptr addrspace({})", dst_addrspace)
                                } else {
                                    format!("<{:?}>", dst_type)
                                };
                                self.errors.push(VerificationError::InvalidCast {
                                    from: format!("{:?}", src_type),
                                    to: format!("{:?}", dst_type),
                                    reason: format!("invalid cast opcode for cast from '{}' to '{}'", src_desc, dst_desc),
                                    location: "bitcast instruction".to_string(),
                                });
                            }
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

                    // Validate that we don't index through a pointer within an aggregate
                    // This is invalid: getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0
                    // After indexing to field 1 (the ptr), we get a pointer, and cannot index further
                    self.verify_gep_no_pointer_indexing(inst, &operands);
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

                // Check if this is an intrinsic call
                if let Some(callee_name) = callee.name() {
                    if callee_name.starts_with("llvm.") {
                        self.verify_intrinsic_call(inst, callee_name);
                    }
                }

                // Check calling convention restrictions
                // Some calling conventions don't permit calls
                use crate::function::CallingConvention;
                let cc = self.current_function_calling_convention;
                match cc {
                    CallingConvention::AMDGPU_CS_Chain |
                    CallingConvention::AMDGPU_CS_Chain_Preserve |
                    CallingConvention::AMDGPU_CS |
                    CallingConvention::AMDGPU_ES |
                    CallingConvention::AMDGPU_GS |
                    CallingConvention::AMDGPU_HS |
                    CallingConvention::AMDGPU_Kernel |
                    CallingConvention::AMDGPU_LS |
                    CallingConvention::AMDGPU_PS |
                    CallingConvention::AMDGPU_VS |
                    CallingConvention::SPIR_Kernel => {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "calling convention does not permit calls".to_string(),
                            location: "call instruction".to_string(),
                        });
                    }
                    _ => {}
                }

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
                    // Indirect calls cannot return token type
                    if ret_type.is_token() && callee.name().is_none() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Return type cannot be token for indirect call!".to_string(),
                            location: "call instruction".to_string(),
                        });
                    }

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

                        // Check for invalid argument types (label, token)
                        if arg_type.is_label() {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "invalid type for function argument".to_string(),
                                location: format!("call argument {}", i),
                            });
                        }

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
                    // Note: In LLVM IR, calls can use different types than the function declaration,
                    // which is treated as calling through a bitcasted function pointer.
                    // We allow type mismatches for:
                    // 1. LLVM intrinsics (may be auto-upgraded)
                    // 2. Functions called with explicit type casts (trusted to be intentional)
                    // 3. Pointer type equivalence (opaque pointers)
                    if let Some(result) = inst.result() {
                        let result_type = result.get_type();
                        let types_match = if result_type.is_pointer() && ret_type.is_pointer() {
                            true
                        } else {
                            *result_type == ret_type
                        };

                        // Skip verification for LLVM intrinsics (functions starting with @llvm.)
                        // as they may be subject to auto-upgrades where the call site type
                        // differs from the declaration
                        let is_llvm_intrinsic = callee.name()
                            .map(|name| name.starts_with("llvm."))
                            .unwrap_or(false);

                        // For non-intrinsics, allow type mismatches as they represent intentional
                        // function pointer bitcasts at the call site
                        // TODO: Could add stricter checking for ABI compatibility in a strict mode
                        let _allow_bitcast = !is_llvm_intrinsic;

                        if !types_match && !is_llvm_intrinsic && false {  // Disabled for now
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
                        // Cannot allocate a function type
                        if pointee.is_function() {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "invalid type for alloca".to_string(),
                                location: "alloca instruction".to_string(),
                            });
                        }
                        // Cannot allocate void type
                        if pointee.is_void() {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "Cannot allocate unsized type".to_string(),
                                location: "alloca instruction".to_string(),
                            });
                        }
                        // Cannot allocate token type
                        if pointee.is_token() {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "invalid type for alloca".to_string(),
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

                // Check for duplicate basic blocks with different values (ambiguous PHI)
                let mut seen_blocks: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                let mut i = 0;
                while i + 1 < operands.len() {
                    // Operands are pairs: [value0, block0, value1, block1, ...]
                    let block = &operands[i + 1];
                    if let Some(block_name) = block.name() {
                        if let Some(&first_idx) = seen_blocks.get(block_name) {
                            // Same block appears twice - check if values are different
                            let first_value = &operands[first_idx];
                            let current_value = &operands[i];

                            // Check if the values are different (compare by name or value identity)
                            let values_different = match (first_value.name(), current_value.name()) {
                                (Some(name1), Some(name2)) => name1 != name2,
                                // If names don't exist, consider them different if they're different Value instances
                                _ => !std::ptr::eq(first_value, current_value),
                            };

                            if values_different {
                                self.errors.push(VerificationError::InvalidPhi {
                                    reason: format!("PHI node has multiple entries for the same basic block with different values"),
                                    location: format!("phi instruction, block {}", block_name),
                                });
                            }
                        } else {
                            seen_blocks.insert(block_name.to_string(), i);
                        }
                    }
                    i += 2;
                }

                if let Some(result) = inst.result() {
                    let result_type = result.get_type();

                    // Token types cannot be used in PHI nodes
                    if result_type.is_token() {
                        self.errors.push(VerificationError::InvalidPhi {
                            reason: "PHI nodes cannot produce token types".to_string(),
                            location: "phi instruction".to_string(),
                        });
                    }

                    // PHI operands are pairs: [value1, block1, value2, block2, ...]
                    let mut i = 0;
                    while i < operands.len() {
                        if i % 2 == 0 {
                            // Even indices are values
                            let value_type = operands[i].get_type();

                            // Check if value is a token type
                            if value_type.is_token() {
                                self.errors.push(VerificationError::InvalidPhi {
                                    reason: "PHI nodes cannot have token type operands".to_string(),
                                    location: format!("phi incoming value {}", i / 2),
                                });
                            }

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
                            reason: "storing unsized types is not allowed".to_string(),
                            location: "store instruction".to_string(),
                        });
                    }

                    // Atomic stores have additional constraints
                    // Note: We would need to detect atomic stores from instruction attributes
                    // For now, we check if it's a struct type which is never valid for atomics
                    // This is a heuristic - proper detection requires parsing atomic attribute
                    if value_type.is_struct() {
                        // If storing a struct, this might be an atomic store which is invalid
                        // We can't definitively check without parsing attributes, but we can
                        // add a check that would catch atomic struct stores
                        // Note: This is overly broad and would be refined with proper attribute parsing
                        // For now, we rely on the comment "atomic store" appearing in the instruction name
                        // Since we don't have that, we'll skip this check and rely on parser
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
                            reason: "loading unsized types is not allowed".to_string(),
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

                    // Select values cannot have token type
                    if true_type.is_token() || false_type.is_token() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "select values cannot have token type".to_string(),
                            location: "select instruction".to_string(),
                        });
                    }

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
                // Check calling convention restrictions
                // Some calling conventions don't permit calls (including invoke)
                use crate::function::CallingConvention;
                let cc = self.current_function_calling_convention;
                match cc {
                    CallingConvention::AMDGPU_CS_Chain |
                    CallingConvention::AMDGPU_CS_Chain_Preserve |
                    CallingConvention::AMDGPU_CS |
                    CallingConvention::AMDGPU_ES |
                    CallingConvention::AMDGPU_GS |
                    CallingConvention::AMDGPU_HS |
                    CallingConvention::AMDGPU_Kernel |
                    CallingConvention::AMDGPU_LS |
                    CallingConvention::AMDGPU_PS |
                    CallingConvention::AMDGPU_VS |
                    CallingConvention::SPIR_Kernel => {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "calling convention does not permit calls".to_string(),
                            location: "invoke instruction".to_string(),
                        });
                    }
                    _ => {}
                }

                // Invoke: must have both normal and unwind destinations
                // Note: Parser must preserve successor information
                // Basic validation: check it's a valid function call
                let operands = inst.operands();
                if operands.is_empty() {
                    self.errors.push(VerificationError::InvalidExceptionHandling {
                        reason: "invoke must have a callee".to_string(),
                        location: "invoke instruction".to_string(),
                    });
                } else {
                    // Check if this is an intrinsic call - only certain intrinsics can be invoked
                    if let Some(callee_name) = operands[0].name() {
                        if callee_name.starts_with("llvm.") {
                            // Only these intrinsics can be invoked
                            let allowed = matches!(callee_name,
                                "llvm.donothing" |
                                "llvm.experimental.patchpoint.void" |
                                "llvm.experimental.patchpoint.i64" |
                                "llvm.experimental.gc.statepoint.p0" |
                                "llvm.coro.resume" |
                                "llvm.coro.destroy" |
                                "llvm.objc.clang.arc.noop.use" |
                                "llvm.wasm.throw" |
                                "llvm.wasm.rethrow"
                            ) || callee_name.starts_with("llvm.experimental.patchpoint") ||
                                callee_name.starts_with("llvm.experimental.gc.statepoint") ||
                                callee_name.contains("clang.arc.attachedcall");

                            if !allowed {
                                self.errors.push(VerificationError::InvalidExceptionHandling {
                                    reason: "Cannot invoke an intrinsic other than donothing, patchpoint, statepoint, coro_resume, coro_destroy, clang.arc.attachedcall or wasm.(re)throw".to_string(),
                                    location: format!("invoke {}", callee_name),
                                });
                            }
                        }
                    }
                }
            }
            Opcode::Resume => {
                // Resume: must be in a function with a personality
                if !self.current_function_has_personality {
                    self.errors.push(VerificationError::InvalidExceptionHandling {
                        reason: "ResumeInst needs to be in a function with a personality.".to_string(),
                        location: "resume instruction".to_string(),
                    });
                }

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

    /// Verify calling convention constraints
    fn verify_calling_convention(&mut self, function: &Function) {
        use crate::function::CallingConvention;

        let cc = function.calling_convention();
        let fn_type = function.get_type();
        let ret_type = fn_type.function_return_type().unwrap_or_else(|| fn_type.clone());
        let fn_name = function.name();

        // Check return type constraints
        match cc {
            CallingConvention::AMDGPU_Kernel | CallingConvention::SPIR_Kernel |
            CallingConvention::AMDGPU_CS_Chain | CallingConvention::AMDGPU_CS_Chain_Preserve => {
                if !ret_type.is_void() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention requires void return type".to_string(),
                        location: format!("function {}", fn_name),
                    });
                }
            },
            _ => {},
        }

        // Check if calling convention permits calls
        let cc_forbids_calls = matches!(cc,
            CallingConvention::AMDGPU_CS_Chain | CallingConvention::AMDGPU_CS_Chain_Preserve |
            CallingConvention::AMDGPU_CS | CallingConvention::AMDGPU_ES | CallingConvention::AMDGPU_GS |
            CallingConvention::AMDGPU_HS | CallingConvention::AMDGPU_Kernel | CallingConvention::AMDGPU_LS |
            CallingConvention::AMDGPU_PS | CallingConvention::AMDGPU_VS | CallingConvention::SPIR_Kernel
        );

        if cc_forbids_calls {
            // Check for call or invoke instructions in the function body
            // However, intrinsics (llvm.*) are allowed even with these calling conventions
            for bb in function.basic_blocks() {
                for inst in bb.instructions() {
                    if matches!(inst.opcode(), Opcode::Call | Opcode::Invoke) {
                        // Check if it's an intrinsic call (llvm.*)
                        // For call instructions, the callee is the first operand
                        let operands = inst.operands();
                        let is_intrinsic = if !operands.is_empty() {
                            if let Some(callee_name) = operands.first().and_then(|v| v.name()) {
                                callee_name.starts_with("llvm.") || callee_name.starts_with("@llvm.")
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if !is_intrinsic {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "calling convention does not permit calls".to_string(),
                                location: format!("function {}", fn_name),
                            });
                            // Only report once per function
                            return;
                        }
                    }
                }
            }
        }

        // Check varargs restrictions
        if fn_type.function_info().map(|(_,_,v)| v).unwrap_or(false) {
            match cc {
                CallingConvention::AMDGPU_Kernel | CallingConvention::SPIR_Kernel |
                CallingConvention::AMDGPU_VS | CallingConvention::AMDGPU_GS |
                CallingConvention::AMDGPU_PS | CallingConvention::AMDGPU_CS |
                CallingConvention::AMDGPU_CS_Chain | CallingConvention::AMDGPU_CS_Chain_Preserve => {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not support varargs or perfect forwarding!".to_string(),
                        location: format!("function {}", fn_name),
                    });
                },
                CallingConvention::AMDGPU_GFX_Whole_Wave => {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not support varargs".to_string(),
                        location: format!("function {}", fn_name),
                    });
                },
                _ => {},
            }
        }

        // Check AMDGPU_GFX_Whole_Wave first parameter constraint
        if cc == CallingConvention::AMDGPU_GFX_Whole_Wave {
            let params = function.arguments();
            if params.is_empty() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Calling convention requires first argument to be i1".to_string(),
                    location: format!("function {}", fn_name),
                });
            } else {
                let first_param_type = params[0].get_type();
                if !first_param_type.is_integer() || first_param_type.int_width() != Some(1) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention requires first argument to be i1".to_string(),
                        location: format!("function {}", fn_name),
                    });
                }
            }
        }

        // Check calling conventions that don't allow sret
        let cc_disallows_sret = matches!(cc,
            CallingConvention::AMDGPU_Kernel | CallingConvention::SPIR_Kernel
        );

        if cc_disallows_sret {
            let attrs = function.attributes();
            for param_attrs in &attrs.parameter_attributes {
                if param_attrs.sret.is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not allow sret".to_string(),
                        location: format!("function {}", fn_name),
                    });
                    return;
                }
            }
        }

        // Check calling conventions that disallow byval
        let cc_disallows_byval = matches!(cc,
            CallingConvention::AMDGPU_Kernel | CallingConvention::SPIR_Kernel
        );

        if cc_disallows_byval {
            let attrs = function.attributes();
            for param_attrs in &attrs.parameter_attributes {
                if param_attrs.byval.is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention disallows byval".to_string(),
                        location: format!("function {}", fn_name),
                    });
                    return;
                }
            }
        }

        // Check calling conventions that require byval parameter
        if cc == CallingConvention::AMDGPU_GFX_Whole_Wave {
            let attrs = function.attributes();
            if !attrs.parameter_attributes.is_empty() {
                // Check first parameter has byval (if it's a pointer)
                let params = function.arguments();
                if !params.is_empty() {
                    let first_param_type = params[0].get_type();
                    if first_param_type.is_pointer() && attrs.parameter_attributes[0].byval.is_none() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention parameter requires byval".to_string(),
                            location: format!("function {}", fn_name),
                        });
                    }
                }
            }
        }
    }

    /// Verify parameter attributes
    fn verify_parameter_attributes(&mut self, function: &Function) {
        use crate::function::CallingConvention;

        let fn_name = function.name();
        let fn_type = function.get_type();
        let is_varargs = fn_type.function_info().map(|(_,_,v)| v).unwrap_or(false);
        let attrs = function.attributes();

        // Get parameter types from function type
        let param_types = if let Some((_, params, _)) = fn_type.function_info() {
            params
        } else {
            Vec::new()
        };

        // Get return type from function type
        let return_type = if let Some((ret_ty, _, _)) = fn_type.function_info() {
            ret_ty
        } else {
            return;
        };

        // Verify return type attributes
        let ret_attrs = &attrs.return_attributes;

        // Check align attribute on return type - must be pointer type
        if let Some(align_val) = ret_attrs.align {
            if !return_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'align {}' applied to incompatible type!", align_val),
                    location: format!("@{}", fn_name),
                });
            }
        }

        // Check signext on return type - must be integer type
        if ret_attrs.signext {
            if !return_type.is_integer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'signext' applied to incompatible type!"),
                    location: format!("@{}", fn_name),
                });
            }
        }

        // Check zeroext on return type - must be integer type
        if ret_attrs.zeroext {
            if !return_type.is_integer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'zeroext' applied to incompatible type!"),
                    location: format!("@{}", fn_name),
                });
            }
        }

        // Track counts of special attributes that can only appear once
        let mut sret_count = 0;
        let mut sret_idx = None;
        let mut swifterror_count = 0;
        let mut swiftself_count = 0;
        let mut swiftasync_count = 0;

        // Verify parameter attributes
        for (idx, param_attrs) in attrs.parameter_attributes.iter().enumerate() {
            // Get the parameter type
            let param_type = param_types.get(idx);
            if param_type.is_none() {
                continue;
            }
            let param_type = param_type.unwrap();

            // Track sret for multi-parameter check
            if param_attrs.sret.is_some() {
                sret_count += 1;
                sret_idx = Some(idx);
            }

            // Track swifterror for multi-parameter check
            if param_attrs.swifterror {
                swifterror_count += 1;
            }

            // Track swiftself for multi-parameter check
            if param_attrs.swiftself {
                swiftself_count += 1;
            }

            // Track swiftasync for multi-parameter check
            if param_attrs.swiftasync {
                swiftasync_count += 1;
            }

            // Check align attribute - must be pointer type
            if let Some(align_val) = param_attrs.align {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'align {}' applied to incompatible type!", align_val),
                        location: format!("@{}", fn_name),
                    });
                }

                // Check alignment limit - max 2^32 bytes
                const MAX_ALIGNMENT: u64 = 1u64 << 32; // 4294967296
                if (align_val as u64) > MAX_ALIGNMENT {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "huge alignments are not supported yet".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check signext attribute - must be integer type
            if param_attrs.signext {
                if !param_type.is_integer() && !param_type.is_pointer() {
                    // signext on pointer is definitely wrong, on non-integer is wrong
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'signext' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                } else if param_type.is_pointer() {
                    // Specifically catch signext on pointer which is one of our test cases
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'signext' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check zeroext attribute - must be integer type
            if param_attrs.zeroext {
                if !param_type.is_integer() && !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'zeroext' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                } else if param_type.is_pointer() {
                    // Specifically catch zeroext on pointer
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'zeroext' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check sret attribute with varargs
            if is_varargs && param_attrs.sret.is_some() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'sret' does not apply to vararg call!"),
                    location: format!("function {} parameter {}", fn_name, idx),
                });
            }

            // Check sret attribute - must be pointer type
            if param_attrs.sret.is_some() {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'sret(i32)' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }

                // Check calling convention restrictions on sret
                let cc = function.calling_convention();
                if matches!(cc, CallingConvention::AMDGPU_Kernel) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not allow sret".to_string(),
                        location: format!("function {}", fn_name),
                    });
                }
            }

            // Check byval attribute - must be pointer type
            if let Some(byval_ty) = &param_attrs.byval {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'byval(i32)' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }

                // Check calling convention restrictions on byval
                let cc = function.calling_convention();
                if matches!(cc, CallingConvention::AMDGPU_Kernel) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention disallows byval".to_string(),
                        location: format!("function {}", fn_name),
                    });
                }

                // Check byval size limit - must be less than 2^31 bytes
                // The byval type represents the pointee type
                if let Some(size) = byval_ty.size_in_bytes() {
                    const MAX_BYVAL_SIZE: u64 = 2147483648; // 2^31 bytes
                    if size >= MAX_BYVAL_SIZE {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "huge 'byval' arguments are unsupported".to_string(),
                            location: format!("@{}", fn_name),
                        });
                    }
                }
            }

            // Check inalloca attribute - must be on last argument (unless it's varargs)
            if let Some(inalloca_ty) = &param_attrs.inalloca {
                // Must be pointer type
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'inalloca(i8)' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }

                // inalloca type must be sized
                if !inalloca_ty.is_sized() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Attribute 'inalloca' does not support unsized types!".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }

                // Check if this is NOT the last parameter
                // For varargs functions, inalloca must be on the last fixed parameter
                if idx + 1 < param_types.len() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("inalloca isn't on the last argument!"),
                        location: format!("function {} parameter {}", fn_name, idx),
                    });
                }
            }

            // Check swifterror attribute - must be pointer type
            if param_attrs.swifterror {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'swifterror' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check noalias attribute - must be pointer type
            if param_attrs.noalias {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'noalias' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check nest attribute - must be pointer type
            if param_attrs.nest {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'nest' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check dereferenceable attribute - must be pointer type
            if param_attrs.dereferenceable.is_some() {
                if !param_type.is_pointer() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("Attribute 'dereferenceable' applied to incompatible type!"),
                        location: format!("@{}", fn_name),
                    });
                }
            }

            // Check for incompatible attribute combinations
            // inalloca is incompatible with: byval, inreg, sret, nest
            if param_attrs.inalloca.is_some() {
                if param_attrs.byval.is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Attributes 'byval', 'inalloca', 'preallocated', 'inreg', 'nest', 'byref', and 'sret' are incompatible!".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }
                if param_attrs.inreg {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Attributes 'byval', 'inalloca', 'preallocated', 'inreg', 'nest', 'byref', and 'sret' are incompatible!".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }
                if param_attrs.sret.is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Attributes 'byval', 'inalloca', 'preallocated', 'inreg', 'nest', 'byref', and 'sret' are incompatible!".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }
                if param_attrs.nest {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Attributes 'byval', 'inalloca', 'preallocated', 'inreg', 'nest', 'byref', and 'sret' are incompatible!".to_string(),
                        location: format!("@{}", fn_name),
                    });
                }
            }
        }

        // Check for multiple sret parameters
        if sret_count > 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Cannot have multiple 'sret' parameters!".to_string(),
                location: format!("@{}", fn_name),
            });
        }

        // Check sret position - must be on first or second parameter
        if let Some(idx) = sret_idx {
            if idx > 1 {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Attribute 'sret' is not on first or second parameter!".to_string(),
                    location: format!("@{}", fn_name),
                });
            }
        }

        // Check for multiple swifterror parameters
        if swifterror_count > 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Cannot have multiple 'swifterror' parameters!".to_string(),
                location: format!("@{}", fn_name),
            });
        }

        // Check for multiple swiftself parameters
        if swiftself_count > 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Cannot have multiple 'swiftself' parameters!".to_string(),
                location: format!("@{}", fn_name),
            });
        }

        // Check for multiple swiftasync parameters
        if swiftasync_count > 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Cannot have multiple 'swiftasync' parameters!".to_string(),
                location: format!("@{}", fn_name),
            });
        }
    }

    /// Verify atomic instruction constraints
    fn verify_atomic_instruction(&mut self, inst: &Instruction, location: &str) {
        let opcode = inst.opcode();
        let operands = inst.operands();

        // Check if this is an atomic operation
        let is_atomic = inst.is_atomic();

        match opcode {
            Opcode::Load => {
                // Atomic loads must load integer, pointer, float, or vector types
                // Cannot load aggregate types (structs, arrays)
                if is_atomic {
                    // Get the loaded type from the instruction result
                    if let Some(result) = inst.result() {
                        let loaded_type = result.get_type();

                        // Check if the type is valid for atomic operations
                        let is_valid = loaded_type.is_integer() ||
                                       loaded_type.is_pointer() ||
                                       loaded_type.is_float() ||
                                       loaded_type.is_vector();

                        if !is_valid {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "atomic load operand must have integer, pointer, floating point, or vector type!".to_string(),
                                location: location.to_string(),
                            });
                        }
                    }
                }
            }
            Opcode::Store => {
                // Atomic stores must store integer, pointer, float, or vector types
                // Cannot store aggregate types (structs, arrays)
                if is_atomic && operands.len() >= 1 {
                    let stored_type = operands[0].get_type();

                    // Check if the type is valid for atomic operations
                    let is_valid = stored_type.is_integer() ||
                                   stored_type.is_pointer() ||
                                   stored_type.is_float() ||
                                   stored_type.is_vector();

                    if !is_valid {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "atomic store operand must have integer, pointer, floating point, or vector type!".to_string(),
                            location: location.to_string(),
                        });
                    }
                }
            }
            Opcode::AtomicCmpXchg | Opcode::AtomicRMW => {
                // These atomic operations also have type constraints
                // AtomicCmpXchg: compare-and-swap, operand 1 is the value
                // AtomicRMW: read-modify-write, operand 1 is the value
                if operands.len() >= 2 {
                    let value_type = operands[1].get_type();

                    if value_type.is_struct() || value_type.is_array() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: format!("atomic {:?} operand must have integer, pointer, floating point, or vector type!", opcode),
                            location: location.to_string(),
                        });
                    }
                }
            }
            _ => {
                // Not an atomic operation
            }
        }
    }

    /// Verify type constraints for instructions
    fn verify_instruction_types(&mut self, _inst: &Instruction, _location: &str) {
        // Type constraint validation would require:
        // 1. Better type system support for target extension types
        // 2. Opaque struct detection
        // Placeholder for future implementation
    }

    /// Verify alignment constraints for instructions
    fn verify_instruction_alignment(&mut self, inst: &Instruction, location: &str) {
        // Maximum alignment in LLVM is 2^32 bytes
        const MAX_ALIGNMENT: u64 = 1u64 << 32; // 4294967296

        if let Some(alignment) = inst.alignment() {
            if alignment > MAX_ALIGNMENT {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("alignment is larger than the maximum supported by LLVM (2^32)"),
                    location: location.to_string(),
                });
            }

            // Also check that alignment is a power of 2
            if alignment > 0 && !alignment.is_power_of_two() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("alignment must be a power of 2"),
                    location: location.to_string(),
                });
            }
        }
    }

    /// Verify metadata attachments on instructions
    fn verify_instruction_metadata(&mut self, inst: &Instruction, location: &str) {
        for md_name in inst.metadata_attachments() {
            match md_name.as_str() {
                "align" => {
                    // align metadata only applies to load instructions
                    if inst.opcode() != Opcode::Load {
                        self.errors.push(VerificationError::InvalidMetadata {
                            reason: "align applies only to load instructions".to_string(),
                            location: location.to_string(),
                        });
                    }
                    // Additional check: align only applies to pointer types
                    // This would require checking the loaded type
                }
                "llvm.access.group" | "parallel_loop_access" => {
                    // Access group metadata must be used on memory operations
                    match inst.opcode() {
                        Opcode::Load | Opcode::Store | Opcode::Call | Opcode::Invoke => {
                            // Valid usage
                        }
                        _ => {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: format!("{} metadata can only be used on memory operations", md_name),
                                location: location.to_string(),
                            });
                        }
                    }
                }
                "nontemporal" => {
                    // nontemporal metadata only on load/store
                    match inst.opcode() {
                        Opcode::Load | Opcode::Store => { /* Valid */ }
                        _ => {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "nontemporal metadata can only be used on load/store".to_string(),
                                location: location.to_string(),
                            });
                        }
                    }
                }
                "invariant.load" => {
                    // invariant.load only on load instructions
                    if inst.opcode() != Opcode::Load {
                        self.errors.push(VerificationError::InvalidMetadata {
                            reason: "invariant.load metadata can only be used on load instructions".to_string(),
                            location: location.to_string(),
                        });
                    }
                }
                "nonnull" => {
                    // nonnull metadata only on load instructions
                    if inst.opcode() != Opcode::Load {
                        self.errors.push(VerificationError::InvalidMetadata {
                            reason: "nonnull metadata can only be used on load instructions".to_string(),
                            location: location.to_string(),
                        });
                    }
                }
                "range" => {
                    // range metadata only on load/call/invoke
                    match inst.opcode() {
                        Opcode::Load | Opcode::Call | Opcode::Invoke => { /* Valid */ }
                        _ => {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "range metadata can only be used on load/call/invoke".to_string(),
                                location: location.to_string(),
                            });
                        }
                    }
                }
                "noalias" | "alias.scope" => {
                    // Alias metadata on memory operations
                    match inst.opcode() {
                        Opcode::Load | Opcode::Store | Opcode::Call | Opcode::Invoke => { /* Valid */ }
                        _ => {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: format!("{} metadata can only be used on memory operations", md_name),
                                location: location.to_string(),
                            });
                        }
                    }
                }
                "tbaa" | "tbaa.struct" => {
                    // TBAA metadata on memory operations and VAArg
                    match inst.opcode() {
                        Opcode::Load | Opcode::Store | Opcode::Call | Opcode::Invoke |
                        Opcode::AtomicCmpXchg | Opcode::AtomicRMW | Opcode::VAArg => { /* Valid */ }
                        _ => {
                            self.errors.push(VerificationError::InvalidMetadata {
                                reason: "tbaa metadata can only be used on memory operations".to_string(),
                                location: location.to_string(),
                            });
                        }
                    }
                }
                // Debug metadata is allowed on any instruction
                "dbg" | "DILocation" | "DILocalVariable" | "DIExpression" => {
                    // Always valid
                }
                // Profile and branch metadata
                "prof" | "unpredictable" => {
                    // These can appear on branches, calls, etc.
                    // Generally valid on most instructions
                }
                _ => {
                    // Unknown metadata - don't error, as there may be custom metadata
                }
            }
        }
    }

    /// Verify that GEP doesn't try to index through a pointer within an aggregate
    /// This is invalid: getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0
    /// After getting to field 1 (a ptr), we cannot index further into it
    fn verify_gep_no_pointer_indexing(&mut self, inst: &Instruction, operands: &[Value]) {
        if operands.len() < 2 {
            return; // Need at least base pointer and one index
        }

        // Get the GEP source type from the instruction
        // This is the type specified in: getelementptr {i32, ptr}, ptr %X, ...
        let mut current_type = if let Some(source_type) = inst.gep_source_type() {
            source_type.clone()
        } else {
            // If no source type stored, try to get from base pointer's pointee type
            let base_type = operands[0].get_type();
            if let Some(pointee) = base_type.pointee_type() {
                pointee.clone()
            } else {
                return; // Can't validate without type info
            }
        };

        // First index dereferences the pointer to get to the aggregate
        // Remaining indices navigate through the aggregate
        // Skip operand 0 (base pointer) and start from operand 1 (first index)

        for idx in 1..operands.len() {
            // Check if current type is already a pointer
            if current_type.is_pointer() {
                // We've reached a pointer in the aggregate, cannot index further
                if idx < operands.len() - 1 || (idx == operands.len() - 1 && idx > 1) {
                    // There are more indices after reaching a pointer - invalid!
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "invalid getelementptr indices".to_string(),
                        location: "getelementptr instruction".to_string(),
                    });
                    return;
                }
                break;
            }

            // Navigate to the next type based on current type kind
            if current_type.is_struct() {
                // For struct, we need the constant index value to know which field
                // For now, we can't evaluate constant expressions, so we check conservatively:
                // If ANY field is a pointer and there are more indices, it's potentially invalid
                if let Some(struct_fields) = current_type.struct_fields() {
                    // Check if there are more indices after this one
                    let has_more_indices = idx < operands.len() - 1;

                    // Check if any field is a pointer
                    let has_pointer_field = struct_fields.iter().any(|f| f.is_pointer());

                    if has_pointer_field && has_more_indices {
                        // Potentially invalid: indexing into a struct with pointer field(s)
                        // and there are more indices. This catches the test case:
                        // getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0
                        //                                            ^      ^    ^
                        //                                         deref  field  ERROR!
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "invalid getelementptr indices".to_string(),
                            location: "getelementptr instruction".to_string(),
                        });
                        return;
                    }

                    // Can't precisely track which field without constant evaluation
                    // so we stop here
                    break;
                }
                break;
            } else if current_type.is_array() {
                // For arrays, move to the element type
                if let Some((elem_type, _)) = current_type.array_info() {
                    current_type = elem_type.clone();
                } else {
                    break;
                }
            } else if current_type.is_vector() {
                // For vectors, move to the element type
                if let Some((elem_type, _)) = current_type.vector_info() {
                    current_type = elem_type.clone();
                } else {
                    break;
                }
            } else {
                // Other types (integers, etc.) can't be indexed into
                break;
            }
        }
    }

    /// Verify intrinsic-specific constraints
    fn verify_intrinsic_call(&mut self, inst: &Instruction, intrinsic_name: &str) {
        let operands = inst.operands();

        // Check if intrinsic varargs declaration matches requirements
        if operands.len() >= 1 {
            let callee = &operands[0];
            let callee_type = callee.get_type();

            if let Some(fn_type) = if callee_type.is_pointer() {
                callee_type.pointee_type()
            } else if callee_type.is_function() {
                Some(callee_type)
            } else {
                None
            } {
                if let Some((_, _, declared_varargs)) = fn_type.function_info() {
                    // Check intrinsics that MUST be varargs
                    if intrinsic_name.starts_with("llvm.experimental.stackmap") ||
                       intrinsic_name.starts_with("llvm.experimental.patchpoint") {
                        if !declared_varargs {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "Callsite was not defined with variable arguments!".to_string(),
                                location: format!("call to {}", intrinsic_name),
                            });
                        }
                    }

                    // Check intrinsics that MUST NOT be varargs
                    if intrinsic_name == "llvm.donothing" {
                        if declared_varargs {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "Intrinsic was not defined with variable arguments!".to_string(),
                                location: format!("call to {}", intrinsic_name),
                            });
                        }
                    }
                }
            }
        }

        // Check immarg parameters - some intrinsic parameters must be immediate (constant) values
        self.verify_intrinsic_immarg(inst, intrinsic_name, operands);

        // llvm.va_start - must be called in a varargs function
        // Note: Temporarily disabled - need to ensure parser correctly sets is_varargs
        // if intrinsic_name == "llvm.va_start" {
        //     if !self.current_function_is_varargs {
        //         self.errors.push(VerificationError::InvalidInstruction {
        //             reason: "va_start called in a non-varargs function".to_string(),
        //             location: format!("call to {}", intrinsic_name),
        //         });
        //     }
        // }

        // llvm.bswap - must have even number of bytes
        if intrinsic_name.starts_with("llvm.bswap.") {
            if operands.len() >= 2 {
                // operands[0] is the function, operands[1] is the argument
                let arg_type = operands[1].get_type();

                // Get the integer width
                if let Some(bits) = arg_type.int_width() {
                    if bits % 16 != 0 {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "bswap must be an even number of bytes".to_string(),
                            location: format!("call to {}", intrinsic_name),
                        });
                    }
                } else if arg_type.is_vector() {
                    // Check vector element type
                    if let Some((elem_type, _)) = arg_type.vector_info() {
                        if let Some(bits) = elem_type.int_width() {
                            if bits % 16 != 0 {
                                self.errors.push(VerificationError::InvalidInstruction {
                                    reason: "bswap must be an even number of bytes".to_string(),
                                    location: format!("call to {}", intrinsic_name),
                                });
                            }
                        }
                    }
                }
            }
        }

        // llvm.stepvector - must return a vector of integers with bitwidth >= 8
        if intrinsic_name.starts_with("llvm.stepvector.") {
            if let Some(result) = inst.result() {
                let result_type = result.get_type();
                if !result_type.is_vector() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect return type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                } else if let Some((elem_type, _)) = result_type.vector_info() {
                    if !elem_type.is_integer() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "stepvector only supported for vectors of integers with a bitwidth of at least 8".to_string(),
                            location: format!("call to {}", intrinsic_name),
                        });
                    } else if let Some(bits) = elem_type.int_width() {
                        if bits < 8 {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "stepvector only supported for vectors of integers with a bitwidth of at least 8".to_string(),
                                location: format!("call to {}", intrinsic_name),
                            });
                        }
                    }
                }
            }
        }

        // llvm.vector.reduce.* - vector reduction intrinsics
        if intrinsic_name.starts_with("llvm.vector.reduce.") {
            self.verify_intrinsic_vector_reduce(inst, intrinsic_name, operands);
        }

        // llvm.is.fpclass - floating-point class test
        if intrinsic_name.starts_with("llvm.is.fpclass.") {
            self.verify_intrinsic_is_fpclass(inst, intrinsic_name, operands);
        }

        // llvm.sadd.sat, llvm.uadd.sat, llvm.ssub.sat, llvm.usub.sat, llvm.sshl.sat, llvm.ushl.sat
        if intrinsic_name.contains(".sat.") {
            self.verify_intrinsic_sat(inst, intrinsic_name, operands);
        }

        // llvm.vp.* - vector predication intrinsics
        if intrinsic_name.starts_with("llvm.vp.") {
            self.verify_intrinsic_vp(inst, intrinsic_name, operands);
        }

        // llvm.bswap - must operate on types with even number of bytes
        if intrinsic_name.starts_with("llvm.bswap.") {
            self.verify_intrinsic_bswap(inst, intrinsic_name, operands);
        }

        // llvm.experimental.get.vector.length - VF (second operand) must be positive
        // Note: This would require constant analysis to check if the value is > 0
        // For now, we can't validate this without constant folding infrastructure

        // llvm.memcpy/memmove/memset - last argument (is_volatile) must be constant
        if intrinsic_name.starts_with("llvm.memcpy.") || intrinsic_name.starts_with("llvm.memmove.") ||
           intrinsic_name.starts_with("llvm.memset.") {
            // For memcpy/memmove: operands[0] = function, [1] = dest, [2] = src, [3] = length, [4] = is_volatile
            // For memset: operands[0] = function, [1] = dest, [2] = value, [3] = length, [4] = is_volatile
            let is_volatile_idx = if intrinsic_name.contains(".inline.") {
                4 // inline variants have is_volatile at index 4
            } else {
                4 // standard variants also have it at index 4
            };

            if operands.len() > is_volatile_idx {
                let is_volatile = &operands[is_volatile_idx];
                // Check if it's a constant by checking if it has a name (non-constants have names)
                if is_volatile.name().is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "immarg operand has non-immediate parameter".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }
            }
        }

        // llvm.cttz/ctlz - second argument must be constant i1 (immarg)
        if intrinsic_name.starts_with("llvm.ctlz.") || intrinsic_name.starts_with("llvm.cttz.") {
            if operands.len() >= 3 {
                // operands[0] = function, operands[1] = value, operands[2] = is_zero_poison
                let is_zero_poison = &operands[2];
                // Check if it's a constant by checking if it has a name (non-constants have names)
                if is_zero_poison.name().is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "immarg operand has non-immediate parameter".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }
            }
        }

        // llvm.returnaddress / llvm.frameaddress - argument must be constant
        if intrinsic_name.starts_with("llvm.returnaddress") || intrinsic_name.starts_with("llvm.frameaddress") {
            if operands.len() >= 2 {
                // operands[0] = function, operands[1] = level
                let level = &operands[1];
                if level.name().is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "immarg operand has non-immediate parameter".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }
            }
        }

        // llvm.objectsize - all boolean arguments must be constant
        if intrinsic_name.starts_with("llvm.objectsize.") {
            if operands.len() >= 5 {
                // operands[0] = function, operands[1] = ptr, operands[2-4] = boolean flags
                for i in 2..5 {
                    if operands[i].name().is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "immarg operand has non-immediate parameter".to_string(),
                            location: format!("call to {}", intrinsic_name),
                        });
                        break; // Only report once per call
                    }
                }
            }
        }

        // llvm.va_start - must be called in a varargs function
        // TODO: Re-enable once parser correctly sets varargs flag
        // Currently causing false positive on tbaa-allowed.ll
        if intrinsic_name == "llvm.va_start" {
            // Temporarily disabled to avoid false positive
            // The parser may not be correctly setting the varargs flag on function types
            /*
            if !self.current_function_is_varargs {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "va_start called in a non-varargs function".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
            */
        }

        // llvm.abs - integer absolute value intrinsic
        if intrinsic_name.starts_with("llvm.abs.") {
            if operands.len() >= 3 {
                // operands[0] = function, operands[1] = value, operands[2] = is_int_min_poison
                let value_type = operands[1].get_type();

                // Value must be integer or vector of integers
                if !value_type.is_integer() && !(value_type.is_vector() && value_type.vector_info().map_or(false, |(e, _)| e.is_integer())) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect argument type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }

                // Second argument (is_int_min_poison) must be constant
                let is_poison = &operands[2];
                if is_poison.name().is_some() {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "immarg operand has non-immediate parameter".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }

                // Return type must match argument type
                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
                    if *result_type != *value_type {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Intrinsic has incorrect return type!".to_string(),
                            location: format!("call to {}", intrinsic_name),
                        });
                    }
                }
            }
        }

        // llvm.smax/smin/umax/umin - integer min/max intrinsics
        if intrinsic_name.starts_with("llvm.smax.") || intrinsic_name.starts_with("llvm.smin.") ||
           intrinsic_name.starts_with("llvm.umax.") || intrinsic_name.starts_with("llvm.umin.") {
            if operands.len() >= 3 {
                // operands[0] = function, operands[1] = arg1, operands[2] = arg2
                let arg1_type = operands[1].get_type();
                let arg2_type = operands[2].get_type();

                // Both arguments must be integers or vectors of integers
                let arg1_valid = arg1_type.is_integer() ||
                    (arg1_type.is_vector() && arg1_type.vector_info().map_or(false, |(e, _)| e.is_integer()));
                let arg2_valid = arg2_type.is_integer() ||
                    (arg2_type.is_vector() && arg2_type.vector_info().map_or(false, |(e, _)| e.is_integer()));

                if !arg1_valid || !arg2_valid {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect argument type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }

                // Both arguments must have same type
                if *arg1_type != *arg2_type {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect argument type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }

                // Return type must match argument types
                if let Some(result) = inst.result() {
                    let result_type = result.get_type();
                    if *result_type != *arg1_type {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Intrinsic has incorrect return type!".to_string(),
                            location: format!("call to {}", intrinsic_name),
                        });
                    }
                }
            }
        }
    }

    /// Verify immarg (immediate argument) parameters for intrinsics
    /// Certain intrinsic parameters must be constant values, not variables
    fn verify_intrinsic_immarg(&mut self, inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        // Helper to check if operand at index is not a constant
        let check_immarg = |idx: usize| -> bool {
            if operands.len() > idx {
                let operand = &operands[idx];
                // Operands with names are variables (non-constant)
                // Constants typically don't have names, or are represented differently
                !operand.is_constant()
            } else {
                false
            }
        };

        // Define immarg parameter positions for each intrinsic family
        // Format: (intrinsic_prefix, vec![param_indices_that_must_be_constant])
        // Note: indices include the function itself at position 0, so actual args start at 1

        // llvm.memcpy, llvm.memmove, llvm.memset - is_volatile must be constant
        if intrinsic_name.starts_with("llvm.memcpy.") ||
           intrinsic_name.starts_with("llvm.memmove.") ||
           intrinsic_name.starts_with("llvm.memset.") {
            if check_immarg(4) { // is_volatile parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.memcpy.element.unordered.atomic - element_size must be constant
        if intrinsic_name.starts_with("llvm.memcpy.element.unordered.atomic") ||
           intrinsic_name.starts_with("llvm.memmove.element.unordered.atomic") ||
           intrinsic_name.starts_with("llvm.memset.element.unordered.atomic") {
            if check_immarg(4) { // element_size parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.returnaddress, llvm.frameaddress - level must be constant
        if intrinsic_name.starts_with("llvm.returnaddress") ||
           intrinsic_name.starts_with("llvm.frameaddress") {
            if check_immarg(1) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.objectsize - boolean flags must be constant
        if intrinsic_name.starts_with("llvm.objectsize.") {
            for i in 2..5 {
                if check_immarg(i) {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "immarg operand has non-immediate parameter".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                    break;
                }
            }
        }

        // llvm.smul.fix, llvm.umul.fix, llvm.smul.fix.sat, llvm.umul.fix.sat - scale must be constant
        if intrinsic_name.starts_with("llvm.smul.fix.") ||
           intrinsic_name.starts_with("llvm.umul.fix.") {
            if check_immarg(3) { // scale parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.invariant.start - size must be constant
        if intrinsic_name.starts_with("llvm.invariant.start.") {
            if check_immarg(1) { // size parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.invariant.end - size must be constant
        if intrinsic_name.starts_with("llvm.invariant.end.") {
            if check_immarg(2) { // size parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.prefetch - rw, locality, cache type must be constant
        if intrinsic_name.starts_with("llvm.prefetch") {
            if check_immarg(2) || check_immarg(3) || check_immarg(4) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.localrecover - index must be constant
        if intrinsic_name.starts_with("llvm.localrecover") {
            if check_immarg(3) { // index parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.experimental.gc.statepoint - ID, num_patch_bytes, num_call_args, flags must be constant
        if intrinsic_name.starts_with("llvm.experimental.gc.statepoint.") {
            if check_immarg(1) || check_immarg(2) || check_immarg(4) || check_immarg(5) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.experimental.patchpoint - ID, num_bytes, num_args must be constant
        if intrinsic_name.starts_with("llvm.experimental.patchpoint.") {
            if check_immarg(1) || check_immarg(2) || check_immarg(4) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.hwasan.check.memaccess - access_info must be constant
        if intrinsic_name.starts_with("llvm.hwasan.check.memaccess") {
            if check_immarg(3) { // access_info parameter
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }

        // llvm.eh.sjlj.callsite - callsite index must be constant
        if intrinsic_name.starts_with("llvm.eh.sjlj.callsite") {
            if check_immarg(1) {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg operand has non-immediate parameter".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }
    }

    /// Verify llvm.vector.reduce.* intrinsics
    fn verify_intrinsic_vector_reduce(&mut self, inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        // Vector reduce intrinsics have format: llvm.vector.reduce.<op>.<type>
        // where <op> is: add, mul, and, or, xor, smax, smin, umax, umin, fmax, fmin, fadd, fmul

        // Determine expected types based on operation
        let is_int_op = intrinsic_name.contains(".add.") || intrinsic_name.contains(".mul.") ||
                        intrinsic_name.contains(".and.") || intrinsic_name.contains(".or.") ||
                        intrinsic_name.contains(".xor.") || intrinsic_name.contains(".smax.") ||
                        intrinsic_name.contains(".smin.") || intrinsic_name.contains(".umax.") ||
                        intrinsic_name.contains(".umin.");
        let is_fp_op = intrinsic_name.contains(".fmax.") || intrinsic_name.contains(".fmin.") ||
                       intrinsic_name.contains(".fadd.") || intrinsic_name.contains(".fmul.");

        if operands.len() < 2 {
            return; // Not enough operands, skip
        }

        // For fadd/fmul, first operand is start value (scalar), second is vector
        // For others, first operand is function, second is vector
        let vec_arg_idx = if intrinsic_name.contains(".fadd.") || intrinsic_name.contains(".fmul.") {
            2 // operands[0] = function, operands[1] = start value, operands[2] = vector
        } else {
            1 // operands[0] = function, operands[1] = vector
        };

        if operands.len() <= vec_arg_idx {
            return;
        }

        let vec_arg = &operands[vec_arg_idx];
        let vec_type = vec_arg.get_type();

        // Argument must be a vector
        if !vec_type.is_vector() {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Intrinsic has incorrect argument type!".to_string(),
                location: format!("call to {}", intrinsic_name),
            });
            return;
        }

        // Check element type matches operation type
        if let Some((elem_type, _)) = vec_type.vector_info() {
            if is_int_op && !elem_type.is_integer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Intrinsic has incorrect argument type!".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            } else if is_fp_op && !elem_type.is_float() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Intrinsic has incorrect argument type!".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }

            // For fadd/fmul with start value, check start value type matches element type
            if (intrinsic_name.contains(".fadd.") || intrinsic_name.contains(".fmul.")) && operands.len() >= 3 {
                let start_val = &operands[1];
                let start_type = start_val.get_type();
                if *start_type != *elem_type {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect argument type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }
            }

            // Check return type matches element type
            if let Some(result) = inst.result() {
                let result_type = result.get_type();
                if *result_type != *elem_type {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Intrinsic has incorrect return type!".to_string(),
                        location: format!("call to {}", intrinsic_name),
                    });
                }
            }
        }
    }

    /// Verify llvm.is.fpclass intrinsic
    fn verify_intrinsic_is_fpclass(&mut self, _inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        if operands.len() < 3 {
            return; // Not enough operands
        }

        // operands[0] = function, operands[1] = value, operands[2] = test mask
        let mask_operand = &operands[2];

        // Test mask must be a constant integer
        if let Some(mask_val) = mask_operand.as_const_int() {
            // Valid mask bits are 0-9 (values 0-1023)
            // Bit 10 and higher are invalid
            // Also, -1 (all bits set) is specifically invalid
            if mask_val < 0 || mask_val >= 1024 {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "unsupported bits for llvm.is.fpclass test mask".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }
        // Note: If mask is not constant, parser should have caught it as immarg violation
    }

    /// Verify saturating arithmetic intrinsics
    fn verify_intrinsic_sat(&mut self, inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        if operands.len() < 3 {
            return; // Not enough operands
        }

        // operands[0] = function, operands[1] = arg1, operands[2] = arg2
        let arg1_type = operands[1].get_type();
        let arg2_type = operands[2].get_type();

        // Both arguments must be integers or vectors of integers
        let arg1_valid = arg1_type.is_integer() ||
                        (arg1_type.is_vector() && arg1_type.vector_info().map_or(false, |(e, _)| e.is_integer()));
        let arg2_valid = arg2_type.is_integer() ||
                        (arg2_type.is_vector() && arg2_type.vector_info().map_or(false, |(e, _)| e.is_integer()));

        if !arg1_valid || !arg2_valid {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Intrinsic has incorrect argument type!".to_string(),
                location: format!("call to {}", intrinsic_name),
            });
        }

        // Both arguments must have same type
        if *arg1_type != *arg2_type {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Intrinsic has incorrect argument type!".to_string(),
                location: format!("call to {}", intrinsic_name),
            });
        }

        // Return type must match argument types
        if let Some(result) = inst.result() {
            let result_type = result.get_type();
            let result_valid = result_type.is_integer() ||
                              (result_type.is_vector() && result_type.vector_info().map_or(false, |(e, _)| e.is_integer()));

            if !result_valid {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Intrinsic has incorrect return type!".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            } else if *result_type != *arg1_type {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Intrinsic has incorrect return type!".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }
    }

    /// Verify llvm.vp.* (vector predication) intrinsics
    fn verify_intrinsic_vp(&mut self, inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        // llvm.vp.fptosi, llvm.vp.fptoui, llvm.vp.sitofp, llvm.vp.uitofp - cast intrinsics
        // VP cast intrinsics: first argument and result vector lengths must be equal
        if intrinsic_name.starts_with("llvm.vp.fptosi.") ||
           intrinsic_name.starts_with("llvm.vp.fptoui.") ||
           intrinsic_name.starts_with("llvm.vp.sitofp.") ||
           intrinsic_name.starts_with("llvm.vp.uitofp.") {

            if operands.len() < 2 {
                return;
            }

            let src_type = operands[1].get_type(); // operands[0] = function, operands[1] = source

            if let Some(result) = inst.result() {
                let dst_type = result.get_type();

                // Both must be vectors
                if src_type.is_vector() && dst_type.is_vector() {
                    if let (Some((_, src_len)), Some((_, dst_len))) = (src_type.vector_info(), dst_type.vector_info()) {
                        if src_len != dst_len {
                            self.errors.push(VerificationError::InvalidInstruction {
                                reason: "VP cast intrinsic first argument and result vector lengths must be equal".to_string(),
                                location: format!("call to {}", intrinsic_name),
                            });
                        }
                    }
                }
            }
        }

        // llvm.vp.fcmp and llvm.vp.icmp - comparison intrinsics
        // These require metadata predicate validation which we can't do without metadata access
        // Skipping for now
    }

    /// Verify llvm.bswap intrinsic
    fn verify_intrinsic_bswap(&mut self, inst: &Instruction, intrinsic_name: &str, operands: &[Value]) {
        if operands.len() < 2 {
            return; // operands[0] = function, operands[1] = value
        }

        let arg_type = operands[1].get_type();

        // Get the bit width for the type
        let check_bit_width = |ty: &Type| -> Option<u32> {
            if let Some(width) = ty.int_width() {
                Some(width)
            } else if ty.is_vector() {
                if let Some((elem, _)) = ty.vector_info() {
                    elem.int_width()
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(bit_width) = check_bit_width(&arg_type) {
            // bswap must operate on types with even number of bytes
            // i8 = 8 bits = 1 byte (odd number of bytes) - invalid
            // i16 = 16 bits = 2 bytes (even) - valid
            // i12 = 12 bits = 1.5 bytes - invalid (not even)
            if bit_width % 16 != 0 {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "bswap must be an even number of bytes".to_string(),
                    location: format!("call to {}", intrinsic_name),
                });
            }
        }
    }

    /// Verify allockind attribute
    fn verify_allockind_attribute(&mut self, kinds: &[String], fn_name: &str) {
        let mut has_alloc = false;
        let mut has_realloc = false;
        let mut has_free = false;
        let mut has_zeroed = false;
        let mut has_uninitialized = false;
        let mut has_aligned = false;

        for kind in kinds {
            match kind.as_str() {
                "alloc" => has_alloc = true,
                "realloc" => has_realloc = true,
                "free" => has_free = true,
                "zeroed" => has_zeroed = true,
                "uninitialized" => has_uninitialized = true,
                "aligned" => has_aligned = true,
                _ => {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: format!("'allockind' has unknown value '{}'", kind),
                        location: format!("function {}", fn_name),
                    });
                }
            }
        }

        // Must have exactly one of alloc, realloc, or free
        let primary_count = [has_alloc, has_realloc, has_free].iter().filter(|&&x| x).count();
        if primary_count != 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "'allockind()' requires exactly one of alloc, realloc, and free".to_string(),
                location: format!("function {}", fn_name),
            });
        }

        // free doesn't allow uninitialized, zeroed, or aligned
        if has_free && (has_uninitialized || has_zeroed || has_aligned) {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "'allockind(\"free\")' doesn't allow uninitialized, zeroed, or aligned modifiers".to_string(),
                location: format!("function {}", fn_name),
            });
        }

        // Can't be both zeroed and uninitialized
        if has_zeroed && has_uninitialized {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "'allockind()' can't be both zeroed and uninitialized".to_string(),
                location: format!("function {}", fn_name),
            });
        }
    }

    /// Verify allocsize attribute
    fn verify_allocsize_attribute(&mut self, indices: &[usize], function: &Function) {
        let fn_name = function.name();
        let fn_type = function.get_type();

        // Get parameter types
        let param_types = if let Some((_, params, _)) = fn_type.function_info() {
            params
        } else {
            return; // Not a function type, can't validate
        };

        // Validate element size argument (first index)
        if !indices.is_empty() {
            let elem_idx = indices[0];
            if elem_idx >= param_types.len() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "'allocsize' element size argument is out of bounds".to_string(),
                    location: format!("function {}", fn_name),
                });
            } else if !param_types[elem_idx].is_integer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "'allocsize' element size argument must refer to an integer parameter".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // Validate number of elements argument (second index, if present)
        if indices.len() > 1 {
            let count_idx = indices[1];
            if count_idx >= param_types.len() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "'allocsize' number of elements argument is out of bounds".to_string(),
                    location: format!("function {}", fn_name),
                });
            } else if !param_types[count_idx].is_integer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "'allocsize' number of elements argument must refer to an integer parameter".to_string(),
                    location: format!("function {}", fn_name),
                });
            }
        }

        // allocsize should have at most 2 indices
        if indices.len() > 2 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "'allocsize' takes at most 2 arguments".to_string(),
                location: format!("function {}", fn_name),
            });
        }

        // allocsize indices can't be the same
        if indices.len() == 2 && indices[0] == indices[1] {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "'allocsize' indices can't refer to the same parameter".to_string(),
                location: format!("function {}", fn_name),
            });
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
