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
    /// Named metadata with fields (e.g., DISubrange(count: 20, lowerBound: 1))
    NamedWithFields {
        name: String,
        fields: std::collections::HashMap<String, Metadata>
    },
    /// Typed metadata node (DILocation, DISubrange, DICompositeType, etc.)
    TypedNode(MetadataNode),
    /// Debug info metadata (legacy)
    DebugInfo(Box<DebugInfo>),
    /// Reference to numbered metadata (!0, !1, etc.) - to be resolved later
    Reference(String),
}

/// Typed metadata nodes for structured debug information
#[derive(Clone)]
pub enum MetadataNode {
    /// DILocation - source location information
    DILocation(DILocation),
    /// DISubrange - array bounds information
    DISubrange(DISubrange),
    /// DICompositeType - struct, union, array types
    DICompositeType(DICompositeType),
    /// DIBasicType - fundamental types
    DIBasicType(DIBasicType),
    /// DILocalVariable - local variable debug info
    DILocalVariable(DILocalVariable),
    /// DIExpression - DWARF expression
    DIExpression(DIExpression),
    /// DISubprogram - function debug info
    DISubprogram(DISubprogram),
    /// DIFile - source file information
    DIFile(DIFile),
    /// DICompileUnit - compilation unit
    DICompileUnit(DICompileUnit),
    /// Generic - untyped metadata node
    Generic,
}

/// DILocation - represents a source code location
#[derive(Clone)]
pub struct DILocation {
    pub line: u32,
    pub column: u32,
    pub scope: Option<Box<Metadata>>,
    pub inlined_at: Option<Box<Metadata>>,
}

/// DISubrange - represents array bounds
#[derive(Clone)]
pub struct DISubrange {
    pub count: Option<i64>,
    pub lower_bound: Option<i64>,
    pub upper_bound: Option<i64>,
    pub stride: Option<i64>,
}

/// DICompositeType - represents composite types (struct, union, array, etc.)
#[derive(Clone)]
pub struct DICompositeType {
    pub tag: DwarfTag,
    pub name: Option<String>,
    pub file: Option<Box<Metadata>>,
    pub line: Option<u32>,
    pub scope: Option<Box<Metadata>>,
    pub base_type: Option<Box<Metadata>>,
    pub size: Option<u64>,
    pub align: Option<u64>,
    pub offset: Option<u64>,
    pub flags: Option<u32>,
    pub elements: Option<Box<Metadata>>,
    pub runtime_lang: Option<u32>,
    pub vtable_holder: Option<Box<Metadata>>,
    pub template_params: Option<Box<Metadata>>,
    pub identifier: Option<String>,
    pub discriminator: Option<Box<Metadata>>,
    pub data_location: Option<Box<Metadata>>,
    pub associated: Option<Box<Metadata>>,
    pub allocated: Option<Box<Metadata>>,
    pub rank: Option<Box<Metadata>>,
    pub annotations: Option<Box<Metadata>>,
}

/// DIBasicType - represents fundamental types
#[derive(Clone)]
pub struct DIBasicType {
    pub name: String,
    pub size: u64,
    pub encoding: DwarfEncoding,
    pub align: Option<u64>,
    pub flags: Option<u32>,
}

/// DILocalVariable - represents a local variable
#[derive(Clone)]
pub struct DILocalVariable {
    pub name: String,
    pub arg: Option<u32>,
    pub file: Option<Box<Metadata>>,
    pub line: Option<u32>,
    pub type_ref: Option<Box<Metadata>>,
    pub scope: Option<Box<Metadata>>,
    pub flags: Option<u32>,
}

/// DIExpression - DWARF expression for location descriptions
#[derive(Clone)]
pub struct DIExpression {
    pub operations: Vec<DwarfOp>,
}

/// DWARF operations for DIExpression
#[derive(Clone, Debug)]
pub enum DwarfOp {
    DW_OP_deref,
    DW_OP_plus_uconst(u64),
    DW_OP_constu(u64),
    DW_OP_LLVM_fragment { offset: u64, size: u64 },
    DW_OP_stack_value,
    DW_OP_swap,
    DW_OP_xderef,
    DW_OP_push_object_address,
    DW_OP_over,
    DW_OP_LLVM_convert { offset: u64, encoding: u8 },
    DW_OP_LLVM_tag_offset(u64),
    DW_OP_LLVM_entry_value(u64),
    DW_OP_LLVM_arg(u64),
    Generic(String),
}

/// DISubprogram - represents a function
#[derive(Clone)]
pub struct DISubprogram {
    pub name: String,
    pub linkage_name: Option<String>,
    pub scope: Option<Box<Metadata>>,
    pub file: Option<Box<Metadata>>,
    pub line: Option<u32>,
    pub type_ref: Option<Box<Metadata>>,
    pub scope_line: Option<u32>,
    pub contains_params: Option<Box<Metadata>>,
    pub virtuality: Option<u32>,
    pub virtual_index: Option<u32>,
    pub this_adjustment: Option<i64>,
    pub flags: Option<u32>,
    pub is_optimized: Option<bool>,
    pub unit: Option<Box<Metadata>>,
    pub template_params: Option<Box<Metadata>>,
    pub declaration: Option<Box<Metadata>>,
    pub retained_nodes: Option<Box<Metadata>>,
    pub thrown_types: Option<Box<Metadata>>,
    pub annotations: Option<Box<Metadata>>,
}

/// DIFile - represents a source file
#[derive(Clone)]
pub struct DIFile {
    pub filename: String,
    pub directory: String,
    pub checksumkind: Option<String>,
    pub checksum: Option<String>,
    pub source: Option<String>,
}

/// DICompileUnit - represents a compilation unit
#[derive(Clone)]
pub struct DICompileUnit {
    pub language: DwarfLang,
    pub file: Box<Metadata>,
    pub producer: Option<String>,
    pub is_optimized: bool,
    pub flags: Option<String>,
    pub runtime_version: u32,
    pub split_debug_filename: Option<String>,
    pub emission_kind: Option<u32>,
    pub enums: Option<Box<Metadata>>,
    pub retained_types: Option<Box<Metadata>>,
    pub globals: Option<Box<Metadata>>,
    pub imports: Option<Box<Metadata>>,
    pub macros: Option<Box<Metadata>>,
    pub dwo_id: Option<u64>,
    pub split_debug_inlining: Option<bool>,
    pub debug_info_for_profiling: Option<bool>,
    pub name_table_kind: Option<u32>,
    pub range_lists_are_address_length: Option<bool>,
}

/// Debug information kinds (legacy - to be migrated to MetadataNode)
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
    // LLVM constant names (DW_TAG_*)
    DW_TAG_array_type,
    DW_TAG_structure_type,
    DW_TAG_union_type,
    DW_TAG_class_type,
    DW_TAG_enumeration_type,
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

    /// Create named metadata with fields
    pub fn named_with_fields(name: String, fields: std::collections::HashMap<String, Metadata>) -> Self {
        Self {
            data: Arc::new(MetadataData::NamedWithFields { name, fields }),
        }
    }

    /// Create debug info metadata
    pub fn debug_info(info: DebugInfo) -> Self {
        Self {
            data: Arc::new(MetadataData::DebugInfo(Box::new(info))),
        }
    }

    /// Create typed metadata node
    pub fn typed_node(node: MetadataNode) -> Self {
        Self {
            data: Arc::new(MetadataData::TypedNode(node)),
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

    /// Get typed metadata node
    pub fn as_typed_node(&self) -> Option<&MetadataNode> {
        match &*self.data {
            MetadataData::TypedNode(node) => Some(node),
            _ => None,
        }
    }

    /// Get DICompositeType if this is one
    pub fn as_di_composite_type(&self) -> Option<&DICompositeType> {
        match &*self.data {
            MetadataData::TypedNode(MetadataNode::DICompositeType(ct)) => Some(ct),
            _ => None,
        }
    }

    /// Get DISubrange if this is one
    pub fn as_di_subrange(&self) -> Option<&DISubrange> {
        match &*self.data {
            MetadataData::TypedNode(MetadataNode::DISubrange(sr)) => Some(sr),
            _ => None,
        }
    }

    /// Get DIExpression if this is one
    pub fn as_di_expression(&self) -> Option<&DIExpression> {
        match &*self.data {
            MetadataData::TypedNode(MetadataNode::DIExpression(expr)) => Some(expr),
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
            MetadataData::NamedWithFields { fields, .. } => fields.len(),
            _ => 0,
        }
    }

    /// Get a specific operand by index
    pub fn get_operand(&self, index: usize) -> Option<&Metadata> {
        self.operands().and_then(|ops| ops.get(index))
    }

    /// Get the name of named metadata (e.g., "DISubrange", "DIExpression", etc.)
    pub fn get_name(&self) -> Option<&str> {
        match &*self.data {
            MetadataData::Named { name, .. } => Some(name.as_str()),
            MetadataData::NamedWithFields { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get a field by name from named metadata with fields
    pub fn get_field(&self, field_name: &str) -> Option<&Metadata> {
        match &*self.data {
            MetadataData::NamedWithFields { fields, .. } => fields.get(field_name),
            _ => None,
        }
    }

    /// Check if a field exists in named metadata with fields
    pub fn has_field(&self, field_name: &str) -> bool {
        match &*self.data {
            MetadataData::NamedWithFields { fields, .. } => fields.contains_key(field_name),
            _ => false,
        }
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
            MetadataData::NamedWithFields { name, fields } => {
                write!(f, "!{}(", name)?;
                let mut first = true;
                for (key, value) in fields.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, ")")
            }
            MetadataData::TypedNode(node) => match node {
                MetadataNode::DICompositeType(_) => write!(f, "!DICompositeType(...)"),
                MetadataNode::DISubrange(_) => write!(f, "!DISubrange(...)"),
                MetadataNode::DIExpression(_) => write!(f, "!DIExpression(...)"),
                MetadataNode::DILocation(_) => write!(f, "!DILocation(...)"),
                MetadataNode::DIBasicType(_) => write!(f, "!DIBasicType(...)"),
                MetadataNode::DILocalVariable(_) => write!(f, "!DILocalVariable(...)"),
                MetadataNode::DISubprogram(_) => write!(f, "!DISubprogram(...)"),
                MetadataNode::DIFile(_) => write!(f, "!DIFile(...)"),
                MetadataNode::DICompileUnit(_) => write!(f, "!DICompileUnit(...)"),
                MetadataNode::Generic => write!(f, "!DINode"),
            },
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
