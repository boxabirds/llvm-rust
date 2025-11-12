///! Value Tracker
///!
///! Tracks where values are located (registers, stack slots, immediates)

use std::collections::HashMap;
use crate::value::Value;
use super::x86_64::registers::X86Register;

/// Location where a value resides
#[derive(Debug, Clone)]
pub enum ValueLocation {
    /// Value is in a register
    Register(X86Register),
    /// Value is in a virtual register (before allocation)
    VirtualRegister(usize),
    /// Value is an immediate constant
    Immediate(i64),
    /// Value is on the stack
    Stack(i64), // offset from RBP
    /// Value is a global symbol
    Symbol(String),
}

/// Tracks the location of IR values during codegen
pub struct ValueTracker {
    /// Map from value name to location
    locations: HashMap<String, ValueLocation>,
    /// Next virtual register number
    next_vreg: usize,
}

impl ValueTracker {
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            next_vreg: 0,
        }
    }

    /// Get the location of a value
    pub fn get_location(&self, value: &Value) -> Option<&ValueLocation> {
        value.name().and_then(|name| self.locations.get(name))
    }

    /// Allocate a new virtual register for a value
    pub fn allocate_vreg(&mut self, value: &Value) -> usize {
        let vreg = self.next_vreg;
        self.next_vreg += 1;
        if let Some(name) = value.name() {
            self.locations.insert(name.to_string(), ValueLocation::VirtualRegister(vreg));
        }
        vreg
    }

    /// Set value location
    pub fn set_location(&mut self, value: &Value, location: ValueLocation) {
        if let Some(name) = value.name() {
            self.locations.insert(name.to_string(), location);
        }
    }

    /// Get value as immediate if it's a constant
    pub fn get_immediate(&self, value: &Value) -> Option<i64> {
        // Check if it's a constant integer
        if let Some(const_val) = value.as_const_int() {
            return Some(const_val);
        }

        // Check if we've tracked it as an immediate
        if let Some(ValueLocation::Immediate(imm)) = self.get_location(value) {
            return Some(*imm);
        }

        None
    }

    /// Check if value is a constant
    pub fn is_constant(&self, value: &Value) -> bool {
        value.is_constant() || matches!(self.get_location(value), Some(ValueLocation::Immediate(_)))
    }
}

impl Default for ValueTracker {
    fn default() -> Self {
        Self::new()
    }
}
