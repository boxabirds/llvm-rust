//! Pass Infrastructure
//!
//! This module provides the infrastructure for running analysis and transformation
//! passes on LLVM IR.

use std::collections::HashMap;
use crate::module::Module;
use crate::function::Function;

/// Result of a pass execution
pub type PassResult<T> = Result<T, PassError>;

/// Pass execution errors
#[derive(Debug, Clone)]
pub enum PassError {
    /// Pass failed
    Failed(String),
    /// Pass prerequisites not met
    PrerequisitesNotMet(Vec<String>),
    /// Invalid IR
    InvalidIR(String),
}

/// Base trait for all passes
pub trait Pass {
    /// Get the name of this pass
    fn name(&self) -> &str;

    /// Get the passes that must run before this pass
    fn prerequisites(&self) -> Vec<String> {
        Vec::new()
    }

    /// Check if this pass preserves all analyses
    fn preserves_all(&self) -> bool {
        false
    }

    /// Get the list of preserved analyses
    fn preserved_analyses(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A module pass operates on an entire module
pub trait ModulePass: Pass {
    /// Run this pass on a module
    fn run_on_module(&mut self, module: &mut Module) -> PassResult<bool>;
}

/// A function pass operates on a single function
pub trait FunctionPass: Pass {
    /// Run this pass on a function
    fn run_on_function(&mut self, function: &mut Function) -> PassResult<bool>;
}

/// An analysis pass produces analysis results
pub trait AnalysisPass: Pass {
    type Result;

    /// Run the analysis
    fn run_analysis(&mut self, function: &Function) -> PassResult<Self::Result>;
}

/// Pass manager for running passes
pub struct PassManager {
    module_passes: Vec<Box<dyn ModulePass>>,
    function_passes: Vec<Box<dyn FunctionPass>>,
    analysis_cache: HashMap<String, Box<dyn std::any::Any>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            module_passes: Vec::new(),
            function_passes: Vec::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Add a module pass to the manager
    pub fn add_module_pass(&mut self, pass: Box<dyn ModulePass>) {
        self.module_passes.push(pass);
    }

    /// Add a function pass to the manager
    pub fn add_function_pass(&mut self, pass: Box<dyn FunctionPass>) {
        self.function_passes.push(pass);
    }

    /// Run all passes on a module
    pub fn run(&mut self, module: &mut Module) -> PassResult<()> {
        // Run module passes
        for pass in &mut self.module_passes {
            pass.run_on_module(module)?;
        }

        // Run function passes on each function
        for function in module.functions() {
            for pass in &mut self.function_passes {
                pass.run_on_function(&mut function.clone())?;
            }
        }

        Ok(())
    }

    /// Clear the analysis cache
    pub fn clear_analysis_cache(&mut self) {
        self.analysis_cache.clear();
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Function pass manager
pub struct FunctionPassManager {
    passes: Vec<Box<dyn FunctionPass>>,
}

impl FunctionPassManager {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    /// Add a pass
    pub fn add_pass(&mut self, pass: Box<dyn FunctionPass>) {
        self.passes.push(pass);
    }

    /// Run all passes on a function
    pub fn run(&mut self, function: &mut Function) -> PassResult<()> {
        for pass in &mut self.passes {
            pass.run_on_function(function)?;
        }
        Ok(())
    }
}

impl Default for FunctionPassManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyModulePass;

    impl Pass for DummyModulePass {
        fn name(&self) -> &str {
            "dummy-module-pass"
        }
    }

    impl ModulePass for DummyModulePass {
        fn run_on_module(&mut self, _module: &mut Module) -> PassResult<bool> {
            Ok(false)
        }
    }

    #[test]
    fn test_pass_manager_creation() {
        let mut pm = PassManager::new();
        pm.add_module_pass(Box::new(DummyModulePass));
        assert_eq!(pm.module_passes.len(), 1);
    }
}
