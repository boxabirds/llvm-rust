//! LLVM Metadata System
//!
//! Metadata provides additional information about IR entities without
//! affecting the program semantics. This includes debug information,
//! optimization hints, and other annotations.

use std::sync::Arc;
use std::fmt;
use crate::value::Value;

/// Metadata node
#[derive(Clone)]
pub struct Metadata {
    data: Arc<MetadataData>,
}

#[allow(dead_code)]
enum MetadataData {
    /// String metadata
    String(String),
    /// Integer metadata
    Int(i64),
    /// Float metadata
    Float(f64),
    /// Value as metadata
    Value(Value),
    /// Metadata tuple (list of metadata nodes)
    Tuple(Vec<Metadata>),
    /// Named metadata
    Named { name: String, operands: Vec<Metadata> },
    /// Debug info metadata
    DebugInfo(Box<DebugInfo>),
    /// Reference to numbered metadata (!0, !1, etc.) - to be resolved later
    Reference(String),
}

/// Debug information kinds
#[derive(Clone)]
pub enum DebugInfo {
    /// Compile unit
    CompileUnit {
        language: DwarfLang,
        file: Box<DebugInfo>,
        producer: String,
        optimized: bool,
        flags: String,
        runtime_version: u32,
    },
    /// File
    File {
        filename: String,
        directory: String,
    },
    /// Subprogram (function)
    Subprogram {
        name: String,
        linkage_name: String,
        file: Box<DebugInfo>,
        line: u32,
        scope_line: u32,
        function: Option<Value>,
    },
    /// Local variable
    LocalVariable {
        name: String,
        file: Box<DebugInfo>,
        line: u32,
        arg: u32,
    },
    /// Lexical block
    LexicalBlock {
        file: Box<DebugInfo>,
        line: u32,
        column: u32,
    },
    /// Basic type
    BasicType {
        name: String,
        size: u64,
        encoding: DwarfEncoding,
    },
    /// Derived type (pointer, reference, typedef, etc.)
    DerivedType {
        tag: DwarfTag,
        name: String,
        base_type: Box<DebugInfo>,
        size: u64,
    },
    /// Composite type (struct, union, array, etc.)
    CompositeType {
        tag: DwarfTag,
        name: String,
        elements: Vec<DebugInfo>,
        size: u64,
    },
    /// Location
    Location {
        line: u32,
        column: u32,
        scope: Box<DebugInfo>,
    },
}

/// DWARF language codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwarfLang {
    C89 = 0x0001,
    C = 0x0002,
    Ada83 = 0x0003,
    CPlusPlus = 0x0004,
    Cobol74 = 0x0005,
    Cobol85 = 0x0006,
    Fortran77 = 0x0007,
    Fortran90 = 0x0008,
    Pascal83 = 0x0009,
    Modula2 = 0x000a,
    Java = 0x000b,
    C99 = 0x000c,
    Ada95 = 0x000d,
    Fortran95 = 0x000e,
    PLI = 0x000f,
    ObjC = 0x0010,
    ObjCPlusPlus = 0x0011,
    UPC = 0x0012,
    D = 0x0013,
    Python = 0x0014,
    Rust = 0x0015,
}

/// DWARF type encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwarfEncoding {
    Address = 0x01,
    Boolean = 0x02,
    ComplexFloat = 0x03,
    Float = 0x04,
    Signed = 0x05,
    SignedChar = 0x06,
    Unsigned = 0x07,
    UnsignedChar = 0x08,
}

/// DWARF tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwarfTag {
    ArrayType = 0x01,
    ClassType = 0x02,
    EntryPoint = 0x03,
    EnumerationType = 0x04,
    FormalParameter = 0x05,
    ImportedDeclaration = 0x08,
    Label = 0x0a,
    LexicalBlock = 0x0b,
    Member = 0x0d,
    PointerType = 0x0f,
    ReferenceType = 0x10,
    CompileUnit = 0x11,
    StringType = 0x12,
    StructureType = 0x13,
    SubroutineType = 0x15,
    Typedef = 0x16,
    UnionType = 0x17,
    UnspecifiedParameters = 0x18,
    Variant = 0x19,
}

impl Metadata {
    /// Create string metadata
    pub fn string(s: String) -> Self {
        Self {
            data: Arc::new(MetadataData::String(s)),
        }
    }

    /// Create integer metadata
    pub fn int(val: i64) -> Self {
        Self {
            data: Arc::new(MetadataData::Int(val)),
        }
    }

    /// Create float metadata
    pub fn float(val: f64) -> Self {
        Self {
            data: Arc::new(MetadataData::Float(val)),
        }
    }

    /// Create value metadata
    pub fn value(val: Value) -> Self {
        Self {
            data: Arc::new(MetadataData::Value(val)),
        }
    }

    /// Create tuple metadata
    pub fn tuple(operands: Vec<Metadata>) -> Self {
        Self {
            data: Arc::new(MetadataData::Tuple(operands)),
        }
    }

    /// Create reference to numbered metadata (!0, !1, etc.)
    pub fn reference(name: String) -> Self {
        Self {
            data: Arc::new(MetadataData::Reference(name)),
        }
    }

    /// Create named metadata
    pub fn named(name: String, operands: Vec<Metadata>) -> Self {
        Self {
            data: Arc::new(MetadataData::Named { name, operands }),
        }
    }

    /// Create debug info metadata
    pub fn debug_info(info: DebugInfo) -> Self {
        Self {
            data: Arc::new(MetadataData::DebugInfo(Box::new(info))),
        }
    }

    // Introspection API

    /// Check if this is a string metadata node
    pub fn is_string(&self) -> bool {
        matches!(&*self.data, MetadataData::String(_))
    }

    /// Check if this is an integer metadata node
    pub fn is_int(&self) -> bool {
        matches!(&*self.data, MetadataData::Int(_))
    }

    /// Check if this is a tuple metadata node
    pub fn is_tuple(&self) -> bool {
        matches!(&*self.data, MetadataData::Tuple(_))
    }

    /// Check if this is a reference to numbered metadata
    pub fn is_reference(&self) -> bool {
        matches!(&*self.data, MetadataData::Reference(_))
    }

    /// Get reference name if this is a reference metadata node
    pub fn as_reference(&self) -> Option<&str> {
        match &*self.data {
            MetadataData::Reference(name) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get string value if this is a string metadata node
    pub fn as_string(&self) -> Option<&str> {
        match &*self.data {
            MetadataData::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get integer value if this is an integer metadata node
    pub fn as_int(&self) -> Option<i64> {
        match &*self.data {
            MetadataData::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Get i32 value if this is an integer metadata node
    pub fn as_i32(&self) -> Option<i32> {
        match &*self.data {
            MetadataData::Int(i) => Some(*i as i32),
            _ => None,
        }
    }

    /// Get tuple operands if this is a tuple metadata node
    pub fn as_tuple(&self) -> Option<&Vec<Metadata>> {
        match &*self.data {
            MetadataData::Tuple(operands) => Some(operands),
            _ => None,
        }
    }

    pub fn as_debug_info(&self) -> Option<&DebugInfo> {
        match &*self.data {
            MetadataData::DebugInfo(di) => Some(di),
            _ => None,
        }
    }

    /// Get operands of this metadata node (works for tuples and named metadata)
    pub fn operands(&self) -> Option<&Vec<Metadata>> {
        match &*self.data {
            MetadataData::Tuple(operands) => Some(operands),
            MetadataData::Named { operands, .. } => Some(operands),
            _ => None,
        }
    }

    /// Get the number of operands
    pub fn num_operands(&self) -> usize {
        match &*self.data {
            MetadataData::Tuple(operands) => operands.len(),
            MetadataData::Named { operands, .. } => operands.len(),
            _ => 0,
        }
    }

    /// Get a specific operand by index
    pub fn get_operand(&self, index: usize) -> Option<&Metadata> {
        self.operands().and_then(|ops| ops.get(index))
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.data {
            MetadataData::String(s) => write!(f, "!\"{}\"", s),
            MetadataData::Int(i) => write!(f, "!{}", i),
            MetadataData::Float(fl) => write!(f, "!{}", fl),
            MetadataData::Value(v) => write!(f, "{}", v),
            MetadataData::Reference(name) => write!(f, "!{}", name),
            MetadataData::Tuple(operands) => {
                write!(f, "!{{")?;
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", op)?;
                }
                write!(f, "}}")
            }
            MetadataData::Named { name, operands } => {
                write!(f, "!{} = !{{", name)?;
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", op)?;
                }
                write!(f, "}}")
            }
            MetadataData::DebugInfo(_) => write!(f, "!DINode"),
        }
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metadata(...)")
    }
}

/// Metadata attachment for instructions
#[derive(Clone)]
pub struct MetadataAttachment {
    kind: String,
    metadata: Metadata,
}

impl MetadataAttachment {
    pub fn new(kind: String, metadata: Metadata) -> Self {
        Self { kind, metadata }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_metadata() {
        let md = Metadata::string("test".to_string());
        assert_eq!(format!("{}", md), "!\"test\"");
    }

    #[test]
    fn test_int_metadata() {
        let md = Metadata::int(42);
        assert_eq!(format!("{}", md), "!42");
    }

    #[test]
    fn test_tuple_metadata() {
        let md1 = Metadata::int(1);
        let md2 = Metadata::int(2);
        let tuple = Metadata::tuple(vec![md1, md2]);
        assert_eq!(format!("{}", tuple), "!{!1, !2}");
    }
}
