//! LLVM Metadata
//!
//! Metadata provides additional information about IR constructs without
//! affecting the semantics of the program. It's used for:
//! - Debug information (DWARF)
//! - Type-based alias analysis (TBAA)
//! - Loop optimizations
//! - Profiling data
//! - Custom annotations

use std::sync::Arc;

use crate::value::Value;

/// Metadata node.
#[derive(Debug, Clone)]
pub enum Metadata {
    /// String metadata
    String(String),
    /// Value metadata
    Value(Box<Value>),
    /// Metadata node (tuple of metadata)
    Node(Vec<Metadata>),
    /// Named metadata (like !llvm.dbg.cu)
    Named {
        name: String,
        operands: Vec<Metadata>,
    },
}

impl Metadata {
    /// Creates a string metadata node.
    pub fn string(s: String) -> Self {
        Metadata::String(s)
    }

    /// Creates a metadata node from a list of metadata.
    pub fn node(operands: Vec<Metadata>) -> Self {
        Metadata::Node(operands)
    }

    /// Creates a named metadata node.
    pub fn named(name: String, operands: Vec<Metadata>) -> Self {
        Metadata::Named { name, operands }
    }
}

/// Debug information metadata kinds.
#[derive(Debug, Clone)]
pub enum DebugInfo {
    /// Compile unit (source file)
    CompileUnit {
        language: DwarfLanguage,
        file: String,
        producer: String,
        is_optimized: bool,
        flags: String,
        runtime_version: u32,
    },
    /// File metadata
    File {
        filename: String,
        directory: String,
    },
    /// Subprogram (function)
    Subprogram {
        name: String,
        linkage_name: Option<String>,
        file: Arc<DebugInfo>,
        line: u32,
        ty: Arc<DebugInfo>,
        scope_line: u32,
        is_local: bool,
        is_definition: bool,
    },
    /// Type information
    Type {
        name: String,
        size_in_bits: u64,
        align_in_bits: u32,
        encoding: DwarfTypeEncoding,
    },
    /// Local variable
    LocalVariable {
        name: String,
        file: Arc<DebugInfo>,
        line: u32,
        ty: Arc<DebugInfo>,
    },
    /// Lexical block
    LexicalBlock {
        file: Arc<DebugInfo>,
        line: u32,
        column: u32,
    },
}

/// DWARF language codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DwarfLanguage {
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
    Rust = 0x0019,
}

/// DWARF type encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DwarfTypeEncoding {
    Address = 0x01,
    Boolean = 0x02,
    ComplexFloat = 0x03,
    Float = 0x04,
    Signed = 0x05,
    SignedChar = 0x06,
    Unsigned = 0x07,
    UnsignedChar = 0x08,
}

/// TBAA (Type-Based Alias Analysis) metadata.
#[derive(Debug, Clone)]
pub struct TBAANode {
    /// Type name
    pub name: String,
    /// Parent node (for type hierarchy)
    pub parent: Option<Arc<TBAANode>>,
    /// Whether this is a struct field
    pub is_constant: bool,
}

impl TBAANode {
    /// Creates a new TBAA root node.
    pub fn root(name: String) -> Self {
        TBAANode {
            name,
            parent: None,
            is_constant: false,
        }
    }

    /// Creates a new TBAA node with a parent.
    pub fn with_parent(name: String, parent: Arc<TBAANode>) -> Self {
        TBAANode {
            name,
            parent: Some(parent),
            is_constant: false,
        }
    }
}

/// Loop metadata for optimization hints.
#[derive(Debug, Clone)]
pub struct LoopMetadata {
    /// Loop ID (must be unique and self-referential)
    pub id: Option<Arc<Metadata>>,
    /// Vectorization hints
    pub vectorize: Option<VectorizeHint>,
    /// Unroll hints
    pub unroll: Option<UnrollHint>,
    /// Whether to disable all optimizations
    pub disable_nonforced: bool,
}

/// Vectorization hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorizeHint {
    /// Force vectorization
    Enable,
    /// Disable vectorization
    Disable,
    /// Vectorization width
    Width(u32),
}

/// Unroll hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnrollHint {
    /// Force unrolling
    Enable,
    /// Disable unrolling
    Disable,
    /// Full unrolling
    Full,
    /// Unroll count
    Count(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let md = Metadata::string("test".to_string());
        assert!(matches!(md, Metadata::String(_)));

        let node = Metadata::node(vec![
            Metadata::string("a".to_string()),
            Metadata::string("b".to_string()),
        ]);
        assert!(matches!(node, Metadata::Node(_)));
    }

    #[test]
    fn test_tbaa_node() {
        let root = Arc::new(TBAANode::root("root".to_string()));
        let child = TBAANode::with_parent("child".to_string(), root.clone());
        assert_eq!(child.name, "child");
        assert!(child.parent.is_some());
    }

    #[test]
    fn test_dwarf_language() {
        let lang = DwarfLanguage::Rust;
        assert_eq!(lang as u32, 0x0019);
    }
}
