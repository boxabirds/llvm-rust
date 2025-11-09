//! LLVM Module
//!
//! A module represents a compilation unit in LLVM IR. It contains:
//! - Functions
//! - Global variables
//! - Type definitions
//! - Metadata
//! - Target information

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::context::Context;
use crate::value::Value;

/// An LLVM module.
///
/// A module is the top-level container for all LLVM IR objects.
/// It represents a single compilation unit.
#[derive(Debug)]
pub struct Module {
    /// The name of this module
    name: String,

    /// The context this module belongs to
    context: Arc<Context>,

    /// Functions in this module
    functions: RwLock<HashMap<String, Arc<Function>>>,

    /// Global variables in this module
    globals: RwLock<HashMap<String, Arc<GlobalVariable>>>,

    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    target_triple: RwLock<Option<String>>,

    /// Data layout string
    data_layout: RwLock<Option<String>>,

    /// Source file name
    source_filename: RwLock<Option<String>>,
}

impl Module {
    /// Creates a new module with the given name.
    pub fn new(name: String, context: Arc<Context>) -> Self {
        Module {
            name,
            context,
            functions: RwLock::new(HashMap::new()),
            globals: RwLock::new(HashMap::new()),
            target_triple: RwLock::new(None),
            data_layout: RwLock::new(None),
            source_filename: RwLock::new(None),
        }
    }

    /// Returns the name of this module.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the context this module belongs to.
    pub fn context(&self) -> &Arc<Context> {
        &self.context
    }

    /// Adds a function to this module.
    pub fn add_function(&self, name: String, function: Function) -> Arc<Function> {
        let func = Arc::new(function);
        self.functions.write().insert(name, func.clone());
        func
    }

    /// Gets a function by name.
    pub fn get_function(&self, name: &str) -> Option<Arc<Function>> {
        self.functions.read().get(name).cloned()
    }

    /// Returns all functions in this module.
    pub fn functions(&self) -> Vec<Arc<Function>> {
        self.functions.read().values().cloned().collect()
    }

    /// Adds a global variable to this module.
    pub fn add_global(&self, name: String, global: GlobalVariable) -> Arc<GlobalVariable> {
        let g = Arc::new(global);
        self.globals.write().insert(name, g.clone());
        g
    }

    /// Gets a global variable by name.
    pub fn get_global(&self, name: &str) -> Option<Arc<GlobalVariable>> {
        self.globals.read().get(name).cloned()
    }

    /// Sets the target triple.
    pub fn set_target_triple(&self, triple: String) {
        *self.target_triple.write() = Some(triple);
    }

    /// Gets the target triple.
    pub fn target_triple(&self) -> Option<String> {
        self.target_triple.read().clone()
    }

    /// Sets the data layout.
    pub fn set_data_layout(&self, layout: String) {
        *self.data_layout.write() = Some(layout);
    }

    /// Gets the data layout.
    pub fn data_layout(&self) -> Option<String> {
        self.data_layout.read().clone()
    }

    /// Sets the source filename.
    pub fn set_source_filename(&self, filename: String) {
        *self.source_filename.write() = Some(filename);
    }

    /// Gets the source filename.
    pub fn source_filename(&self) -> Option<String> {
        self.source_filename.read().clone()
    }
}

/// A function in LLVM IR.
#[derive(Debug)]
pub struct Function {
    /// The name of this function
    name: String,

    /// The function type
    ty: crate::types::FunctionType,

    /// Basic blocks in this function
    basic_blocks: RwLock<Vec<Arc<BasicBlock>>>,

    /// Function attributes
    attributes: RwLock<Vec<FunctionAttribute>>,

    /// Linkage type
    linkage: Linkage,

    /// Calling convention
    calling_convention: CallingConvention,
}

impl Function {
    /// Creates a new function.
    pub fn new(
        name: String,
        ty: crate::types::FunctionType,
        linkage: Linkage,
    ) -> Self {
        Function {
            name,
            ty,
            basic_blocks: RwLock::new(Vec::new()),
            attributes: RwLock::new(Vec::new()),
            linkage,
            calling_convention: CallingConvention::C,
        }
    }

    /// Returns the name of this function.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the function type.
    pub fn ty(&self) -> &crate::types::FunctionType {
        &self.ty
    }

    /// Adds a basic block to this function.
    pub fn add_basic_block(&self, bb: BasicBlock) -> Arc<BasicBlock> {
        let bb = Arc::new(bb);
        self.basic_blocks.write().push(bb.clone());
        bb
    }

    /// Returns all basic blocks in this function.
    pub fn basic_blocks(&self) -> Vec<Arc<BasicBlock>> {
        self.basic_blocks.read().clone()
    }

    /// Returns the linkage of this function.
    pub fn linkage(&self) -> Linkage {
        self.linkage
    }

    /// Sets the calling convention.
    pub fn set_calling_convention(&mut self, cc: CallingConvention) {
        self.calling_convention = cc;
    }

    /// Returns the calling convention.
    pub fn calling_convention(&self) -> CallingConvention {
        self.calling_convention
    }
}

/// A basic block in LLVM IR.
#[derive(Debug)]
pub struct BasicBlock {
    /// Optional name for this basic block
    name: Option<String>,

    /// Instructions in this basic block
    instructions: RwLock<Vec<Value>>,
}

impl BasicBlock {
    /// Creates a new basic block.
    pub fn new(name: Option<String>) -> Self {
        BasicBlock {
            name,
            instructions: RwLock::new(Vec::new()),
        }
    }

    /// Returns the name of this basic block.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Adds an instruction to this basic block.
    pub fn add_instruction(&self, inst: Value) {
        self.instructions.write().push(inst);
    }

    /// Returns all instructions in this basic block.
    pub fn instructions(&self) -> Vec<Value> {
        self.instructions.read().clone()
    }
}

/// A global variable in LLVM IR.
#[derive(Debug)]
pub struct GlobalVariable {
    /// The name of this global
    name: String,

    /// The type of this global
    ty: crate::types::Type,

    /// Optional initializer
    initializer: Option<Value>,

    /// Whether this global is constant
    is_constant: bool,

    /// Linkage type
    linkage: Linkage,
}

impl GlobalVariable {
    /// Creates a new global variable.
    pub fn new(
        name: String,
        ty: crate::types::Type,
        is_constant: bool,
        linkage: Linkage,
    ) -> Self {
        GlobalVariable {
            name,
            ty,
            initializer: None,
            is_constant,
            linkage,
        }
    }

    /// Sets the initializer for this global.
    pub fn set_initializer(&mut self, init: Value) {
        self.initializer = Some(init);
    }

    /// Returns the name of this global.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the type of this global.
    pub fn ty(&self) -> &crate::types::Type {
        &self.ty
    }

    /// Returns whether this global is constant.
    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    /// Returns the linkage of this global.
    pub fn linkage(&self) -> Linkage {
        self.linkage
    }
}

/// Linkage types for functions and globals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    /// External linkage (default)
    External,
    /// Internal linkage (static in C)
    Internal,
    /// Private linkage
    Private,
    /// Weak linkage
    Weak,
    /// LinkOnce linkage
    LinkOnce,
    /// Common linkage
    Common,
    /// Appending linkage (for arrays only)
    Appending,
    /// ExternalWeak linkage
    ExternalWeak,
}

/// Calling conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    /// C calling convention
    C,
    /// Fast calling convention
    Fast,
    /// Cold calling convention
    Cold,
    /// X86 stdcall
    X86_StdCall,
    /// X86 fastcall
    X86_FastCall,
}

/// Function attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionAttribute {
    NoReturn,
    NoUnwind,
    ReadOnly,
    WriteOnly,
    AlwaysInline,
    NoInline,
    OptimizeNone,
    OptimizeForSize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FunctionType, Type};

    #[test]
    fn test_module_creation() {
        let ctx = Arc::new(Context::new());
        let module = Module::new("test_module".to_string(), ctx);
        assert_eq!(module.name(), "test_module");
    }

    #[test]
    fn test_add_function() {
        let ctx = Arc::new(Context::new());
        let module = Module::new("test".to_string(), ctx);

        let fn_ty = FunctionType::new(Type::void(), vec![Type::i32()], false);
        let func = Function::new("my_func".to_string(), fn_ty, Linkage::External);

        module.add_function("my_func".to_string(), func);
        assert!(module.get_function("my_func").is_some());
    }

    #[test]
    fn test_basic_block() {
        let bb = BasicBlock::new(Some("entry".to_string()));
        assert_eq!(bb.name(), Some("entry"));
        assert_eq!(bb.instructions().len(), 0);
    }

    #[test]
    fn test_global_variable() {
        let global = GlobalVariable::new(
            "my_global".to_string(),
            Type::i32(),
            true,
            Linkage::External,
        );
        assert_eq!(global.name(), "my_global");
        assert!(global.is_constant());
    }
}
