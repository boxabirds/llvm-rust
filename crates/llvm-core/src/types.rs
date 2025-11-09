//! LLVM Type System
//!
//! This module defines all LLVM types including:
//! - Integer types (i1, i8, i16, i32, i64, i128, etc.)
//! - Floating-point types (half, float, double, fp128)
//! - Pointer types
//! - Array types
//! - Vector types
//! - Struct types
//! - Function types

use parking_lot::RwLock;
use smallvec::SmallVec;
use std::sync::Arc;

/// The main Type enum representing all LLVM types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Void type (for functions that return nothing)
    Void,
    /// Integer type with specified bit width
    Integer(Arc<IntegerType>),
    /// Floating-point type
    Float(FloatType),
    /// Pointer type
    Pointer(Box<PointerType>),
    /// Array type
    Array(Box<ArrayType>),
    /// Vector type
    Vector(Box<VectorType>),
    /// Struct type
    Struct(Arc<StructType>),
    /// Function type
    Function(Arc<FunctionType>),
}

impl Type {
    /// Creates an integer type with the specified bit width.
    pub fn int(bit_width: u32) -> Self {
        Type::Integer(Arc::new(IntegerType::new(bit_width)))
    }

    /// Creates an i1 (boolean) type.
    pub fn i1() -> Self {
        Type::int(1)
    }

    /// Creates an i8 type.
    pub fn i8() -> Self {
        Type::int(8)
    }

    /// Creates an i16 type.
    pub fn i16() -> Self {
        Type::int(16)
    }

    /// Creates an i32 type.
    pub fn i32() -> Self {
        Type::int(32)
    }

    /// Creates an i64 type.
    pub fn i64() -> Self {
        Type::int(64)
    }

    /// Creates an i128 type.
    pub fn i128() -> Self {
        Type::int(128)
    }

    /// Creates a void type.
    pub fn void() -> Self {
        Type::Void
    }

    /// Creates a float type.
    pub fn float() -> Self {
        Type::Float(FloatType::Float)
    }

    /// Creates a double type.
    pub fn double() -> Self {
        Type::Float(FloatType::Double)
    }

    /// Creates a pointer type.
    pub fn pointer(pointee: Type, address_space: u32) -> Self {
        Type::Pointer(Box::new(PointerType {
            pointee,
            address_space,
        }))
    }

    /// Creates an array type.
    pub fn array(element: Type, length: u64) -> Self {
        Type::Array(Box::new(ArrayType { element, length }))
    }

    /// Creates a vector type.
    pub fn vector(element: Type, count: u64) -> Self {
        Type::Vector(Box::new(VectorType { element, count }))
    }

    /// Returns true if this is a void type.
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    /// Returns true if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Integer(_))
    }

    /// Returns true if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float(_))
    }

    /// Returns true if this is a pointer type.
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    /// Returns the size of this type in bits, if known.
    pub fn size_in_bits(&self) -> Option<u64> {
        match self {
            Type::Void => None,
            Type::Integer(int_ty) => Some(int_ty.bit_width() as u64),
            Type::Float(float_ty) => Some(float_ty.size_in_bits()),
            Type::Pointer(_) => Some(64), // Assume 64-bit pointers
            Type::Array(arr) => arr
                .element
                .size_in_bits()
                .map(|elem_size| elem_size * arr.length),
            Type::Vector(vec) => vec
                .element
                .size_in_bits()
                .map(|elem_size| elem_size * vec.count),
            Type::Struct(struct_ty) => struct_ty.size_in_bits(),
            Type::Function(_) => None,
        }
    }
}

/// Integer type with arbitrary bit width.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerType {
    bit_width: u32,
}

impl IntegerType {
    /// Creates a new integer type with the specified bit width.
    pub fn new(bit_width: u32) -> Self {
        assert!(bit_width > 0, "Integer type must have positive bit width");
        IntegerType { bit_width }
    }

    /// Returns the bit width of this integer type.
    pub fn bit_width(&self) -> u32 {
        self.bit_width
    }
}

/// Floating-point types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatType {
    /// 16-bit float (IEEE 754 half precision)
    Half,
    /// 32-bit float (IEEE 754 single precision)
    Float,
    /// 64-bit float (IEEE 754 double precision)
    Double,
    /// 128-bit float (IEEE 754 quad precision)
    FP128,
    /// x86 80-bit extended precision float
    X86_FP80,
    /// PowerPC 128-bit float (two 64-bit doubles)
    PPC_FP128,
}

impl FloatType {
    /// Returns the size of this float type in bits.
    pub fn size_in_bits(&self) -> u64 {
        match self {
            FloatType::Half => 16,
            FloatType::Float => 32,
            FloatType::Double => 64,
            FloatType::FP128 => 128,
            FloatType::X86_FP80 => 80,
            FloatType::PPC_FP128 => 128,
        }
    }
}

/// Pointer type.
#[derive(Debug, Clone, PartialEq)]
pub struct PointerType {
    /// The type being pointed to
    pub pointee: Type,
    /// Address space (0 for default)
    pub address_space: u32,
}

/// Array type.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayType {
    /// Element type
    pub element: Type,
    /// Number of elements
    pub length: u64,
}

/// Vector type (SIMD).
#[derive(Debug, Clone, PartialEq)]
pub struct VectorType {
    /// Element type
    pub element: Type,
    /// Number of elements
    pub count: u64,
}

/// Struct type.
#[derive(Debug)]
pub struct StructType {
    /// Optional name for named structs
    name: Option<String>,
    /// Field types
    fields: RwLock<Option<SmallVec<[Type; 8]>>>,
    /// Whether this is a packed struct
    packed: RwLock<bool>,
}

impl StructType {
    /// Creates a new unnamed struct type.
    pub fn new(fields: Vec<Type>, packed: bool) -> Self {
        StructType {
            name: None,
            fields: RwLock::new(Some(SmallVec::from_vec(fields))),
            packed: RwLock::new(packed),
        }
    }

    /// Creates a new named struct type (initially opaque).
    pub fn new_named(name: String) -> Self {
        StructType {
            name: Some(name),
            fields: RwLock::new(None),
            packed: RwLock::new(false),
        }
    }

    /// Sets the body of this struct type.
    pub fn set_body(&self, fields: Vec<Type>, packed: bool) {
        *self.fields.write() = Some(SmallVec::from_vec(fields));
        *self.packed.write() = packed;
    }

    /// Returns the name of this struct, if it has one.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns true if this is an opaque struct (no body defined).
    pub fn is_opaque(&self) -> bool {
        self.fields.read().is_none()
    }

    /// Returns the number of fields.
    pub fn num_fields(&self) -> usize {
        self.fields
            .read()
            .as_ref()
            .map(|f| f.len())
            .unwrap_or(0)
    }

    /// Returns the field types.
    pub fn fields(&self) -> Option<Vec<Type>> {
        self.fields.read().as_ref().map(|f| f.to_vec())
    }

    /// Returns true if this is a packed struct.
    pub fn is_packed(&self) -> bool {
        *self.packed.read()
    }

    /// Returns the size in bits, if the struct is not opaque.
    pub fn size_in_bits(&self) -> Option<u64> {
        let fields = self.fields.read();
        fields.as_ref().and_then(|fields| {
            let mut total = 0u64;
            for field in fields.iter() {
                total += field.size_in_bits()?;
            }
            Some(total)
        })
    }
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        // Compare names
        if self.name != other.name {
            return false;
        }
        // Compare fields
        let self_fields = self.fields.read();
        let other_fields = other.fields.read();
        if *self_fields != *other_fields {
            return false;
        }
        // Compare packed flag
        *self.packed.read() == *other.packed.read()
    }
}

/// Function type.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    /// Return type
    pub return_type: Type,
    /// Parameter types
    pub param_types: SmallVec<[Type; 8]>,
    /// Whether this function is variadic
    pub is_var_arg: bool,
}

impl FunctionType {
    /// Creates a new function type.
    pub fn new(return_type: Type, param_types: Vec<Type>, is_var_arg: bool) -> Self {
        FunctionType {
            return_type,
            param_types: SmallVec::from_vec(param_types),
            is_var_arg,
        }
    }

    /// Returns the number of parameters.
    pub fn num_params(&self) -> usize {
        self.param_types.len()
    }
}

/// Type data for internal use.
#[derive(Debug)]
pub(crate) enum TypeData {
    Integer(IntegerType),
    Float(FloatType),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_types() {
        let i32_ty = Type::i32();
        assert!(i32_ty.is_integer());
        assert_eq!(i32_ty.size_in_bits(), Some(32));

        let i64_ty = Type::i64();
        assert_eq!(i64_ty.size_in_bits(), Some(64));
    }

    #[test]
    fn test_float_types() {
        let float_ty = Type::float();
        assert!(float_ty.is_float());
        assert_eq!(float_ty.size_in_bits(), Some(32));

        let double_ty = Type::double();
        assert_eq!(double_ty.size_in_bits(), Some(64));
    }

    #[test]
    fn test_pointer_type() {
        let i32_ty = Type::i32();
        let ptr_ty = Type::pointer(i32_ty, 0);
        assert!(ptr_ty.is_pointer());
    }

    #[test]
    fn test_array_type() {
        let i32_ty = Type::i32();
        let arr_ty = Type::array(i32_ty, 10);
        assert_eq!(arr_ty.size_in_bits(), Some(320)); // 32 * 10
    }

    #[test]
    fn test_struct_type() {
        let fields = vec![Type::i32(), Type::i64(), Type::float()];
        let struct_ty = StructType::new(fields, false);
        assert_eq!(struct_ty.num_fields(), 3);
        assert!(!struct_ty.is_opaque());
        assert_eq!(struct_ty.size_in_bits(), Some(32 + 64 + 32));
    }

    #[test]
    fn test_named_struct() {
        let struct_ty = StructType::new_named("MyStruct".to_string());
        assert!(struct_ty.is_opaque());
        assert_eq!(struct_ty.name(), Some("MyStruct"));

        struct_ty.set_body(vec![Type::i32(), Type::i32()], false);
        assert!(!struct_ty.is_opaque());
        assert_eq!(struct_ty.num_fields(), 2);
    }

    #[test]
    fn test_function_type() {
        let ret_ty = Type::i32();
        let params = vec![Type::i32(), Type::i64()];
        let fn_ty = FunctionType::new(ret_ty, params, false);
        assert_eq!(fn_ty.num_params(), 2);
        assert!(!fn_ty.is_var_arg);
    }

    #[test]
    fn test_void_type() {
        let void_ty = Type::void();
        assert!(void_ty.is_void());
        assert_eq!(void_ty.size_in_bits(), None);
    }
}
