//! The Context is the top-level container for all LLVM entities.
//! It owns and manages all types and constants.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A context is a container for all LLVM IR entities.
/// In LLVM, a context owns all types and constants, ensuring type uniqueness.
#[derive(Default, Clone)]
pub struct Context {
    // Type interning - ensures type uniqueness
    type_cache: Arc<Mutex<HashMap<String, Arc<crate::types::TypeData>>>>,
}

impl Context {
    /// Create a new LLVM context
    pub fn new() -> Self {
        Self {
            type_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a type in this context
    pub(crate) fn intern_type(&self, key: String, type_data: crate::types::TypeData) -> Arc<crate::types::TypeData> {
        let mut cache = self.type_cache.lock().unwrap();
        cache.entry(key).or_insert_with(|| Arc::new(type_data)).clone()
    }

    // Type construction helpers

    /// Get the void type
    pub fn void_type(&self) -> crate::types::Type {
        crate::types::Type::void(self)
    }

    /// Get an integer type with the specified bit width
    pub fn int_type(&self, bits: u32) -> crate::types::Type {
        crate::types::Type::int(self, bits)
    }

    /// Get a 1-bit integer type (boolean)
    pub fn bool_type(&self) -> crate::types::Type {
        self.int_type(1)
    }

    /// Get an 8-bit integer type
    pub fn int8_type(&self) -> crate::types::Type {
        self.int_type(8)
    }

    /// Get a 16-bit integer type
    pub fn int16_type(&self) -> crate::types::Type {
        self.int_type(16)
    }

    /// Get a 32-bit integer type
    pub fn int32_type(&self) -> crate::types::Type {
        self.int_type(32)
    }

    /// Get a 64-bit integer type
    pub fn int64_type(&self) -> crate::types::Type {
        self.int_type(64)
    }

    /// Get a half-precision floating point type (16-bit)
    pub fn half_type(&self) -> crate::types::Type {
        crate::types::Type::half(self)
    }

    /// Get a single-precision floating point type (32-bit)
    pub fn float_type(&self) -> crate::types::Type {
        crate::types::Type::float(self)
    }

    /// Get a double-precision floating point type (64-bit)
    pub fn double_type(&self) -> crate::types::Type {
        crate::types::Type::double(self)
    }

    /// Get a pointer type
    pub fn ptr_type(&self, pointee: crate::types::Type) -> crate::types::Type {
        crate::types::Type::ptr(self, pointee)
    }

    /// Get an array type
    pub fn array_type(&self, element: crate::types::Type, size: usize) -> crate::types::Type {
        crate::types::Type::array(self, element, size)
    }

    /// Get a function type
    pub fn function_type(&self, return_type: crate::types::Type, param_types: Vec<crate::types::Type>, is_var_arg: bool) -> crate::types::Type {
        crate::types::Type::function(self, return_type, param_types, is_var_arg)
    }
}
