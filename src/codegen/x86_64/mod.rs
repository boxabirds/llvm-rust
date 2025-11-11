//! x86-64 Backend
//!
//! This module implements code generation for the x86-64 architecture.

pub mod registers;
pub mod instructions;
pub mod calling_convention;
pub mod asm_printer;

use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::{Instruction, Opcode};
use super::{TargetMachine, CodegenError};
use registers::*;
use std::collections::HashMap;

/// x86-64 target machine
pub struct X86_64TargetMachine {
    /// Function to register mapping
    reg_map: HashMap<String, X86Register>,
}

impl X86_64TargetMachine {
    /// Create a new x86-64 target machine
    pub fn new() -> Self {
        Self {
            reg_map: HashMap::new(),
        }
    }

    /// Select instructions for a function
    fn select_instructions(&mut self, function: &Function) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut machine_instrs = Vec::new();

        // Emit function prologue
        machine_instrs.push(MachineInstr::Push(X86Register::RBP));
        machine_instrs.push(MachineInstr::Mov {
            dest: X86Register::RBP,
            src: MachineOperand::Register(X86Register::RSP),
        });

        // Process each basic block
        for bb in function.basic_blocks() {
            let bb_instrs = self.select_basic_block_instructions(&bb)?;
            machine_instrs.extend(bb_instrs);
        }

        Ok(machine_instrs)
    }

    /// Select instructions for a basic block
    fn select_basic_block_instructions(&mut self, bb: &BasicBlock) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut machine_instrs = Vec::new();

        // Add label
        if let Some(name) = bb.name() {
            machine_instrs.push(MachineInstr::Label(name.to_string()));
        }

        // Process each instruction
        for inst in bb.instructions() {
            let selected = self.select_instruction(&inst)?;
            machine_instrs.extend(selected);
        }

        Ok(machine_instrs)
    }

    /// Select machine instructions for an IR instruction
    fn select_instruction(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        match inst.opcode() {
            Opcode::Add => self.select_add(inst),
            Opcode::Sub => self.select_sub(inst),
            Opcode::Mul => self.select_mul(inst),
            Opcode::Ret => self.select_ret(inst),
            Opcode::Load => self.select_load(inst),
            Opcode::Store => self.select_store(inst),
            _ => Err(CodegenError::UnsupportedInstruction(format!("{:?}", inst.opcode()))),
        }
    }

    fn select_add(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Simplified: add eax, ebx
        Ok(vec![MachineInstr::Add {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_sub(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Sub {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_mul(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::IMul {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_ret(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut instrs = Vec::new();

        // If returning a value, move it to RAX
        if !inst.operands().is_empty() {
            // Simplified: assume value is already in a register
            // In a full implementation, we'd track where values are
        }

        // Epilogue
        instrs.push(MachineInstr::Mov {
            dest: X86Register::RSP,
            src: MachineOperand::Register(X86Register::RBP),
        });
        instrs.push(MachineInstr::Pop(X86Register::RBP));
        instrs.push(MachineInstr::Ret);

        Ok(instrs)
    }

    fn select_load(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Memory {
                base: X86Register::RBP,
                offset: -8,
            },
        }])
    }

    fn select_store(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RBP, // Simplified
            src: MachineOperand::Register(X86Register::RAX),
        }])
    }
}

impl TargetMachine for X86_64TargetMachine {
    fn emit_assembly(&mut self, module: &Module) -> Result<String, CodegenError> {
        let mut asm = String::new();

        // Emit header
        asm.push_str("\t.text\n");

        // Process each function
        for function in module.functions() {
            let function_asm = self.emit_function_assembly(&function)?;
            asm.push_str(&function_asm);
        }

        Ok(asm)
    }

    fn emit_function(&mut self, function: &Function) -> Result<Vec<u8>, CodegenError> {
        // For now, just return empty - full implementation would generate machine code
        let _ = self.select_instructions(function)?;
        Ok(Vec::new())
    }
}

impl X86_64TargetMachine {
    fn emit_function_assembly(&mut self, function: &Function) -> Result<String, CodegenError> {
        let mut asm = String::new();

        // Emit function label
        asm.push_str(&format!("\t.globl {}\n", function.name()));
        asm.push_str(&format!("\t.type {}, @function\n", function.name()));
        asm.push_str(&format!("{}:\n", function.name()));

        // Select and emit instructions
        let machine_instrs = self.select_instructions(function)?;
        for instr in machine_instrs {
            asm.push_str(&format!("\t{}\n", instr));
        }

        Ok(asm)
    }
}

/// Machine instruction representation
#[derive(Debug, Clone)]
pub enum MachineInstr {
    /// Label
    Label(String),
    /// Move
    Mov { dest: X86Register, src: MachineOperand },
    /// Add
    Add { dest: X86Register, src: MachineOperand },
    /// Subtract
    Sub { dest: X86Register, src: MachineOperand },
    /// Multiply
    IMul { dest: X86Register, src: MachineOperand },
    /// Push
    Push(X86Register),
    /// Pop
    Pop(X86Register),
    /// Return
    Ret,
}

impl std::fmt::Display for MachineInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineInstr::Label(l) => write!(f, "{}:", l),
            MachineInstr::Mov { dest, src } => write!(f, "mov {}, {}", dest, src),
            MachineInstr::Add { dest, src } => write!(f, "add {}, {}", dest, src),
            MachineInstr::Sub { dest, src } => write!(f, "sub {}, {}", dest, src),
            MachineInstr::IMul { dest, src } => write!(f, "imul {}, {}", dest, src),
            MachineInstr::Push(reg) => write!(f, "push {}", reg),
            MachineInstr::Pop(reg) => write!(f, "pop {}", reg),
            MachineInstr::Ret => write!(f, "ret"),
        }
    }
}

/// Machine operand
#[derive(Debug, Clone)]
pub enum MachineOperand {
    /// Register
    Register(X86Register),
    /// Immediate value
    Immediate(i64),
    /// Memory location
    Memory { base: X86Register, offset: i64 },
}

impl std::fmt::Display for MachineOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineOperand::Register(reg) => write!(f, "{}", reg),
            MachineOperand::Immediate(imm) => write!(f, "${}", imm),
            MachineOperand::Memory { base, offset } => {
                if *offset >= 0 {
                    write!(f, "{}({})", offset, base)
                } else {
                    write!(f, "{}({})", offset, base)
                }
            }
        }
    }
}
