//! Pass Infrastructure
//!
//! This module provides the infrastructure for running analysis and transformation
//! passes on LLVM IR.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
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

/// Type for pass constructor functions
type PassConstructor = Arc<dyn Fn() -> Box<dyn FunctionPass> + Send + Sync>;

/// Global pass registry for registering and creating passes by name
pub struct PassRegistry {
    passes: HashMap<String, PassConstructor>,
}

impl PassRegistry {
    fn new() -> Self {
        Self {
            passes: HashMap::new(),
        }
    }

    /// Register a function pass with a constructor
    pub fn register_pass<F>(&mut self, name: String, constructor: F)
    where
        F: Fn() -> Box<dyn FunctionPass> + Send + Sync + 'static,
    {
        self.passes.insert(name, Arc::new(constructor));
    }

    /// Create a pass by name
    pub fn create_pass(&self, name: &str) -> Option<Box<dyn FunctionPass>> {
        self.passes.get(name).map(|ctor| ctor())
    }

    /// Get all registered pass names
    pub fn registered_passes(&self) -> Vec<String> {
        self.passes.keys().cloned().collect()
    }

    /// Get the global pass registry instance
    pub fn global() -> Arc<Mutex<PassRegistry>> {
        static INIT: Once = Once::new();
        static mut REGISTRY: Option<Arc<Mutex<PassRegistry>>> = None;

        unsafe {
            INIT.call_once(|| {
                REGISTRY = Some(Arc::new(Mutex::new(PassRegistry::new())));
            });
            REGISTRY.clone().unwrap()
        }
    }
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

    /// Add a function pass by name from the registry
    pub fn add_pass_by_name(&mut self, name: &str) -> Result<(), String> {
        let registry = PassRegistry::global();
        let registry_lock = registry.lock().unwrap();

        if let Some(pass) = registry_lock.create_pass(name) {
            self.function_passes.push(pass);
            Ok(())
        } else {
            Err(format!("Pass '{}' not found in registry", name))
        }
    }

    /// Sort passes based on their dependencies (topological sort)
    fn sort_passes(&mut self) -> PassResult<()> {
        // Extract pass names and prerequisites
        let mut pass_info: Vec<(String, Vec<String>)> = Vec::new();
        for pass in &self.function_passes {
            pass_info.push((pass.name().to_string(), pass.prerequisites()));
        }

        // Perform topological sort
        let _sorted_indices = self.topological_sort(&pass_info)?;

        // Note: Reordering passes based on sorted indices would require
        // restructuring the trait object storage. For now, we just validate
        // that prerequisites are met when running (see validate_prerequisites).
        // A full implementation would reorder self.function_passes here.

        Ok(())
    }

    /// Topological sort helper
    fn topological_sort(&self, pass_info: &[(String, Vec<String>)]) -> PassResult<Vec<usize>> {
        let n = pass_info.len();
        let mut in_degree = vec![0; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        // Build adjacency list and in-degree
        for i in 0..n {
            for prereq in &pass_info[i].1 {
                // Find prerequisite index
                if let Some(j) = pass_info.iter().position(|(name, _)| name == prereq) {
                    adj[j].push(i);
                    in_degree[i] += 1;
                } else {
                    // Prerequisite not found - this is just a warning
                    // In a real implementation, we might require all prerequisites
                }
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<usize> = Vec::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push(i);
            }
        }

        let mut result = Vec::new();
        while let Some(u) = queue.pop() {
            result.push(u);
            for &v in &adj[u] {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push(v);
                }
            }
        }

        if result.len() != n {
            return Err(PassError::PrerequisitesNotMet(vec![
                "Circular dependency detected in pass ordering".to_string()
            ]));
        }

        Ok(result)
    }

    /// Validate that all pass prerequisites are available
    fn validate_prerequisites(&self) -> PassResult<()> {
        let available_passes: std::collections::HashSet<String> =
            self.function_passes.iter().map(|p| p.name().to_string()).collect();

        for pass in &self.function_passes {
            for prereq in pass.prerequisites() {
                if !available_passes.contains(&prereq) {
                    return Err(PassError::PrerequisitesNotMet(vec![
                        format!("Pass '{}' requires '{}' but it's not in the pass list",
                                pass.name(), prereq)
                    ]));
                }
            }
        }

        Ok(())
    }

    /// Run all passes on a module
    pub fn run(&mut self, module: &mut Module) -> PassResult<()> {
        // Validate prerequisites before running
        self.validate_prerequisites()?;

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
