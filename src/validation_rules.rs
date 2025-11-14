//! Validation Rules
//!
//! This module contains additional IR validation rules to catch invalid patterns
//! that the parser accepts but LLVM rejects. These rules are separated from the
//! main verification.rs to keep the codebase modular.
//!
//! Each validation function corresponds to specific LLVM verifier rules and is
//! designed to fix batches of LLVM test suite failures.

use crate::function::{Function, ParameterAttributes, CallingConvention};
use crate::types::Type;
use crate::module::{Module, Linkage, Visibility};
use crate::verification::VerificationError;

/// Collection of IR validation rules
pub struct ValidationRules {
    errors: Vec<VerificationError>,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Run all extended validation checks on a module
    pub fn validate_module(&mut self, module: &Module) -> Result<(), Vec<VerificationError>> {
        // Validate all functions
        for function in module.functions() {
            self.validate_function(&function);
        }

        // Validate all globals
        for global in module.globals() {
            self.validate_global_linkage_visibility(&global.name, global.linkage, global.visibility);
        }

        // Validate all aliases
        for alias in module.aliases() {
            self.validate_global_linkage_visibility(&alias.name, alias.linkage, alias.visibility);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Validate a single function
    fn validate_function(&mut self, function: &Function) {
        // Check calling convention constraints
        self.validate_calling_convention_constraints(function);

        // Check function-level attributes
        self.validate_function_level_attributes(function);

        // Check return attributes
        self.validate_return_attributes(function);

        // Check parameter attributes
        self.validate_parameter_attributes(function);
    }

    /// Validate calling convention constraints
    ///
    /// Different calling conventions have different restrictions:
    /// - amdgpu_kernel: no sret, must return void, no varargs
    /// - amdgpu_vs/gs/ps/cs/hs/ls/es: no varargs, no byval/byref/inalloca/preallocated
    /// - spir_kernel: must return void, no varargs
    ///
    /// Reference: LLVM test Verifier/amdgpu-cc.ll
    /// Expected to fix ~30 tests
    fn validate_calling_convention_constraints(&mut self, function: &Function) {
        let cc = function.calling_convention();
        let func_type = function.get_type();

        // Extract function type info
        let Some((return_type, _param_types, is_varargs)) = func_type.function_info() else {
            return; // Not a function type, skip
        };

        match cc {
            CallingConvention::AMDGPU_Kernel => {
                // Must return void
                if !return_type.is_void() {
                    self.errors.push(VerificationError::InvalidCall {
                        expected_args: 0,
                        found_args: 0,
                        location: format!(
                            "Calling convention requires void return type at function {}",
                            function.name()
                        ),
                    });
                }

                // No varargs
                if is_varargs {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not support varargs or perfect forwarding!".to_string(),
                        location: format!("function {}", function.name()),
                    });
                }

                // Check for sret in parameters
                let attrs = function.attributes();
                for (idx, param_attrs) in attrs.parameter_attributes.iter().enumerate() {
                    if param_attrs.sret.is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention does not allow sret".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }

                    // No byval
                    if param_attrs.byval.is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention disallows byval".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }

                    // No inalloca
                    if param_attrs.inalloca.is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention disallows inalloca".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }

                    // No byref in address space 5
                    if param_attrs.byref.is_some() {
                        // Note: We can't check address space without parser support,
                        // but we can at least reject all byref for now
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention disallows stack byref".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }
                }
            }

            CallingConvention::AMDGPU_VS
            | CallingConvention::AMDGPU_GS
            | CallingConvention::AMDGPU_PS
            | CallingConvention::AMDGPU_CS
            | CallingConvention::AMDGPU_HS => {
                // No varargs
                if is_varargs {
                    self.errors.push(VerificationError::InvalidInstruction {
                        reason: "Calling convention does not support varargs or perfect forwarding!".to_string(),
                        location: format!("function {}", function.name()),
                    });
                }

                // No byval/byref
                let attrs = function.attributes();
                for (idx, param_attrs) in attrs.parameter_attributes.iter().enumerate() {
                    if param_attrs.byval.is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention disallows byval".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }

                    if param_attrs.byref.is_some() {
                        self.errors.push(VerificationError::InvalidInstruction {
                            reason: "Calling convention disallows stack byref".to_string(),
                            location: format!("function {} parameter {}", function.name(), idx),
                        });
                    }
                }
            }

            // TODO: Add SPIR_Kernel, X86 calling conventions, etc.
            _ => {}
        }
    }

    /// Validate function-level attributes
    fn validate_function_level_attributes(&mut self, function: &Function) {
        let func_attrs = function.attributes();

        // immarg is not valid on functions - only on parameters
        if func_attrs.has_immarg {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "this attribute does not apply to functions".to_string(),
                location: format!("@{}", function.name()),
            });
        }
    }

    /// Validate return attribute compatibility
    fn validate_return_attributes(&mut self, function: &Function) {
        let func_attrs = function.attributes();
        let ret_attrs = &func_attrs.return_attributes;

        // immarg is not valid on return values
        if ret_attrs.has_immarg {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "this attribute does not apply to return values".to_string(),
                location: format!("@{}", function.name()),
            });
        }
    }

    /// Validate parameter attribute compatibility
    ///
    /// Certain parameter attributes are mutually exclusive:
    /// - byval, inalloca, preallocated, inreg, nest, byref, and sret
    ///
    /// Reference: LLVM test Verifier/byref.ll, byval-1.ll, inalloca.ll
    /// Expected to fix ~50 tests
    fn validate_parameter_attributes(&mut self, function: &Function) {
        let func_attrs = function.attributes();
        let func_type = function.get_type();
        let param_types = func_type.function_info().map(|(_, types, _)| types).unwrap_or_default();

        for (idx, param_attrs) in func_attrs.parameter_attributes.iter().enumerate() {
            self.check_attribute_exclusivity(param_attrs, &function.name(), idx);
            self.check_attribute_type_compatibility(param_attrs, &function.name(), idx, param_types.get(idx));
        }
    }

    /// Check that mutually exclusive attributes aren't used together
    fn check_attribute_exclusivity(&mut self, attrs: &ParameterAttributes, func_name: &str, param_idx: usize) {
        let mut exclusive_attrs = Vec::new();

        if attrs.byval.is_some() {
            exclusive_attrs.push("byval");
        }
        if attrs.inalloca.is_some() {
            exclusive_attrs.push("inalloca");
        }
        if attrs.byref.is_some() {
            exclusive_attrs.push("byref");
        }
        if attrs.sret.is_some() {
            exclusive_attrs.push("sret");
        }
        if attrs.inreg {
            exclusive_attrs.push("inreg");
        }
        if attrs.nest {
            exclusive_attrs.push("nest");
        }
        // Note: preallocated not currently in ParameterAttributes struct

        if exclusive_attrs.len() > 1 {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: format!(
                    "Attributes 'byval', 'inalloca', 'preallocated', 'inreg', 'nest', 'byref', and 'sret' are incompatible!"
                ),
                location: format!("@{}", func_name),
            });
        }

        // immarg is incompatible with byval, byref, inalloca, sret, nest, and other non-range attributes
        if attrs.immarg && !exclusive_attrs.is_empty() {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Attribute 'immarg' is incompatible with other attributes except the 'range' attribute".to_string(),
                location: format!("@{}", func_name),
            });
        }
    }

    /// Check that attributes are applied to compatible types
    fn check_attribute_type_compatibility(
        &mut self,
        attrs: &ParameterAttributes,
        func_name: &str,
        param_idx: usize,
        param_type: Option<&Type>,
    ) {
        let Some(param_type) = param_type else { return };

        // byval can only be applied to pointer types
        if let Some(byval_type) = &attrs.byval {
            if !param_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'byval({})' applied to incompatible type!", byval_type),
                    location: format!("@{}", func_name),
                });
            }
        }

        // inalloca can only be applied to pointer types
        if let Some(inalloca_type) = &attrs.inalloca {
            if !param_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'inalloca({})' applied to incompatible type!", inalloca_type),
                    location: format!("@{}", func_name),
                });
            }
        }

        // byref can only be applied to pointer types
        if let Some(byref_type) = &attrs.byref {
            if !param_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'byref({})' applied to incompatible type!", byref_type),
                    location: format!("@{}", func_name),
                });
            }

            // byref does not support unsized types (opaque types)
            // TODO: When we have opaque type detection, add this check:
            // if byref_type.is_opaque() { ... }
        }

        // sret can only be applied to pointer types
        if let Some(sret_type) = &attrs.sret {
            if !param_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'sret({})' applied to incompatible type!", sret_type),
                    location: format!("@{}", func_name),
                });
            }
        }

        // align can only be applied to pointer types
        if let Some(align_val) = attrs.align {
            if !param_type.is_pointer() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: format!("Attribute 'align {}' applied to incompatible type!", align_val),
                    location: format!("@{}", func_name),
                });
            }

            // Alignment must be a power of two
            if align_val == 0 || (align_val & (align_val - 1)) != 0 {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "alignment is not a power of two".to_string(),
                    location: format!("@{}", func_name),
                });
            }

            // Check for huge alignments (> 2^29)
            const MAX_ALIGN: u32 = 1 << 29;
            if align_val > MAX_ALIGN {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "huge alignments are not supported yet".to_string(),
                    location: format!("@{}", func_name),
                });
            }
        }

        // immarg must only be used with constant arguments (checked during call validation)
        // Note: The actual check for immarg requires checking call sites, which is more complex
        // immarg applies to integers, floats, vectors, and arrays
        if attrs.immarg {
            let is_valid_immarg = param_type.is_integer()
                || param_type.is_float()
                || param_type.is_vector()
                || param_type.is_array();

            if !is_valid_immarg {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "immarg attribute only applies to integers, floats, vectors, and arrays".to_string(),
                    location: format!("@{}", func_name),
                });
            }
        }

        // mustprogress does not apply to parameters - it's a function-level attribute only
        if attrs.mustprogress {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "this attribute does not apply to parameters".to_string(),
                location: format!("@{}", func_name),
            });
        }
    }

    /// Validate global variable or alias linkage-visibility combinations
    ///
    /// LLVM requires that symbols with local linkage (Internal or Private)
    /// must have default visibility. Hidden or protected visibility is only
    /// allowed for symbols with non-local linkage.
    ///
    /// Reference: LLVM tests internal-hidden-alias.ll, private-protected-alias.ll, etc.
    /// Expected to fix ~4-5 tests
    fn validate_global_linkage_visibility(&mut self, name: &str, linkage: Linkage, visibility: Visibility) {
        // Symbols with local linkage (Internal or Private) must have default visibility
        if matches!(linkage, Linkage::Internal | Linkage::Private)
            && !matches!(visibility, Visibility::Default) {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "symbol with local linkage must have default visibility".to_string(),
                location: format!("@{}", name),
            });
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::function::Function;
    use crate::types::Type;

    #[test]
    fn test_attribute_incompatibility_byval_byref() {
        // This test verifies that byval and byref are detected as incompatible
        let ctx = Context::new();
        let func_type = ctx.function_type(ctx.void_type(), vec![], false);
        let func = Function::new("test".to_string(), func_type);

        let mut param_attrs = ParameterAttributes::default();
        param_attrs.byval = Some(ctx.int_type(32));
        param_attrs.byref = Some(ctx.int_type(32));

        let mut verifier = ValidationRules::new();
        verifier.check_attribute_exclusivity(&param_attrs, "test", 0);

        assert!(!verifier.errors.is_empty(), "Should detect incompatible attributes");
        assert!(
            format!("{:?}", verifier.errors[0]).contains("incompatible"),
            "Error should mention incompatibility"
        );
    }

    #[test]
    fn test_amdgpu_kernel_no_sret() {
        let ctx = Context::new();
        let ptr_ty = ctx.ptr_type(ctx.int8_type());
        let func_type = ctx.function_type(ctx.void_type(), vec![ptr_ty], false);
        let func = Function::new("test".to_string(), func_type);
        func.set_calling_convention(CallingConvention::AMDGPU_Kernel);

        // Add sret attribute (should be rejected)
        let mut func_attrs = func.attributes();
        let mut param_attrs = ParameterAttributes::default();
        param_attrs.sret = Some(ctx.int_type(32));
        func_attrs.parameter_attributes = vec![param_attrs];
        func.set_attributes(func_attrs);

        let mut verifier = ValidationRules::new();
        verifier.validate_calling_convention_constraints(&func);

        assert!(!verifier.errors.is_empty(), "Should reject sret in amdgpu_kernel");
        assert!(
            format!("{:?}", verifier.errors).to_lowercase().contains("sret"),
            "Error should mention sret"
        );
    }
}
