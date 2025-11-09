//! Analysis Passes
//!
//! This module provides various analysis passes for LLVM IR.

use std::collections::{HashMap, HashSet};
use crate::function::Function;
use crate::cfg::{CFG, Loop};
use crate::passes::{Pass, AnalysisPass, PassResult};

/// Dominator tree analysis
pub struct DominatorTree {
    /// Maps block index to its immediate dominator
    idoms: Vec<Option<usize>>,
    /// Maps block index to blocks it dominates
    dominated: Vec<HashSet<usize>>,
    /// CFG being analyzed
    cfg: CFG,
}

impl DominatorTree {
    /// Compute dominator tree for a function
    pub fn new(function: &Function) -> Self {
        let cfg = CFG::from_function(function);
        let num_blocks = cfg.num_blocks();

        let mut idoms = vec![None; num_blocks];
        let mut dominated = vec![HashSet::new(); num_blocks];

        // Entry block dominates itself
        if num_blocks > 0 {
            idoms[0] = Some(0);
        }

        // Compute dominators using iterative algorithm
        let mut changed = true;
        while changed {
            changed = false;
            for block in 1..num_blocks {
                let preds = cfg.predecessors(block);
                if preds.is_empty() {
                    continue;
                }

                // Find first processed predecessor
                let mut new_idom = None;
                for &pred in preds {
                    if idoms[pred].is_some() {
                        new_idom = Some(pred);
                        break;
                    }
                }

                // Intersect with all other predecessors
                if let Some(mut idom) = new_idom {
                    for &pred in preds {
                        if idoms[pred].is_some() {
                            idom = Self::intersect(&idoms, pred, idom);
                        }
                    }

                    if idoms[block] != Some(idom) {
                        idoms[block] = Some(idom);
                        changed = true;
                    }
                }
            }
        }

        // Build dominated sets
        for (block, idom) in idoms.iter().enumerate() {
            if let Some(dom) = idom {
                if block != *dom {
                    dominated[*dom].insert(block);
                }
            }
        }

        Self {
            idoms,
            dominated,
            cfg,
        }
    }

    fn intersect(idoms: &[Option<usize>], mut b1: usize, mut b2: usize) -> usize {
        while b1 != b2 {
            while b1 > b2 {
                if let Some(idom) = idoms[b1] {
                    b1 = idom;
                } else {
                    break;
                }
            }
            while b2 > b1 {
                if let Some(idom) = idoms[b2] {
                    b2 = idom;
                } else {
                    break;
                }
            }
        }
        b1
    }

    /// Check if block1 dominates block2
    pub fn dominates(&self, block1: usize, block2: usize) -> bool {
        if block1 == block2 {
            return true;
        }

        let mut current = block2;
        while let Some(idom) = self.idoms[current] {
            if idom == block1 {
                return true;
            }
            if idom == current {
                break;
            }
            current = idom;
        }
        false
    }

    /// Get the immediate dominator of a block
    pub fn idom(&self, block: usize) -> Option<usize> {
        self.idoms[block]
    }

    /// Get the blocks dominated by a block
    pub fn dominated_by(&self, block: usize) -> &HashSet<usize> {
        &self.dominated[block]
    }

    /// Check if block1 strictly dominates block2
    pub fn strictly_dominates(&self, block1: usize, block2: usize) -> bool {
        block1 != block2 && self.dominates(block1, block2)
    }
}

/// Dominator tree pass
pub struct DominatorTreePass;

impl Pass for DominatorTreePass {
    fn name(&self) -> &str {
        "domtree"
    }
}

impl AnalysisPass for DominatorTreePass {
    type Result = DominatorTree;

    fn run_analysis(&mut self, function: &Function) -> PassResult<Self::Result> {
        Ok(DominatorTree::new(function))
    }
}

/// Loop analysis
pub struct LoopInfo {
    loops: Vec<Loop>,
    block_to_loop: HashMap<usize, usize>,
}

impl LoopInfo {
    /// Compute loop information for a function
    pub fn new(function: &Function) -> Self {
        let cfg = CFG::from_function(function);
        let loops = cfg.find_loops();

        let mut block_to_loop = HashMap::new();
        for (i, loop_info) in loops.iter().enumerate() {
            for &block in &loop_info.blocks {
                block_to_loop.insert(block, i);
            }
        }

        Self {
            loops,
            block_to_loop,
        }
    }

    /// Get all loops
    pub fn loops(&self) -> &[Loop] {
        &self.loops
    }

    /// Get the loop containing a block, if any
    pub fn loop_for_block(&self, block: usize) -> Option<&Loop> {
        self.block_to_loop.get(&block).map(|&i| &self.loops[i])
    }

    /// Get the number of loops
    pub fn num_loops(&self) -> usize {
        self.loops.len()
    }
}

/// Loop analysis pass
pub struct LoopAnalysisPass;

impl Pass for LoopAnalysisPass {
    fn name(&self) -> &str {
        "loops"
    }
}

impl AnalysisPass for LoopAnalysisPass {
    type Result = LoopInfo;

    fn run_analysis(&mut self, function: &Function) -> PassResult<Self::Result> {
        Ok(LoopInfo::new(function))
    }
}

/// Simple alias analysis
pub struct AliasAnalysis {
    /// Map of potentially aliasing pointers
    may_alias: HashMap<String, HashSet<String>>,
}

impl AliasAnalysis {
    pub fn new(_function: &Function) -> Self {
        // Simplified - a real implementation would do inter-procedural analysis
        Self {
            may_alias: HashMap::new(),
        }
    }

    /// Check if two values may alias
    pub fn may_alias(&self, v1: &str, v2: &str) -> bool {
        if v1 == v2 {
            return true;
        }

        if let Some(aliases) = self.may_alias.get(v1) {
            if aliases.contains(v2) {
                return true;
            }
        }

        if let Some(aliases) = self.may_alias.get(v2) {
            if aliases.contains(v1) {
                return true;
            }
        }

        false
    }

    /// Check if two values must alias (are the same)
    pub fn must_alias(&self, v1: &str, v2: &str) -> bool {
        v1 == v2
    }

    /// Check if two values definitely don't alias
    pub fn no_alias(&self, v1: &str, v2: &str) -> bool {
        !self.may_alias(v1, v2)
    }
}

/// Alias analysis pass
pub struct AliasAnalysisPass;

impl Pass for AliasAnalysisPass {
    fn name(&self) -> &str {
        "aa"
    }
}

impl AnalysisPass for AliasAnalysisPass {
    type Result = AliasAnalysis;

    fn run_analysis(&mut self, function: &Function) -> PassResult<Self::Result> {
        Ok(AliasAnalysis::new(function))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, BasicBlock};
    use crate::instruction::{Instruction, Opcode};

    #[test]
    fn test_dominator_tree() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        let entry = BasicBlock::new(Some("entry".to_string()));
        entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
        func.add_basic_block(entry);

        let domtree = DominatorTree::new(&func);
        assert!(domtree.dominates(0, 0));
    }

    #[test]
    fn test_loop_info() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        let entry = BasicBlock::new(Some("entry".to_string()));
        entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
        func.add_basic_block(entry);

        let loop_info = LoopInfo::new(&func);
        assert_eq!(loop_info.num_loops(), 0);
    }
}
