//! IR Printer
//!
//! This module provides functionality to print LLVM IR in the standard format.

use std::fmt::Write as FmtWrite;
use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::Instruction;
use crate::value::Value;

/// IR printer
pub struct IRPrinter {
    indent_level: usize,
    output: String,
}

impl IRPrinter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    /// Print a module to a string
    pub fn print_module(&mut self, module: &Module) -> String {
        self.output.clear();
        self.indent_level = 0;

        // Print module header
        writeln!(self.output, "; ModuleID = '{}'", module.name()).unwrap();
        writeln!(self.output, "source_filename = \"{}\"", module.name()).unwrap();
        writeln!(self.output).unwrap();

        // Print global variables
        for global in module.globals() {
            self.print_global(&global);
        }

        if !module.globals().is_empty() {
            writeln!(self.output).unwrap();
        }

        // Print functions
        for (i, function) in module.functions().iter().enumerate() {
            if i > 0 {
                writeln!(self.output).unwrap();
            }
            self.print_function(function);
        }

        self.output.clone()
    }

    /// Print a global variable
    fn print_global(&mut self, global: &crate::module::GlobalVariable) {
        write!(self.output, "@{} = ", global.name()).unwrap();
        if global.is_constant() {
            write!(self.output, "constant ").unwrap();
        } else {
            write!(self.output, "global ").unwrap();
        }
        write!(self.output, "{}", global.get_type()).unwrap();
        if let Some(init) = global.initializer() {
            write!(self.output, " {}", init).unwrap();
        }
        writeln!(self.output).unwrap();
    }

    /// Print a function
    pub fn print_function(&mut self, function: &Function) -> String {
        // Print function declaration or definition
        write!(self.output, "define {} @{}(", function.get_type(), function.name()).unwrap();

        // Print arguments
        for (i, arg) in function.arguments().iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ").unwrap();
            }
            write!(self.output, "{} {}", arg.get_type(), arg).unwrap();
        }

        write!(self.output, ")").unwrap();

        // If function has no body, it's just a declaration
        if !function.has_body() {
            writeln!(self.output).unwrap();
            return self.output.clone();
        }

        // Print function body
        writeln!(self.output, " {{").unwrap();
        self.indent_level += 1;

        for bb in function.basic_blocks() {
            self.print_basic_block(&bb);
        }

        self.indent_level -= 1;
        writeln!(self.output, "}}").unwrap();

        self.output.clone()
    }

    /// Print a basic block
    pub fn print_basic_block(&mut self, bb: &BasicBlock) -> String {
        // Print block label
        if let Some(name) = bb.name() {
            writeln!(self.output, "{}:", name).unwrap();
        } else {
            writeln!(self.output, "bb:").unwrap();
        }

        // Print instructions
        for inst in bb.instructions() {
            self.print_instruction(&inst);
        }

        self.output.clone()
    }

    /// Print an instruction
    pub fn print_instruction(&mut self, inst: &Instruction) -> String {
        self.write_indent();

        // Print result if any
        if let Some(result) = inst.result() {
            write!(self.output, "{} = ", result).unwrap();
        }

        // Print opcode
        write!(self.output, "{:?}", inst.opcode()).unwrap();

        // Print operands
        if !inst.operands().is_empty() {
            write!(self.output, " ").unwrap();
            for (i, operand) in inst.operands().iter().enumerate() {
                if i > 0 {
                    write!(self.output, ", ").unwrap();
                }
                write!(self.output, "{} {}", operand.get_type(), operand).unwrap();
            }
        }

        writeln!(self.output).unwrap();

        self.output.clone()
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            write!(self.output, "  ").unwrap();
        }
    }
}

impl Default for IRPrinter {
    fn default() -> Self {
        Self::new()
    }
}

/// Print a module to a string
pub fn print_module(module: &Module) -> String {
    let mut printer = IRPrinter::new();
    printer.print_module(module)
}

/// Print a function to a string
pub fn print_function(function: &Function) -> String {
    let mut printer = IRPrinter::new();
    printer.print_function(function)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, BasicBlock};

    #[test]
    fn test_print_empty_module() {
        let ctx = Context::new();
        let module = Module::new("test".to_string(), ctx);
        let output = print_module(&module);
        assert!(output.contains("ModuleID = 'test'"));
    }

    #[test]
    fn test_print_function() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);
        let func = Function::new("main".to_string(), fn_type);

        let output = print_function(&func);
        assert!(output.contains("@main"));
    }
}
