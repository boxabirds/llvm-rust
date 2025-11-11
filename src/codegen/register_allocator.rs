//! Register Allocator
//!
//! Implements linear scan register allocation algorithm.

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

/// Virtual register
pub type VReg = usize;

/// Physical register
pub type PReg = usize;

/// Live range for a virtual register
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRange {
    pub vreg: VReg,
    pub start: usize,
    pub end: usize,
}

impl Ord for LiveRange {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by start point (reversed for BinaryHeap min-heap behavior)
        other.start.cmp(&self.start)
    }
}

impl PartialOrd for LiveRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Active range tracking
#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveRange {
    vreg: VReg,
    end: usize,
    preg: PReg,
}

impl Ord for ActiveRange {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by end point (reversed for min-heap)
        other.end.cmp(&self.end)
    }
}

impl PartialOrd for ActiveRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Register allocator using linear scan algorithm
pub struct RegisterAllocator {
    /// Mapping from virtual to physical registers
    allocation: HashMap<VReg, PReg>,
    /// Mapping from virtual registers to spill locations
    spills: HashMap<VReg, usize>,
    /// Next virtual register to allocate
    next_vreg: VReg,
    /// Available physical registers
    available_pregs: Vec<PReg>,
    /// Next spill slot
    next_spill_slot: usize,
}

impl RegisterAllocator {
    /// Create a new register allocator
    pub fn new(num_physical_regs: usize) -> Self {
        Self {
            allocation: HashMap::new(),
            spills: HashMap::new(),
            next_vreg: 0,
            available_pregs: (0..num_physical_regs).collect(),
            next_spill_slot: 0,
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

    /// Check if a virtual register is spilled
    pub fn is_spilled(&self, vreg: VReg) -> bool {
        self.spills.contains_key(&vreg)
    }

    /// Get the spill location for a virtual register
    pub fn spill_location(&self, vreg: VReg) -> Option<usize> {
        self.spills.get(&vreg).copied()
    }

    /// Perform linear scan register allocation
    pub fn allocate(&mut self, live_ranges: &[LiveRange]) -> Result<(), String> {
        // Sort live ranges by start point
        let mut ranges: Vec<_> = live_ranges.iter().cloned().collect();
        ranges.sort_by_key(|r| r.start);

        // Active ranges (currently live)
        let mut active: BinaryHeap<ActiveRange> = BinaryHeap::new();

        // Free registers pool
        let mut free_regs = self.available_pregs.clone();

        for range in ranges {
            // Expire old intervals
            self.expire_old_intervals(&mut active, &mut free_regs, range.start);

            if free_regs.is_empty() {
                // Need to spill
                self.spill_at_interval(&mut active, &mut free_regs, &range)?;
            } else {
                // Allocate a free register
                let preg = free_regs.pop().unwrap();
                self.allocation.insert(range.vreg, preg);
                active.push(ActiveRange {
                    vreg: range.vreg,
                    end: range.end,
                    preg,
                });
            }
        }

        Ok(())
    }

    /// Expire intervals that have ended
    fn expire_old_intervals(
        &mut self,
        active: &mut BinaryHeap<ActiveRange>,
        free_regs: &mut Vec<PReg>,
        current_point: usize,
    ) {
        let mut expired = Vec::new();

        // Collect expired intervals
        while let Some(range) = active.peek() {
            if range.end < current_point {
                expired.push(active.pop().unwrap());
            } else {
                break;
            }
        }

        // Free their registers
        for range in expired {
            free_regs.push(range.preg);
        }
    }

    /// Spill the interval with the furthest end point
    fn spill_at_interval(
        &mut self,
        active: &mut BinaryHeap<ActiveRange>,
        free_regs: &mut Vec<PReg>,
        new_range: &LiveRange,
    ) -> Result<(), String> {
        // Get the active interval with the furthest end point
        let last = active.peek().ok_or("No active intervals to spill")?;

        if last.end > new_range.end {
            // Spill the last interval
            let spill = active.pop().unwrap();

            // Allocate spill slot
            let spill_slot = self.next_spill_slot;
            self.next_spill_slot += 1;
            self.spills.insert(spill.vreg, spill_slot);

            // Remove from allocation
            self.allocation.remove(&spill.vreg);

            // Assign the register to the new range
            self.allocation.insert(new_range.vreg, spill.preg);
            active.push(ActiveRange {
                vreg: new_range.vreg,
                end: new_range.end,
                preg: spill.preg,
            });

            Ok(())
        } else {
            // Spill the new interval
            let spill_slot = self.next_spill_slot;
            self.next_spill_slot += 1;
            self.spills.insert(new_range.vreg, spill_slot);

            Ok(())
        }
    }

    /// Get total number of spill slots used
    pub fn num_spill_slots(&self) -> usize {
        self.next_spill_slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_allocator_creation() {
        let allocator = RegisterAllocator::new(8);
        assert_eq!(allocator.available_pregs.len(), 8);
    }

    #[test]
    fn test_virtual_register_allocation() {
        let mut allocator = RegisterAllocator::new(4);
        let v0 = allocator.new_vreg();
        let v1 = allocator.new_vreg();
        assert_eq!(v0, 0);
        assert_eq!(v1, 1);
    }

    #[test]
    fn test_linear_scan_simple() {
        let mut allocator = RegisterAllocator::new(2);

        let ranges = vec![
            LiveRange { vreg: 0, start: 0, end: 10 },
            LiveRange { vreg: 1, start: 5, end: 15 },
        ];

        let result = allocator.allocate(&ranges);
        assert!(result.is_ok());

        // Both should get physical registers
        assert!(allocator.get(0).is_some());
        assert!(allocator.get(1).is_some());
    }

    #[test]
    fn test_linear_scan_with_spilling() {
        let mut allocator = RegisterAllocator::new(2);

        // Three live ranges with only 2 physical registers
        let ranges = vec![
            LiveRange { vreg: 0, start: 0, end: 20 },
            LiveRange { vreg: 1, start: 5, end: 15 },
            LiveRange { vreg: 2, start: 10, end: 25 },
        ];

        let result = allocator.allocate(&ranges);
        assert!(result.is_ok());

        // At least one should be spilled
        let spilled_count = (0..3).filter(|&v| allocator.is_spilled(v)).count();
        assert!(spilled_count > 0);
    }

    #[test]
    fn test_expire_intervals() {
        let mut allocator = RegisterAllocator::new(2);
        let mut active = BinaryHeap::new();
        let mut free_regs = vec![0, 1];

        // Add an active range
        active.push(ActiveRange {
            vreg: 0,
            end: 10,
            preg: 0,
        });
        free_regs.pop();

        // Expire at point 15 (after range ends)
        allocator.expire_old_intervals(&mut active, &mut free_regs, 15);

        // The register should be freed
        assert_eq!(free_regs.len(), 2);
        assert!(active.is_empty());
    }
}
