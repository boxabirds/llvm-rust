//! External Function Support (Level 9)
//!
//! This module provides support for calling external functions from standard libraries.

use std::collections::HashMap;

/// External function registry
pub struct ExternalFunctionRegistry {
    /// Map of function names to their signatures
    functions: HashMap<String, ExternalFunction>,
}

/// External function definition
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub name: String,
    pub return_type: ExternalType,
    pub param_types: Vec<ExternalType>,
    pub is_variadic: bool,
}

/// External type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalType {
    Void,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

impl ExternalFunctionRegistry {
    /// Create a new external function registry with standard library functions
    pub fn new_with_stdlib() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Add common libc functions
        registry.add_printf();
        registry.add_malloc();
        registry.add_free();
        registry.add_strlen();
        registry.add_strcpy();
        registry.add_strcmp();
        registry.add_memcpy();
        registry.add_memset();
        registry.add_exit();
        registry.add_puts();
        registry.add_fopen();
        registry.add_fclose();
        registry.add_fread();
        registry.add_fwrite();

        registry
    }

    /// Register an external function
    pub fn register(&mut self, func: ExternalFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Get an external function by name
    pub fn get(&self, name: &str) -> Option<&ExternalFunction> {
        self.functions.get(name)
    }

    /// Check if a function is external
    pub fn is_external(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    // Standard library function definitions

    fn add_printf(&mut self) {
        self.register(ExternalFunction {
            name: "printf".to_string(),
            return_type: ExternalType::I32,
            param_types: vec![ExternalType::Ptr], // format string
            is_variadic: true,
        });
    }

    fn add_malloc(&mut self) {
        self.register(ExternalFunction {
            name: "malloc".to_string(),
            return_type: ExternalType::Ptr,
            param_types: vec![ExternalType::I64], // size
            is_variadic: false,
        });
    }

    fn add_free(&mut self) {
        self.register(ExternalFunction {
            name: "free".to_string(),
            return_type: ExternalType::Void,
            param_types: vec![ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_strlen(&mut self) {
        self.register(ExternalFunction {
            name: "strlen".to_string(),
            return_type: ExternalType::I64,
            param_types: vec![ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_strcpy(&mut self) {
        self.register(ExternalFunction {
            name: "strcpy".to_string(),
            return_type: ExternalType::Ptr,
            param_types: vec![ExternalType::Ptr, ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_strcmp(&mut self) {
        self.register(ExternalFunction {
            name: "strcmp".to_string(),
            return_type: ExternalType::I32,
            param_types: vec![ExternalType::Ptr, ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_memcpy(&mut self) {
        self.register(ExternalFunction {
            name: "memcpy".to_string(),
            return_type: ExternalType::Ptr,
            param_types: vec![ExternalType::Ptr, ExternalType::Ptr, ExternalType::I64],
            is_variadic: false,
        });
    }

    fn add_memset(&mut self) {
        self.register(ExternalFunction {
            name: "memset".to_string(),
            return_type: ExternalType::Ptr,
            param_types: vec![ExternalType::Ptr, ExternalType::I32, ExternalType::I64],
            is_variadic: false,
        });
    }

    fn add_exit(&mut self) {
        self.register(ExternalFunction {
            name: "exit".to_string(),
            return_type: ExternalType::Void,
            param_types: vec![ExternalType::I32],
            is_variadic: false,
        });
    }

    fn add_puts(&mut self) {
        self.register(ExternalFunction {
            name: "puts".to_string(),
            return_type: ExternalType::I32,
            param_types: vec![ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_fopen(&mut self) {
        self.register(ExternalFunction {
            name: "fopen".to_string(),
            return_type: ExternalType::Ptr,
            param_types: vec![ExternalType::Ptr, ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_fclose(&mut self) {
        self.register(ExternalFunction {
            name: "fclose".to_string(),
            return_type: ExternalType::I32,
            param_types: vec![ExternalType::Ptr],
            is_variadic: false,
        });
    }

    fn add_fread(&mut self) {
        self.register(ExternalFunction {
            name: "fread".to_string(),
            return_type: ExternalType::I64,
            param_types: vec![
                ExternalType::Ptr,
                ExternalType::I64,
                ExternalType::I64,
                ExternalType::Ptr,
            ],
            is_variadic: false,
        });
    }

    fn add_fwrite(&mut self) {
        self.register(ExternalFunction {
            name: "fwrite".to_string(),
            return_type: ExternalType::I64,
            param_types: vec![
                ExternalType::Ptr,
                ExternalType::I64,
                ExternalType::I64,
                ExternalType::Ptr,
            ],
            is_variadic: false,
        });
    }
}

/// FFI helper for generating external function calls
pub struct FFIHelper {
    registry: ExternalFunctionRegistry,
}

impl FFIHelper {
    /// Create a new FFI helper
    pub fn new() -> Self {
        Self {
            registry: ExternalFunctionRegistry::new_with_stdlib(),
        }
    }

    /// Generate calling code for an external function
    pub fn generate_call(&self, func_name: &str, args: &[String]) -> Result<String, String> {
        let func = self.registry.get(func_name)
            .ok_or_else(|| format!("Unknown external function: {}", func_name))?;

        if !func.is_variadic && args.len() != func.param_types.len() {
            return Err(format!(
                "Argument count mismatch for {}: expected {}, got {}",
                func_name,
                func.param_types.len(),
                args.len()
            ));
        }

        // Generate assembly for external call
        let mut asm = String::new();

        // In System V ABI, first 6 integer arguments go in registers
        let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

        for (i, arg) in args.iter().enumerate() {
            if i < 6 {
                asm.push_str(&format!("\tmov {}, {}\n", arg_regs[i], arg));
            } else {
                // Remaining arguments go on stack
                asm.push_str(&format!("\tpush {}\n", arg));
            }
        }

        asm.push_str(&format!("\tcall {}@PLT\n", func_name));

        Ok(asm)
    }

    /// Get the registry
    pub fn registry(&self) -> &ExternalFunctionRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_registration() {
        let registry = ExternalFunctionRegistry::new_with_stdlib();
        assert!(registry.is_external("printf"));
        assert!(registry.is_external("malloc"));
        assert!(registry.is_external("free"));
    }

    #[test]
    fn test_function_lookup() {
        let registry = ExternalFunctionRegistry::new_with_stdlib();
        let printf = registry.get("printf").unwrap();
        assert_eq!(printf.name, "printf");
        assert!(printf.is_variadic);
    }

    #[test]
    fn test_ffi_helper() {
        let helper = FFIHelper::new();
        let result = helper.generate_call("puts", &["rax".to_string()]);
        assert!(result.is_ok());
    }
}
