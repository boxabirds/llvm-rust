//! Register Allocator
//!
//! Implements a simple linear scan register allocator.

use std::collections::HashMap;

/// Virtual register
pub type VReg = usize;

/// Physical register
pub type PReg = usize;

/// Register allocator
pub struct RegisterAllocator {
    /// Mapping from virtual to physical registers
    allocation: HashMap<VReg, PReg>,
    /// Next virtual register to allocate
    next_vreg: VReg,
    /// Available physical registers
    available_pregs: Vec<PReg>,
}

impl RegisterAllocator {
    /// Create a new register allocator
    pub fn new(num_physical_regs: usize) -> Self {
        Self {
            allocation: HashMap::new(),
            next_vreg: 0,
            available_pregs: (0..num_physical_regs).collect(),
        }
    }

    /// Allocate a new virtual register
    pub fn new_vreg(&mut self) -> VReg {
        let vreg = self.next_vreg;
        self.next_vreg += 1;
        vreg
    }

    /// Assign a virtual register to a physical register
    pub fn assign(&mut self, vreg: VReg, preg: PReg) {
        self.allocation.insert(vreg, preg);
    }

    /// Get the physical register for a virtual register
    pub fn get(&self, vreg: VReg) -> Option<PReg> {
        self.allocation.get(&vreg).copied()
    }

    /// Perform linear scan register allocation
    pub fn allocate(&mut self, _live_ranges: &[(VReg, usize, usize)]) -> Result<(), String> {
        // Simplified implementation
        // A full implementation would:
        // 1. Sort live ranges by start point
        // 2. Walk through sorted list
        // 3. Free expired registers
        // 4. Allocate register for current range
        // 5. Spill if no registers available

        Ok(())
    }
}
