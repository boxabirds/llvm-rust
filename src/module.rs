//! LLVM Modules
//!
//! A module is the top-level container for all LLVM IR entities.
//! It contains functions, global variables, and metadata.

use std::sync::{Arc, RwLock};
use std::fmt;
use crate::context::Context;
use crate::function::Function;
use crate::types::Type;
use crate::value::Value;

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
}

/// A global variable in a module
#[derive(Clone)]
pub struct GlobalVariable {
    name: String,
    ty: Type,
    is_constant: bool,
    initializer: Option<Value>,
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
}

impl GlobalVariable {
    /// Create a new global variable
    pub fn new(name: String, ty: Type, is_constant: bool, initializer: Option<Value>) -> Self {
        Self {
            name,
            ty,
            is_constant,
            initializer,
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
