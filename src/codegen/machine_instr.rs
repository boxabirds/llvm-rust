//! Machine Instruction Representation
//!
//! Generic machine instruction representation that can be specialized
//! for different targets.

/// Machine instruction trait
pub trait MachineInstruction {
    /// Check if this is a terminator instruction
    fn is_terminator(&self) -> bool;

    /// Check if this instruction has side effects
    fn has_side_effects(&self) -> bool;

    /// Get the operands that are read
    fn reads(&self) -> Vec<usize>;

    /// Get the operands that are written
    fn writes(&self) -> Vec<usize>;
}
