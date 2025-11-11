//! Stack Frame Management
//!
//! Handles stack frame layout and management for function calls.

use std::collections::HashMap;

/// Stack frame layout manager
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Total frame size in bytes
    frame_size: usize,
    /// Offset for local variables
    local_offset: usize,
    /// Offset for spill slots
    spill_offset: usize,
    /// Map of variable names to stack offsets
    var_offsets: HashMap<String, isize>,
    /// Number of spill slots
    num_spills: usize,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new() -> Self {
        Self {
            frame_size: 0,
            local_offset: 0,
            spill_offset: 0,
            var_offsets: HashMap::new(),
            num_spills: 0,
        }
    }

    /// Allocate space for a local variable
    pub fn allocate_local(&mut self, name: String, size: usize, alignment: usize) -> isize {
        // Align the offset
        self.local_offset = Self::align_up(self.local_offset, alignment);

        // Allocate space
        self.local_offset += size;

        // Offset is negative from frame pointer
        let offset = -(self.local_offset as isize);

        self.var_offsets.insert(name, offset);
        self.frame_size = self.local_offset.max(self.frame_size);

        offset
    }

    /// Allocate a spill slot
    pub fn allocate_spill(&mut self, size: usize) -> isize {
        // Start spill slots after locals
        if self.spill_offset == 0 {
            self.spill_offset = self.local_offset;
        }

        // Allocate spill slot
        self.spill_offset += size;
        self.num_spills += 1;

        let offset = -(self.spill_offset as isize);
        self.frame_size = self.spill_offset.max(self.frame_size);

        offset
    }

    /// Get the offset for a variable
    pub fn get_offset(&self, name: &str) -> Option<isize> {
        self.var_offsets.get(name).copied()
    }

    /// Get the total frame size
    pub fn total_size(&self) -> usize {
        // Align frame size to 16 bytes (System V ABI requirement)
        Self::align_up(self.frame_size, 16)
    }

    /// Get the number of spill slots
    pub fn spill_count(&self) -> usize {
        self.num_spills
    }

    /// Generate prologue code
    pub fn gen_prologue(&self) -> Vec<String> {
        let mut code = Vec::new();

        code.push("\tpush %rbp".to_string());
        code.push("\tmov %rsp, %rbp".to_string());

        let frame_size = self.total_size();
        if frame_size > 0 {
            code.push(format!("\tsub ${}, %rsp", frame_size));
        }

        code
    }

    /// Generate epilogue code
    pub fn gen_epilogue(&self) -> Vec<String> {
        let mut code = Vec::new();

        code.push("\tmov %rbp, %rsp".to_string());
        code.push("\tpop %rbp".to_string());
        code.push("\tret".to_string());

        code
    }

    /// Align a value up to the given alignment
    fn align_up(value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }
}

/// Calling convention handler
pub struct CallingConvention {
    /// Argument registers (System V ABI for x86-64)
    arg_regs: Vec<String>,
    /// Caller-saved registers
    caller_saved: Vec<String>,
    /// Callee-saved registers
    callee_saved: Vec<String>,
}

impl CallingConvention {
    /// Create System V AMD64 calling convention
    pub fn system_v_amd64() -> Self {
        Self {
            arg_regs: vec![
                "%rdi".to_string(),
                "%rsi".to_string(),
                "%rdx".to_string(),
                "%rcx".to_string(),
                "%r8".to_string(),
                "%r9".to_string(),
            ],
            caller_saved: vec![
                "%rax".to_string(),
                "%rcx".to_string(),
                "%rdx".to_string(),
                "%rsi".to_string(),
                "%rdi".to_string(),
                "%r8".to_string(),
                "%r9".to_string(),
                "%r10".to_string(),
                "%r11".to_string(),
            ],
            callee_saved: vec![
                "%rbx".to_string(),
                "%rbp".to_string(),
                "%r12".to_string(),
                "%r13".to_string(),
                "%r14".to_string(),
                "%r15".to_string(),
            ],
        }
    }

    /// Get the register for argument N
    pub fn arg_register(&self, index: usize) -> Option<&str> {
        self.arg_regs.get(index).map(|s| s.as_str())
    }

    /// Check if a register is caller-saved
    pub fn is_caller_saved(&self, reg: &str) -> bool {
        self.caller_saved.contains(&reg.to_string())
    }

    /// Check if a register is callee-saved
    pub fn is_callee_saved(&self, reg: &str) -> bool {
        self.callee_saved.contains(&reg.to_string())
    }

    /// Get all callee-saved registers
    pub fn callee_saved_regs(&self) -> &[String] {
        &self.callee_saved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame::new();
        assert_eq!(frame.frame_size, 0);
        assert_eq!(frame.spill_count(), 0);
    }

    #[test]
    fn test_allocate_local() {
        let mut frame = StackFrame::new();
        let offset = frame.allocate_local("x".to_string(), 8, 8);
        assert!(offset < 0);
        assert_eq!(frame.get_offset("x"), Some(offset));
    }

    #[test]
    fn test_allocate_spill() {
        let mut frame = StackFrame::new();
        frame.allocate_local("x".to_string(), 8, 8);
        let spill_offset = frame.allocate_spill(8);
        assert!(spill_offset < 0);
        assert_eq!(frame.spill_count(), 1);
    }

    #[test]
    fn test_frame_size_alignment() {
        let mut frame = StackFrame::new();
        frame.allocate_local("x".to_string(), 5, 1);
        // Frame should be aligned to 16 bytes
        assert_eq!(frame.total_size() % 16, 0);
    }

    #[test]
    fn test_prologue_generation() {
        let mut frame = StackFrame::new();
        frame.allocate_local("x".to_string(), 8, 8);
        let prologue = frame.gen_prologue();
        assert!(!prologue.is_empty());
        assert!(prologue[0].contains("push"));
    }

    #[test]
    fn test_epilogue_generation() {
        let frame = StackFrame::new();
        let epilogue = frame.gen_epilogue();
        assert!(!epilogue.is_empty());
        assert!(epilogue.last().unwrap().contains("ret"));
    }

    #[test]
    fn test_calling_convention() {
        let cc = CallingConvention::system_v_amd64();
        assert_eq!(cc.arg_register(0), Some("%rdi"));
        assert_eq!(cc.arg_register(1), Some("%rsi"));
        assert!(cc.is_caller_saved("%rax"));
        assert!(cc.is_callee_saved("%rbx"));
    }
}
