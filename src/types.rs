//! LLVM Type System
//!
//! This module implements LLVM's type system, including:
//! - Void type
//! - Integer types (i1, i8, i16, i32, i64, etc.)
//! - Floating point types (half, float, double)
//! - Pointer types
//! - Array types
//! - Struct types
//! - Function types

use std::sync::Arc;
use std::fmt;

/// Represents an LLVM type
#[derive(Clone)]
pub struct Type {
    data: Arc<TypeData>,
}

/// Internal representation of type data
pub(crate) enum TypeData {
    Void,
    Integer { bits: u32 },
    Float { kind: FloatKind },
    Pointer { pointee: Type },
    Array { element: Type, size: usize },
    Struct { fields: Vec<Type>, name: Option<String> },
    Function { return_type: Type, param_types: Vec<Type>, is_var_arg: bool },
}

/// Floating point type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatKind {
    Half,   // 16-bit
    Float,  // 32-bit
    Double, // 64-bit
}

impl Type {
    pub(crate) fn new(data: TypeData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    // Type constructors

    pub fn void(ctx: &crate::Context) -> Self {
        let key = "void".to_string();
        let data = ctx.intern_type(key, TypeData::Void);
        Self { data }
    }

    pub fn int(ctx: &crate::Context, bits: u32) -> Self {
        let key = format!("i{}", bits);
        let data = ctx.intern_type(key, TypeData::Integer { bits });
        Self { data }
    }

    pub fn half(ctx: &crate::Context) -> Self {
        let key = "half".to_string();
        let data = ctx.intern_type(key, TypeData::Float { kind: FloatKind::Half });
        Self { data }
    }

    pub fn float(ctx: &crate::Context) -> Self {
        let key = "float".to_string();
        let data = ctx.intern_type(key, TypeData::Float { kind: FloatKind::Float });
        Self { data }
    }

    pub fn double(ctx: &crate::Context) -> Self {
        let key = "double".to_string();
        let data = ctx.intern_type(key, TypeData::Float { kind: FloatKind::Double });
        Self { data }
    }

    pub fn ptr(ctx: &crate::Context, pointee: Type) -> Self {
        let key = format!("ptr<{}>", pointee);
        let data = ctx.intern_type(key, TypeData::Pointer { pointee: pointee.clone() });
        Self { data }
    }

    pub fn array(ctx: &crate::Context, element: Type, size: usize) -> Self {
        let key = format!("[{} x {}]", size, element);
        let data = ctx.intern_type(key, TypeData::Array { element: element.clone(), size });
        Self { data }
    }

    pub fn function(ctx: &crate::Context, return_type: Type, param_types: Vec<Type>, is_var_arg: bool) -> Self {
        let params_str = param_types.iter()
            .map(|t| format!("{}", t))
            .collect::<Vec<_>>()
            .join(", ");
        let vararg_str = if is_var_arg { ", ..." } else { "" };
        let key = format!("fn({}{}) -> {}", params_str, vararg_str, return_type);
        let data = ctx.intern_type(key, TypeData::Function {
            return_type: return_type.clone(),
            param_types: param_types.clone(),
            is_var_arg
        });
        Self { data }
    }

    pub fn struct_type(ctx: &crate::Context, fields: Vec<Type>, name: Option<String>) -> Self {
        let fields_str = fields.iter()
            .map(|t| format!("{}", t))
            .collect::<Vec<_>>()
            .join(", ");
        let key = if let Some(ref n) = name {
            format!("struct {} {{ {} }}", n, fields_str)
        } else {
            format!("{{ {} }}", fields_str)
        };
        let data = ctx.intern_type(key, TypeData::Struct { fields: fields.clone(), name: name.clone() });
        Self { data }
    }

    // Type queries

    pub fn is_void(&self) -> bool {
        matches!(&*self.data, TypeData::Void)
    }

    pub fn is_integer(&self) -> bool {
        matches!(&*self.data, TypeData::Integer { .. })
    }

    pub fn is_float(&self) -> bool {
        matches!(&*self.data, TypeData::Float { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(&*self.data, TypeData::Pointer { .. })
    }

    pub fn is_array(&self) -> bool {
        matches!(&*self.data, TypeData::Array { .. })
    }

    pub fn is_struct(&self) -> bool {
        matches!(&*self.data, TypeData::Struct { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(&*self.data, TypeData::Function { .. })
    }

    /// Get the bit width of an integer type
    pub fn int_width(&self) -> Option<u32> {
        match &*self.data {
            TypeData::Integer { bits } => Some(*bits),
            _ => None,
        }
    }

    /// Get the element type of a pointer
    pub fn pointee_type(&self) -> Option<&Type> {
        match &*self.data {
            TypeData::Pointer { pointee } => Some(pointee),
            _ => None,
        }
    }

    /// Get the element type and size of an array
    pub fn array_info(&self) -> Option<(&Type, usize)> {
        match &*self.data {
            TypeData::Array { element, size } => Some((element, *size)),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.data {
            TypeData::Void => write!(f, "void"),
            TypeData::Integer { bits } => write!(f, "i{}", bits),
            TypeData::Float { kind } => match kind {
                FloatKind::Half => write!(f, "half"),
                FloatKind::Float => write!(f, "float"),
                FloatKind::Double => write!(f, "double"),
            },
            TypeData::Pointer { pointee } => write!(f, "{}*", pointee),
            TypeData::Array { element, size } => write!(f, "[{} x {}]", size, element),
            TypeData::Struct { fields, name } => {
                if let Some(n) = name {
                    write!(f, "%{}", n)
                } else {
                    write!(f, "{{ ")?;
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", field)?;
                    }
                    write!(f, " }}")
                }
            }
            TypeData::Function { return_type, param_types, is_var_arg } => {
                write!(f, "{} (", return_type)?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                if *is_var_arg {
                    if !param_types.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type({})", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[test]
    fn test_integer_types() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        assert!(i32_type.is_integer());
        assert_eq!(i32_type.int_width(), Some(32));
        assert_eq!(format!("{}", i32_type), "i32");
    }

    #[test]
    fn test_float_types() {
        let ctx = Context::new();
        let float_type = ctx.float_type();
        assert!(float_type.is_float());
        assert_eq!(format!("{}", float_type), "float");
    }

    #[test]
    fn test_pointer_type() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let ptr_type = ctx.ptr_type(i32_type);
        assert!(ptr_type.is_pointer());
        assert_eq!(format!("{}", ptr_type), "i32*");
    }

    #[test]
    fn test_array_type() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let array_type = ctx.array_type(i32_type, 10);
        assert!(array_type.is_array());
        assert_eq!(format!("{}", array_type), "[10 x i32]");
    }

    #[test]
    fn test_function_type() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
        assert!(fn_type.is_function());
    }
}
