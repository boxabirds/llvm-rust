//! x86-64 Backend
//!
//! This module implements code generation for the x86-64 architecture.

pub mod registers;
pub mod instructions;
pub mod calling_convention;
pub mod asm_printer;
pub mod instruction_selection;

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
            // Arithmetic
            Opcode::Add | Opcode::FAdd => self.select_add(inst),
            Opcode::Sub | Opcode::FSub => self.select_sub(inst),
            Opcode::Mul | Opcode::FMul => self.select_mul(inst),
            Opcode::UDiv | Opcode::SDiv | Opcode::FDiv => self.select_div(inst),
            Opcode::URem | Opcode::SRem | Opcode::FRem => self.select_rem(inst),

            // Bitwise
            Opcode::And => self.select_and(inst),
            Opcode::Or => self.select_or(inst),
            Opcode::Xor => self.select_xor(inst),
            Opcode::Shl => self.select_shl(inst),
            Opcode::LShr | Opcode::AShr => self.select_shr(inst),

            // Comparison
            Opcode::ICmp => self.select_icmp(inst),
            Opcode::FCmp => self.select_fcmp(inst),

            // Memory
            Opcode::Load => self.select_load(inst),
            Opcode::Store => self.select_store(inst),
            Opcode::Alloca => self.select_alloca(inst),

            // Control flow
            Opcode::Ret => self.select_ret(inst),
            Opcode::Br => self.select_br(inst),
            Opcode::CondBr => self.select_condbr(inst),
            Opcode::Call => self.select_call(inst),

            // Conversions
            Opcode::Trunc | Opcode::ZExt | Opcode::SExt => self.select_int_conversion(inst),
            Opcode::FPTrunc | Opcode::FPExt => self.select_fp_conversion(inst),
            Opcode::FPToUI | Opcode::FPToSI => self.select_fp_to_int(inst),
            Opcode::UIToFP | Opcode::SIToFP => self.select_int_to_fp(inst),
            Opcode::BitCast => self.select_bitcast(inst),

            // Other
            Opcode::Select => self.select_select(inst),
            Opcode::PHI => self.select_phi(inst),

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

    fn select_div(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Simplified division - uses idiv/div
        Ok(vec![MachineInstr::IDiv {
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_rem(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Remainder - result is in RDX after div
        Ok(vec![MachineInstr::IDiv {
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_and(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::And {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_or(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Or {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_xor(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Xor {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_shl(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Shl {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RCX),
        }])
    }

    fn select_shr(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![MachineInstr::Shr {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RCX),
        }])
    }

    fn select_icmp(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        Ok(vec![
            MachineInstr::Cmp {
                left: X86Register::RAX,
                right: MachineOperand::Register(X86Register::RBX),
            },
            MachineInstr::SetCC {
                condition: ConditionCode::Equal,
                dest: X86Register::AL,
            },
        ])
    }

    fn select_fcmp(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Floating point comparison - simplified
        Ok(vec![MachineInstr::Cmp {
            left: X86Register::RAX,
            right: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_alloca(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Allocate stack space
        Ok(vec![MachineInstr::Sub {
            dest: X86Register::RSP,
            src: MachineOperand::Immediate(8),
        }])
    }

    fn select_br(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Unconditional branch
        if let Some(target) = inst.operands().first() {
            if let Some(label) = target.name() {
                return Ok(vec![MachineInstr::Jmp(label.to_string())]);
            }
        }
        Err(CodegenError::InvalidOperand("Branch target missing".to_string()))
    }

    fn select_condbr(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Conditional branch - simplified
        let operands = inst.operands();
        if operands.len() >= 3 {
            if let (Some(true_label), Some(false_label)) = (operands[1].name(), operands[2].name()) {
                return Ok(vec![
                    MachineInstr::Test {
                        reg: X86Register::RAX,
                        src: MachineOperand::Register(X86Register::RAX),
                    },
                    MachineInstr::Je(false_label.to_string()),
                    MachineInstr::Jmp(true_label.to_string()),
                ]);
            }
        }
        Err(CodegenError::InvalidOperand("Conditional branch operands invalid".to_string()))
    }

    fn select_call(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Function call - simplified
        Ok(vec![MachineInstr::Call("function".to_string())])
    }

    fn select_int_conversion(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Integer conversions - often just moves or sign extensions
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_fp_conversion(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // FP conversions - simplified
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_fp_to_int(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // FP to int conversion
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_int_to_fp(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Int to FP conversion
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_bitcast(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Bitcast - often a no-op or simple move
        Ok(vec![MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        }])
    }

    fn select_select(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Select instruction (ternary operator)
        Ok(vec![
            MachineInstr::Test {
                reg: X86Register::RAX,
                src: MachineOperand::Register(X86Register::RAX),
            },
            MachineInstr::CMov {
                condition: ConditionCode::NotEqual,
                dest: X86Register::RAX,
                src: MachineOperand::Register(X86Register::RBX),
            },
        ])
    }

    fn select_phi(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Phi nodes are handled during register allocation/SSA destruction
        Ok(vec![])
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

/// Condition codes for conditional operations
#[derive(Debug, Clone, Copy)]
pub enum ConditionCode {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl std::fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionCode::Equal => write!(f, "e"),
            ConditionCode::NotEqual => write!(f, "ne"),
            ConditionCode::Less => write!(f, "l"),
            ConditionCode::LessEqual => write!(f, "le"),
            ConditionCode::Greater => write!(f, "g"),
            ConditionCode::GreaterEqual => write!(f, "ge"),
        }
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
    /// Division
    IDiv { src: MachineOperand },
    /// Bitwise AND
    And { dest: X86Register, src: MachineOperand },
    /// Bitwise OR
    Or { dest: X86Register, src: MachineOperand },
    /// Bitwise XOR
    Xor { dest: X86Register, src: MachineOperand },
    /// Shift left
    Shl { dest: X86Register, src: MachineOperand },
    /// Shift right
    Shr { dest: X86Register, src: MachineOperand },
    /// Compare
    Cmp { left: X86Register, right: MachineOperand },
    /// Test
    Test { reg: X86Register, src: MachineOperand },
    /// Set byte on condition
    SetCC { condition: ConditionCode, dest: X86Register },
    /// Conditional move
    CMov { condition: ConditionCode, dest: X86Register, src: MachineOperand },
    /// Jump
    Jmp(String),
    /// Conditional jump (equal)
    Je(String),
    /// Conditional jump (not equal)
    Jne(String),
    /// Call
    Call(String),
    /// Push
    Push(X86Register),
    /// Pop
    Pop(X86Register),
    /// Return
    Ret,
    /// Sign extend RAX to RDX:RAX (for division)
    Cqo,
    /// Move with zero extension
    Movzx { dest: X86Register, src: MachineOperand },
}

impl std::fmt::Display for MachineInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineInstr::Label(l) => write!(f, "{}:", l),
            MachineInstr::Mov { dest, src } => write!(f, "mov {}, {}", dest, src),
            MachineInstr::Add { dest, src } => write!(f, "add {}, {}", dest, src),
            MachineInstr::Sub { dest, src } => write!(f, "sub {}, {}", dest, src),
            MachineInstr::IMul { dest, src } => write!(f, "imul {}, {}", dest, src),
            MachineInstr::IDiv { src } => write!(f, "idiv {}", src),
            MachineInstr::And { dest, src } => write!(f, "and {}, {}", dest, src),
            MachineInstr::Or { dest, src } => write!(f, "or {}, {}", dest, src),
            MachineInstr::Xor { dest, src } => write!(f, "xor {}, {}", dest, src),
            MachineInstr::Shl { dest, src } => write!(f, "shl {}, {}", dest, src),
            MachineInstr::Shr { dest, src } => write!(f, "shr {}, {}", dest, src),
            MachineInstr::Cmp { left, right } => write!(f, "cmp {}, {}", left, right),
            MachineInstr::Test { reg, src } => write!(f, "test {}, {}", reg, src),
            MachineInstr::SetCC { condition, dest } => write!(f, "set{} {}", condition, dest),
            MachineInstr::CMov { condition, dest, src } => write!(f, "cmov{} {}, {}", condition, dest, src),
            MachineInstr::Jmp(label) => write!(f, "jmp {}", label),
            MachineInstr::Je(label) => write!(f, "je {}", label),
            MachineInstr::Jne(label) => write!(f, "jne {}", label),
            MachineInstr::Call(func) => write!(f, "call {}", func),
            MachineInstr::Push(reg) => write!(f, "push {}", reg),
            MachineInstr::Pop(reg) => write!(f, "pop {}", reg),
            MachineInstr::Ret => write!(f, "ret"),
            MachineInstr::Cqo => write!(f, "cqo"),
            MachineInstr::Movzx { dest, src } => write!(f, "movzx {}, {}", dest, src),
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
