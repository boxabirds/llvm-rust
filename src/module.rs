//! LLVM Modules
//!
//! A module is the top-level container for all LLVM IR entities.
//! It contains functions, global variables, and metadata.

use std::sync::{Arc, RwLock};
use std::fmt;
use std::collections::HashMap;
use crate::context::Context;
use crate::function::Function;
use crate::types::Type;
use crate::value::Value;
use crate::metadata::Metadata;

/// Linkage types for global values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    External,
    Private,
    Internal,
    AvailableExternally,
    Linkonce,
    Weak,
    Common,
    Appending,
    ExternWeak,
    LinkonceOdr,
    WeakOdr,
}

impl Default for Linkage {
    fn default() -> Self {
        Linkage::External
    }
}

/// Visibility types for global values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Default,
    Hidden,
    Protected,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Default
    }
}

/// DLL storage class for global values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DLLStorageClass {
    Default,
    DllImport,
    DllExport,
}

impl Default for DLLStorageClass {
    fn default() -> Self {
        DLLStorageClass::Default
    }
}

/// Thread local mode for global variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadLocalMode {
    NotThreadLocal,
    GeneralDynamic,
    LocalDynamic,
    InitialExec,
    LocalExec,
}

impl Default for ThreadLocalMode {
    fn default() -> Self {
        ThreadLocalMode::NotThreadLocal
    }
}

/// Unnamed address type for global values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnnamedAddr {
    None,
    Local,
    Global,
}

impl Default for UnnamedAddr {
    fn default() -> Self {
        UnnamedAddr::None
    }
}

/// A module in LLVM IR
#[derive(Clone)]
pub struct Module {
    data: Arc<RwLock<ModuleData>>,
}

struct ModuleData {
    name: String,
    context: Context,
    functions: Vec<Function>,
    globals: Vec<GlobalVariable>,
    named_metadata: HashMap<String, Vec<Metadata>>,
    module_flags: Vec<Metadata>,
}

/// A global variable in a module
#[derive(Clone)]
pub struct GlobalVariable {
    pub name: String,
    pub ty: Type,
    pub is_constant: bool,
    pub initializer: Option<Value>,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub dll_storage_class: DLLStorageClass,
    pub thread_local_mode: ThreadLocalMode,
    pub unnamed_addr: UnnamedAddr,
    pub addrspace: Option<u32>,
    pub externally_initialized: bool,
    pub section: Option<String>,
    pub alignment: Option<u32>,
    pub comdat: Option<String>,
}

impl Module {
    /// Create a new module with the given name and context
    pub fn new(name: String, context: Context) -> Self {
        Self {
            data: Arc::new(RwLock::new(ModuleData {
                name,
                context,
                functions: Vec::new(),
                globals: Vec::new(),
                named_metadata: HashMap::new(),
                module_flags: Vec::new(),
            })),
        }
    }

    /// Get the name of this module
    pub fn name(&self) -> String {
        self.data.read().unwrap().name.clone()
    }

    /// Get the context associated with this module
    pub fn context(&self) -> Context {
        self.data.read().unwrap().context.clone()
    }

    /// Add a function to this module
    pub fn add_function(&self, function: Function) {
        let mut data = self.data.write().unwrap();
        data.functions.push(function);
    }

    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Option<Function> {
        self.data.read().unwrap()
            .functions
            .iter()
            .find(|f| f.name() == name)
            .cloned()
    }

    /// Get all functions in this module
    pub fn functions(&self) -> Vec<Function> {
        self.data.read().unwrap().functions.clone()
    }

    /// Get the number of functions in this module
    pub fn function_count(&self) -> usize {
        self.data.read().unwrap().functions.len()
    }

    /// Add a global variable to this module
    pub fn add_global(&self, global: GlobalVariable) {
        let mut data = self.data.write().unwrap();
        data.globals.push(global);
    }

    /// Get a global variable by name
    pub fn get_global(&self, name: &str) -> Option<GlobalVariable> {
        self.data.read().unwrap()
            .globals
            .iter()
            .find(|g| g.name == name)
            .cloned()
    }

    /// Get all global variables in this module
    pub fn globals(&self) -> Vec<GlobalVariable> {
        self.data.read().unwrap().globals.clone()
    }

    /// Add named metadata to the module
    pub fn add_named_metadata(&self, name: String, metadata: Vec<Metadata>) {
        let mut data = self.data.write().unwrap();
        data.named_metadata.insert(name, metadata);
    }

    /// Get named metadata by name
    pub fn get_named_metadata(&self, name: &str) -> Option<Vec<Metadata>> {
        self.data.read().unwrap()
            .named_metadata
            .get(name)
            .cloned()
    }

    /// Get all named metadata
    pub fn named_metadata(&self) -> HashMap<String, Vec<Metadata>> {
        self.data.read().unwrap().named_metadata.clone()
    }

    /// Add module flags metadata
    pub fn add_module_flag(&self, flag: Metadata) {
        let mut data = self.data.write().unwrap();
        data.module_flags.push(flag);
    }

    /// Get all module flags
    pub fn module_flags(&self) -> Vec<Metadata> {
        self.data.read().unwrap().module_flags.clone()
    }
}

impl GlobalVariable {
    /// Create a new global variable
    pub fn new(name: String, ty: Type, is_constant: bool, initializer: Option<Value>) -> Self {
        Self {
            name,
            ty,
            is_constant,
            initializer,
            linkage: Linkage::default(),
            visibility: Visibility::default(),
            dll_storage_class: DLLStorageClass::default(),
            thread_local_mode: ThreadLocalMode::default(),
            unnamed_addr: UnnamedAddr::default(),
            addrspace: None,
            externally_initialized: false,
            section: None,
            alignment: None,
            comdat: None,
        }
    }

    /// Create a new global variable with full attributes
    pub fn new_with_attributes(
        name: String,
        ty: Type,
        is_constant: bool,
        initializer: Option<Value>,
        linkage: Linkage,
        visibility: Visibility,
        dll_storage_class: DLLStorageClass,
        thread_local_mode: ThreadLocalMode,
        unnamed_addr: UnnamedAddr,
        addrspace: Option<u32>,
        externally_initialized: bool,
        section: Option<String>,
        alignment: Option<u32>,
        comdat: Option<String>,
    ) -> Self {
        Self {
            name,
            ty,
            is_constant,
            initializer,
            linkage,
            visibility,
            dll_storage_class,
            thread_local_mode,
            unnamed_addr,
            addrspace,
            externally_initialized,
            section,
            alignment,
            comdat,
        }
    }

    /// Get the name of this global variable
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the type of this global variable
    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    /// Check if this global is constant
    pub fn is_constant(&self) -> bool {
        self.is_constant
    }

    /// Get the initializer value, if any
    pub fn initializer(&self) -> Option<&Value> {
        self.initializer.as_ref()
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();

        writeln!(f, "; ModuleID = '{}'", data.name)?;
        writeln!(f)?;

        // Print global variables
        for global in &data.globals {
            write!(f, "@{} = ", global.name)?;
            if global.is_constant {
                write!(f, "constant ")?;
            } else {
                write!(f, "global ")?;
            }
            write!(f, "{}", global.ty)?;
            if let Some(init) = &global.initializer {
                write!(f, " {}", init)?;
            }
            writeln!(f)?;
        }

        if !data.globals.is_empty() {
            writeln!(f)?;
        }

        // Print functions
        for (i, func) in data.functions.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", func)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();
        write!(f, "Module('{}', {} functions, {} globals)",
            data.name,
            data.functions.len(),
            data.globals.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let ctx = Context::new();
        let module = Module::new("test_module".to_string(), ctx);
        assert_eq!(module.name(), "test_module");
        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_add_function() {
        let ctx = Context::new();
        let module = Module::new("test_module".to_string(), ctx.clone());

        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        module.add_function(func);
        assert_eq!(module.function_count(), 1);
        assert!(module.get_function("test").is_some());
    }
}
