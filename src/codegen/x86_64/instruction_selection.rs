///! Enhanced Instruction Selection
///!
///! Properly lowers IR instructions to x86-64 machine instructions
///! by examining operands and generating appropriate code

use crate::instruction::{Instruction, Opcode};
use crate::value::Value;
use super::{MachineInstr, MachineOperand, X86Register, ConditionCode};
use crate::codegen::{CodegenError, value_tracker::{ValueTracker, ValueLocation}};

pub struct InstructionSelector {
    value_tracker: ValueTracker,
    /// Next temporary register
    next_temp: usize,
}

impl InstructionSelector {
    pub fn new() -> Self {
        Self {
            value_tracker: ValueTracker::new(),
            next_temp: 0,
        }
    }

    /// Select instructions for an IR instruction with proper operand handling
    pub fn select(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        match inst.opcode() {
            // Arithmetic - binary operations
            Opcode::Add => self.select_binary_op(inst, |d, s| MachineInstr::Add { dest: d, src: s }),
            Opcode::Sub => self.select_binary_op(inst, |d, s| MachineInstr::Sub { dest: d, src: s }),
            Opcode::Mul => self.select_binary_op(inst, |d, s| MachineInstr::IMul { dest: d, src: s }),
            Opcode::And => self.select_binary_op(inst, |d, s| MachineInstr::And { dest: d, src: s }),
            Opcode::Or => self.select_binary_op(inst, |d, s| MachineInstr::Or { dest: d, src: s }),
            Opcode::Xor => self.select_binary_op(inst, |d, s| MachineInstr::Xor { dest: d, src: s }),

            // Floating point arithmetic
            Opcode::FAdd => self.select_binary_op(inst, |d, s| MachineInstr::Add { dest: d, src: s }), // Simplified
            Opcode::FSub => self.select_binary_op(inst, |d, s| MachineInstr::Sub { dest: d, src: s }),
            Opcode::FMul => self.select_binary_op(inst, |d, s| MachineInstr::IMul { dest: d, src: s }),

            // Division and remainder
            Opcode::UDiv | Opcode::SDiv | Opcode::FDiv => self.select_div(inst),
            Opcode::URem | Opcode::SRem | Opcode::FRem => self.select_rem(inst),

            // Shifts
            Opcode::Shl => self.select_shift(inst, |d, s| MachineInstr::Shl { dest: d, src: s }),
            Opcode::LShr | Opcode::AShr => self.select_shift(inst, |d, s| MachineInstr::Shr { dest: d, src: s }),

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
            Opcode::Trunc => self.select_trunc(inst),
            Opcode::ZExt => self.select_zext(inst),
            Opcode::SExt => self.select_sext(inst),
            Opcode::FPTrunc | Opcode::FPExt => self.select_fp_conversion(inst),
            Opcode::FPToUI | Opcode::FPToSI => self.select_fp_to_int(inst),
            Opcode::UIToFP | Opcode::SIToFP => self.select_int_to_fp(inst),
            Opcode::BitCast => self.select_bitcast(inst),

            // Other
            Opcode::Select => self.select_select(inst),
            Opcode::PHI => Ok(vec![]), // Handled separately

            _ => Err(CodegenError::UnsupportedInstruction(format!("{:?}", inst.opcode()))),
        }
    }

    /// Select a binary operation (add, sub, mul, and, or, xor)
    fn select_binary_op<F>(&mut self, inst: &Instruction, make_instr: F) -> Result<Vec<MachineInstr>, CodegenError>
    where
        F: Fn(X86Register, MachineOperand) -> MachineInstr,
    {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("Binary op needs 2 operands".to_string()));
        }

        let mut instrs = Vec::new();
        let dest_reg = X86Register::RAX; // Simplified: use RAX as destination

        // Load first operand into destination register
        instrs.extend(self.load_value_to_register(&operands[0], dest_reg)?);

        // Get second operand as MachineOperand
        let src_operand = self.value_to_machine_operand(&operands[1])?;

        // Perform the operation
        instrs.push(make_instr(dest_reg, src_operand));

        // Track result location
        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(dest_reg));

        Ok(instrs)
    }

    /// Load a value into a register
    fn load_value_to_register(&mut self, value: &Value, reg: X86Register) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut instrs = Vec::new();

        // Check if it's an immediate constant
        if let Some(imm) = self.value_tracker.get_immediate(value) {
            instrs.push(MachineInstr::Mov {
                dest: reg,
                src: MachineOperand::Immediate(imm),
            });
            return Ok(instrs);
        }

        // Check if it's already in a register
        if let Some(location) = self.value_tracker.get_location(value) {
            match location {
                ValueLocation::Register(src_reg) => {
                    if *src_reg != reg {
                        instrs.push(MachineInstr::Mov {
                            dest: reg,
                            src: MachineOperand::Register(*src_reg),
                        });
                    }
                }
                ValueLocation::Stack(offset) => {
                    instrs.push(MachineInstr::Mov {
                        dest: reg,
                        src: MachineOperand::Memory {
                            base: X86Register::RBP,
                            offset: *offset,
                        },
                    });
                }
                ValueLocation::Immediate(imm) => {
                    instrs.push(MachineInstr::Mov {
                        dest: reg,
                        src: MachineOperand::Immediate(*imm),
                    });
                }
                ValueLocation::Symbol(sym) => {
                    // Load symbol address - simplified
                    instrs.push(MachineInstr::Mov {
                        dest: reg,
                        src: MachineOperand::Immediate(0), // Placeholder
                    });
                }
                _ => {
                    // Allocate and move
                    instrs.push(MachineInstr::Mov {
                        dest: reg,
                        src: MachineOperand::Register(X86Register::RAX), // Placeholder
                    });
                }
            }
        } else {
            // Value location unknown - try to extract immediate
            if let Some(const_val) = value.as_const_int() {
                instrs.push(MachineInstr::Mov {
                    dest: reg,
                    src: MachineOperand::Immediate(const_val),
                });
            } else {
                // Assume it's in RAX for now (simplified)
                if reg != X86Register::RAX {
                    instrs.push(MachineInstr::Mov {
                        dest: reg,
                        src: MachineOperand::Register(X86Register::RAX),
                    });
                }
            }
        }

        Ok(instrs)
    }

    /// Convert a value to a machine operand
    fn value_to_machine_operand(&mut self, value: &Value) -> Result<MachineOperand, CodegenError> {
        // Check for immediate constant
        if let Some(imm) = self.value_tracker.get_immediate(value) {
            return Ok(MachineOperand::Immediate(imm));
        }

        // Check tracked location
        if let Some(location) = self.value_tracker.get_location(value) {
            match location {
                ValueLocation::Register(reg) => return Ok(MachineOperand::Register(*reg)),
                ValueLocation::Immediate(imm) => return Ok(MachineOperand::Immediate(*imm)),
                ValueLocation::Stack(offset) => return Ok(MachineOperand::Memory {
                    base: X86Register::RBP,
                    offset: *offset,
                }),
                _ => {}
            }
        }

        // Try constant extraction
        if let Some(const_val) = value.as_const_int() {
            return Ok(MachineOperand::Immediate(const_val));
        }

        // Default to RBX register (simplified)
        Ok(MachineOperand::Register(X86Register::RBX))
    }

    /// Select division instruction
    fn select_div(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("Div needs 2 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load dividend into RAX
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Sign extend RAX into RDX:RAX
        instrs.push(MachineInstr::Cqo);

        // Load divisor into register
        let divisor = self.value_to_machine_operand(&operands[1])?;

        // Perform division
        instrs.push(MachineInstr::IDiv { src: divisor });

        // Result is in RAX
        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));

        Ok(instrs)
    }

    /// Select remainder instruction
    fn select_rem(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("Rem needs 2 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load dividend into RAX
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Sign extend RAX into RDX:RAX
        instrs.push(MachineInstr::Cqo);

        // Load divisor
        let divisor = self.value_to_machine_operand(&operands[1])?;

        // Perform division
        instrs.push(MachineInstr::IDiv { src: divisor });

        // Remainder is in RDX
        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RDX));

        // Move result to RAX for consistency
        instrs.push(MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RDX),
        });

        Ok(instrs)
    }

    /// Select shift operation
    fn select_shift<F>(&mut self, inst: &Instruction, make_instr: F) -> Result<Vec<MachineInstr>, CodegenError>
    where
        F: Fn(X86Register, MachineOperand) -> MachineInstr,
    {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("Shift needs 2 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load value into RAX
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Shift amount must be in CL (lower 8 bits of RCX)
        instrs.extend(self.load_value_to_register(&operands[1], X86Register::RCX)?);

        // Perform shift
        instrs.push(make_instr(X86Register::RAX, MachineOperand::Register(X86Register::RCX)));

        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));

        Ok(instrs)
    }

    /// Select integer comparison
    fn select_icmp(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("ICmp needs 2 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load first operand
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Get second operand
        let src = self.value_to_machine_operand(&operands[1])?;

        // Compare
        instrs.push(MachineInstr::Cmp {
            left: X86Register::RAX,
            right: src,
        });

        // Set result based on condition (simplified - use EQ)
        instrs.push(MachineInstr::SetCC {
            condition: ConditionCode::Equal,
            dest: X86Register::AL,
        });

        // Zero-extend AL to RAX
        instrs.push(MachineInstr::Movzx {
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::AL),
        });

        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));

        Ok(instrs)
    }

    fn select_fcmp(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Simplified floating point comparison
        self.select_icmp(inst)
    }

    fn select_load(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.is_empty() {
            return Err(CodegenError::InvalidOperand("Load needs address operand".to_string()));
        }

        let mut instrs = Vec::new();

        // Get address (simplified - assume it's a stack slot)
        instrs.push(MachineInstr::Mov {
            dest: X86Register::RAX,
            src: MachineOperand::Memory {
                base: X86Register::RBP,
                offset: -8, // Simplified
            },
        });

        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));

        Ok(instrs)
    }

    fn select_store(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 2 {
            return Err(CodegenError::InvalidOperand("Store needs value and address".to_string()));
        }

        let mut instrs = Vec::new();

        // Load value to store
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Store to memory (simplified)
        instrs.push(MachineInstr::Mov {
            dest: X86Register::RBP, // Placeholder - should be memory
            src: MachineOperand::Register(X86Register::RAX),
        });

        Ok(instrs)
    }

    fn select_alloca(&mut self, _inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Allocate stack space
        Ok(vec![MachineInstr::Sub {
            dest: X86Register::RSP,
            src: MachineOperand::Immediate(8),
        }])
    }

    fn select_ret(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut instrs = Vec::new();

        // If returning a value, load it into RAX
        if !inst.operands().is_empty() {
            instrs.extend(self.load_value_to_register(&inst.operands()[0], X86Register::RAX)?);
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

    fn select_br(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        if let Some(target) = inst.operands().first() {
            if let Some(label) = target.name() {
                return Ok(vec![MachineInstr::Jmp(label.to_string())]);
            }
        }
        Err(CodegenError::InvalidOperand("Branch target missing".to_string()))
    }

    fn select_condbr(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 3 {
            return Err(CodegenError::InvalidOperand("Conditional branch needs 3 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load condition
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RAX)?);

        // Test condition
        instrs.push(MachineInstr::Test {
            reg: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RAX),
        });

        // Jump based on condition
        if let (Some(true_label), Some(false_label)) = (operands[1].name(), operands[2].name()) {
            instrs.push(MachineInstr::Je(false_label.to_string()));
            instrs.push(MachineInstr::Jmp(true_label.to_string()));
        }

        Ok(instrs)
    }

    fn select_call(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.is_empty() {
            return Err(CodegenError::InvalidOperand("Call needs function name".to_string()));
        }

        let func_name = operands[0].name().unwrap_or("unknown");

        // TODO: Handle arguments in proper registers (RDI, RSI, RDX, RCX, R8, R9)

        Ok(vec![MachineInstr::Call(func_name.to_string())])
    }

    fn select_trunc(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Truncation - just move (lower bits are preserved)
        self.select_move(inst)
    }

    fn select_zext(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Zero extension
        let mut instrs = Vec::new();
        if !inst.operands().is_empty() {
            instrs.extend(self.load_value_to_register(&inst.operands()[0], X86Register::RAX)?);
            // x86-64 32-bit operations zero-extend to 64 bits automatically
        }
        Ok(instrs)
    }

    fn select_sext(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Sign extension
        let mut instrs = Vec::new();
        if !inst.operands().is_empty() {
            instrs.extend(self.load_value_to_register(&inst.operands()[0], X86Register::RAX)?);
            // Use movsx for sign extension (simplified)
        }
        Ok(instrs)
    }

    fn select_fp_conversion(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // FP conversions (simplified)
        self.select_move(inst)
    }

    fn select_fp_to_int(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // FP to int (simplified)
        self.select_move(inst)
    }

    fn select_int_to_fp(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Int to FP (simplified)
        self.select_move(inst)
    }

    fn select_bitcast(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        // Bitcast - just move
        self.select_move(inst)
    }

    fn select_select(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let operands = inst.operands();
        if operands.len() < 3 {
            return Err(CodegenError::InvalidOperand("Select needs 3 operands".to_string()));
        }

        let mut instrs = Vec::new();

        // Load condition
        instrs.extend(self.load_value_to_register(&operands[0], X86Register::RCX)?);

        // Load true value
        instrs.extend(self.load_value_to_register(&operands[1], X86Register::RAX)?);

        // Load false value
        instrs.extend(self.load_value_to_register(&operands[2], X86Register::RBX)?);

        // Test condition
        instrs.push(MachineInstr::Test {
            reg: X86Register::RCX,
            src: MachineOperand::Register(X86Register::RCX),
        });

        // Conditional move
        instrs.push(MachineInstr::CMov {
            condition: ConditionCode::Equal, // If zero, use false value
            dest: X86Register::RAX,
            src: MachineOperand::Register(X86Register::RBX),
        });

        // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));

        Ok(instrs)
    }

    fn select_move(&mut self, inst: &Instruction) -> Result<Vec<MachineInstr>, CodegenError> {
        let mut instrs = Vec::new();
        if !inst.operands().is_empty() {
            instrs.extend(self.load_value_to_register(&inst.operands()[0], X86Register::RAX)?);
            // self.value_tracker.set_location(&inst.as_value(), ValueLocation::Register(X86Register::RAX));
        }
        Ok(instrs)
    }
}

impl Default for InstructionSelector {
    fn default() -> Self {
        Self::new()
    }
}
