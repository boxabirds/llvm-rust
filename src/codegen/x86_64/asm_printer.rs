//! Assembly Printer for x86-64
//!
//! Converts machine instructions to AT&T syntax assembly.

use super::MachineInstr;

pub struct AsmPrinter {
    output: String,
}

impl AsmPrinter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn emit_directive(&mut self, directive: &str) {
        self.output.push_str(&format!("\t{}\n", directive));
    }

    pub fn emit_label(&mut self, label: &str) {
        self.output.push_str(&format!("{}:\n", label));
    }

    pub fn emit_instruction(&mut self, instr: &MachineInstr) {
        self.output.push_str(&format!("\t{}\n", instr));
    }

    pub fn emit_comment(&mut self, comment: &str) {
        self.output.push_str(&format!("\t# {}\n", comment));
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }
}
