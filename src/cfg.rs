//! Control Flow Graph Utilities
//!
//! This module provides utilities for analyzing and manipulating control flow graphs.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::Opcode;

/// Control Flow Graph
#[allow(dead_code)]
#[derive(Clone)]
pub struct CFG {
    /// Maps block names to their indices
    block_indices: HashMap<String, usize>,
    /// Successor edges
    successors: Vec<Vec<usize>>,
    /// Predecessor edges
    predecessors: Vec<Vec<usize>>,
    /// Basic blocks
    blocks: Vec<BasicBlock>,
}

impl CFG {
    /// Build a CFG from a function
    pub fn from_function(function: &Function) -> Self {
        let blocks = function.basic_blocks();
        let mut block_indices = HashMap::new();
        let successors = vec![Vec::new(); blocks.len()];
        let predecessors = vec![Vec::new(); blocks.len()];

        // Build block index map
        for (i, bb) in blocks.iter().enumerate() {
            if let Some(name) = bb.name() {
                block_indices.insert(name, i);
            } else {
                block_indices.insert(format!("bb{}", i), i);
            }
        }

        // Build successor/predecessor edges
        for (_i, bb) in blocks.iter().enumerate() {
            if let Some(term) = bb.terminator() {
                match term.opcode() {
                    Opcode::Br => {
                        // Unconditional branch - one successor
                        // In a real implementation, we'd extract the target from operands
                    }
                    Opcode::CondBr => {
                        // Conditional branch - two successors
                        // In a real implementation, we'd extract the targets from operands
                    }
                    Opcode::Switch => {
                        // Switch - multiple successors
                        // In a real implementation, we'd extract the targets from operands
                    }
                    Opcode::Ret | Opcode::Unreachable => {
                        // No successors
                    }
                    _ => {}
                }
            }
        }

        Self {
            block_indices,
            successors,
            predecessors,
            blocks,
        }
    }

    /// Get the successors of a block
    pub fn successors(&self, block_index: usize) -> &[usize] {
        &self.successors[block_index]
    }

    /// Get the predecessors of a block
    pub fn predecessors(&self, block_index: usize) -> &[usize] {
        &self.predecessors[block_index]
    }

    /// Get the number of blocks
    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Check if a block is reachable from the entry block
    pub fn is_reachable(&self, block_index: usize) -> bool {
        self.reachable_blocks().contains(&block_index)
    }

    /// Get all reachable blocks from the entry block
    pub fn reachable_blocks(&self) -> HashSet<usize> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from entry block (index 0)
        queue.push_back(0);
        reachable.insert(0);

        while let Some(current) = queue.pop_front() {
            for &succ in self.successors(current) {
                if reachable.insert(succ) {
                    queue.push_back(succ);
                }
            }
        }

        reachable
    }

    /// Compute reverse postorder traversal
    pub fn reverse_postorder(&self) -> Vec<usize> {
        let mut visited: HashSet<usize> = HashSet::new();
        let mut postorder = Vec::new();

        self.dfs_postorder(0, &mut visited, &mut postorder);

        postorder.reverse();
        postorder
    }

    fn dfs_postorder(&self, block: usize, visited: &mut HashSet<usize>, postorder: &mut Vec<usize>) {
        if !visited.insert(block) {
            return;
        }

        for &succ in self.successors(block) {
            self.dfs_postorder(succ, visited, postorder);
        }

        postorder.push(block);
    }

    /// Find loops in the CFG using a simple backedge detection
    pub fn find_loops(&self) -> Vec<Loop> {
        let mut loops = Vec::new();
        let rpo = self.reverse_postorder();
        let mut _visited: HashSet<usize> = HashSet::new();

        for &block in &rpo {
            for &succ in self.successors(block) {
                // If successor appears before block in RPO, it's a backedge
                if rpo.iter().position(|&b| b == succ) < rpo.iter().position(|&b| b == block) {
                    // Found a loop with header at succ
                    let loop_blocks = self.find_loop_blocks(succ, block);
                    loops.push(Loop {
                        header: succ,
                        blocks: loop_blocks,
                    });
                }
            }
        }

        loops
    }

    fn find_loop_blocks(&self, header: usize, back_edge_source: usize) -> HashSet<usize> {
        let mut loop_blocks = HashSet::new();
        let mut queue = VecDeque::new();

        loop_blocks.insert(header);
        queue.push_back(back_edge_source);
        loop_blocks.insert(back_edge_source);

        while let Some(current) = queue.pop_front() {
            for &pred in self.predecessors(current) {
                if loop_blocks.insert(pred) {
                    queue.push_back(pred);
                }
            }
        }

        loop_blocks
    }
}

/// Represents a loop in the CFG
#[derive(Clone)]
pub struct Loop {
    /// Loop header block
    pub header: usize,
    /// All blocks in the loop
    pub blocks: HashSet<usize>,
}

impl Loop {
    /// Check if a block is in this loop
    pub fn contains(&self, block: usize) -> bool {
        self.blocks.contains(&block)
    }

    /// Get the number of blocks in the loop
    pub fn size(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, BasicBlock};
    use crate::instruction::{Instruction, Opcode};

    #[test]
    fn test_cfg_creation() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        let entry = BasicBlock::new(Some("entry".to_string()));
        entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
        func.add_basic_block(entry);

        let cfg = CFG::from_function(&func);
        assert_eq!(cfg.num_blocks(), 1);
    }

    #[test]
    fn test_reachable_blocks() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("test".to_string(), fn_type);

        let entry = BasicBlock::new(Some("entry".to_string()));
        entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
        func.add_basic_block(entry);

        let cfg = CFG::from_function(&func);
        let reachable = cfg.reachable_blocks();
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&0));
    }
}
