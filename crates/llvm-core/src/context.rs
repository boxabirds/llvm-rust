//! LLVM Context
//!
//! The Context is a container for all LLVM global data structures and types.
//! It owns and manages the lifetime of types and constants.

use bumpalo::Bump;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use crate::types::{FunctionType, IntegerType, StructType};

/// A context for LLVM IR.
///
/// All LLVM IR constructs are associated with a context. The context owns
/// the memory for all types and uniqued constants.
#[derive(Debug)]
pub struct Context {
    /// Arena allocator for types
    type_arena: Arc<Bump>,

    /// Cache of integer types by bit width
    int_types: RwLock<FxHashMap<u32, Arc<IntegerType>>>,

    /// Cache of struct types by name
    named_structs: RwLock<FxHashMap<String, Arc<StructType>>>,

    /// Cache of function types
    function_types: RwLock<Vec<Arc<FunctionType>>>,
}

impl Context {
    /// Creates a new LLVM context.
    pub fn new() -> Self {
        Context {
            type_arena: Arc::new(Bump::new()),
            int_types: RwLock::new(FxHashMap::default()),
            named_structs: RwLock::new(FxHashMap::default()),
            function_types: RwLock::new(Vec::new()),
        }
    }

    /// Gets or creates an integer type with the specified bit width.
    pub fn get_int_type(&self, bit_width: u32) -> Arc<IntegerType> {
        let mut cache = self.int_types.write();
        cache
            .entry(bit_width)
            .or_insert_with(|| Arc::new(IntegerType::new(bit_width)))
            .clone()
    }

    /// Gets the i1 (boolean) type.
    pub fn i1_type(&self) -> Arc<IntegerType> {
        self.get_int_type(1)
    }

    /// Gets the i8 type.
    pub fn i8_type(&self) -> Arc<IntegerType> {
        self.get_int_type(8)
    }

    /// Gets the i16 type.
    pub fn i16_type(&self) -> Arc<IntegerType> {
        self.get_int_type(16)
    }

    /// Gets the i32 type.
    pub fn i32_type(&self) -> Arc<IntegerType> {
        self.get_int_type(32)
    }

    /// Gets the i64 type.
    pub fn i64_type(&self) -> Arc<IntegerType> {
        self.get_int_type(64)
    }

    /// Gets the i128 type.
    pub fn i128_type(&self) -> Arc<IntegerType> {
        self.get_int_type(128)
    }

    /// Creates or gets a named struct type.
    pub fn get_or_create_struct(&self, name: String) -> Arc<StructType> {
        let mut cache = self.named_structs.write();
        cache
            .entry(name.clone())
            .or_insert_with(|| Arc::new(StructType::new_named(name)))
            .clone()
    }

    /// Gets a struct type by name, if it exists.
    pub fn get_struct_type(&self, name: &str) -> Option<Arc<StructType>> {
        self.named_structs.read().get(name).cloned()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = Context::new();
        let _ = ctx.i32_type();
    }

    #[test]
    fn test_int_type_caching() {
        let ctx = Context::new();
        let i32_1 = ctx.i32_type();
        let i32_2 = ctx.i32_type();
        assert!(Arc::ptr_eq(&i32_1, &i32_2));
    }

    #[test]
    fn test_various_int_types() {
        let ctx = Context::new();
        let i1 = ctx.i1_type();
        let i8 = ctx.i8_type();
        let i16 = ctx.i16_type();
        let i32 = ctx.i32_type();
        let i64 = ctx.i64_type();
        let i128 = ctx.i128_type();

        assert_eq!(i1.bit_width(), 1);
        assert_eq!(i8.bit_width(), 8);
        assert_eq!(i16.bit_width(), 16);
        assert_eq!(i32.bit_width(), 32);
        assert_eq!(i64.bit_width(), 64);
        assert_eq!(i128.bit_width(), 128);
    }

    #[test]
    fn test_named_struct() {
        let ctx = Context::new();
        let s1 = ctx.get_or_create_struct("MyStruct".to_string());
        let s2 = ctx.get_or_create_struct("MyStruct".to_string());
        assert!(Arc::ptr_eq(&s1, &s2));
    }
}
