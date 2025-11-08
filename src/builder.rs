//! IR Builder
//!
//! The Builder provides a convenient API for constructing LLVM IR.
//! It maintains an insertion point and provides methods for creating
//! various instructions.

use crate::basic_block::BasicBlock;
use crate::value::Value;
use crate::instruction::{Instruction, Opcode, IntPredicate, FloatPredicate};
use crate::types::Type;
use crate::context::Context;

/// An IR builder for constructing LLVM instructions
pub struct Builder {
    context: Context,
    insertion_point: Option<BasicBlock>,
}

impl Builder {
    /// Create a new builder with the given context
    pub fn new(context: Context) -> Self {
        Self {
            context,
            insertion_point: None,
        }
    }

    /// Set the insertion point to the end of the given basic block
    pub fn position_at_end(&mut self, bb: BasicBlock) {
        self.insertion_point = Some(bb);
    }

    /// Get the current insertion point
    pub fn insertion_point(&self) -> Option<&BasicBlock> {
        self.insertion_point.as_ref()
    }

    /// Insert an instruction at the current insertion point
    fn insert(&self, inst: Instruction) -> Value {
        if let Some(bb) = &self.insertion_point {
            bb.add_instruction(inst.clone());
        }

        // Return a value representing the instruction result
        if let Some(result) = inst.result() {
            result.clone()
        } else {
            // For instructions without results, return undef
            Value::undef(self.context.void_type())
        }
    }

    // Terminator instructions

    /// Create a return instruction with no value (void return)
    pub fn build_ret_void(&self) -> Value {
        let inst = Instruction::new(Opcode::Ret, vec![], None);
        self.insert(inst)
    }

    /// Create a return instruction with a value
    pub fn build_ret(&self, value: Value) -> Value {
        let inst = Instruction::new(Opcode::Ret, vec![value], None);
        self.insert(inst)
    }

    /// Create an unconditional branch
    pub fn build_br(&self, _dest: BasicBlock) {
        let inst = Instruction::new(Opcode::Br, vec![], None);
        self.insert(inst);
    }

    /// Create a conditional branch
    pub fn build_cond_br(&self, cond: Value, _then_bb: BasicBlock, _else_bb: BasicBlock) {
        let inst = Instruction::new(Opcode::CondBr, vec![cond], None);
        self.insert(inst);
    }

    // Binary operations

    /// Create an integer addition
    pub fn build_add(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Add, name);
        let inst = Instruction::new(Opcode::Add, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create an integer subtraction
    pub fn build_sub(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Sub, name);
        let inst = Instruction::new(Opcode::Sub, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create an integer multiplication
    pub fn build_mul(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Mul, name);
        let inst = Instruction::new(Opcode::Mul, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create an unsigned integer division
    pub fn build_udiv(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::UDiv, name);
        let inst = Instruction::new(Opcode::UDiv, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a signed integer division
    pub fn build_sdiv(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::SDiv, name);
        let inst = Instruction::new(Opcode::SDiv, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    // Floating point operations

    /// Create a floating point addition
    pub fn build_fadd(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::FAdd, name);
        let inst = Instruction::new(Opcode::FAdd, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a floating point subtraction
    pub fn build_fsub(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::FSub, name);
        let inst = Instruction::new(Opcode::FSub, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a floating point multiplication
    pub fn build_fmul(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::FMul, name);
        let inst = Instruction::new(Opcode::FMul, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a floating point division
    pub fn build_fdiv(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::FDiv, name);
        let inst = Instruction::new(Opcode::FDiv, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    // Bitwise operations

    /// Create a bitwise AND
    pub fn build_and(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::And, name);
        let inst = Instruction::new(Opcode::And, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a bitwise OR
    pub fn build_or(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Or, name);
        let inst = Instruction::new(Opcode::Or, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a bitwise XOR
    pub fn build_xor(&self, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = lhs.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Xor, name);
        let inst = Instruction::new(Opcode::Xor, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    // Memory operations

    /// Create an alloca instruction (stack allocation)
    pub fn build_alloca(&self, ty: Type, name: Option<String>) -> Value {
        let ptr_type = self.context.ptr_type(ty);
        let result = Value::instruction(ptr_type, Opcode::Alloca, name);
        let inst = Instruction::new(Opcode::Alloca, vec![], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a load instruction
    pub fn build_load(&self, ty: Type, ptr: Value, name: Option<String>) -> Value {
        let result = Value::instruction(ty, Opcode::Load, name);
        let inst = Instruction::new(Opcode::Load, vec![ptr], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a store instruction
    pub fn build_store(&self, value: Value, ptr: Value) {
        let inst = Instruction::new(Opcode::Store, vec![value, ptr], None);
        self.insert(inst);
    }

    // Comparison operations

    /// Create an integer comparison
    pub fn build_icmp(&self, _pred: IntPredicate, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = self.context.bool_type();
        let result = Value::instruction(result_type, Opcode::ICmp, name);
        let inst = Instruction::new(Opcode::ICmp, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a floating point comparison
    pub fn build_fcmp(&self, _pred: FloatPredicate, lhs: Value, rhs: Value, name: Option<String>) -> Value {
        let result_type = self.context.bool_type();
        let result = Value::instruction(result_type, Opcode::FCmp, name);
        let inst = Instruction::new(Opcode::FCmp, vec![lhs, rhs], Some(result.clone()));
        self.insert(inst);
        result
    }

    // Conversion operations

    /// Create a zero extension (unsigned extension)
    pub fn build_zext(&self, value: Value, dest_ty: Type, name: Option<String>) -> Value {
        let result = Value::instruction(dest_ty, Opcode::ZExt, name);
        let inst = Instruction::new(Opcode::ZExt, vec![value], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a sign extension (signed extension)
    pub fn build_sext(&self, value: Value, dest_ty: Type, name: Option<String>) -> Value {
        let result = Value::instruction(dest_ty, Opcode::SExt, name);
        let inst = Instruction::new(Opcode::SExt, vec![value], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a truncation
    pub fn build_trunc(&self, value: Value, dest_ty: Type, name: Option<String>) -> Value {
        let result = Value::instruction(dest_ty, Opcode::Trunc, name);
        let inst = Instruction::new(Opcode::Trunc, vec![value], Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a bitcast
    pub fn build_bitcast(&self, value: Value, dest_ty: Type, name: Option<String>) -> Value {
        let result = Value::instruction(dest_ty, Opcode::BitCast, name);
        let inst = Instruction::new(Opcode::BitCast, vec![value], Some(result.clone()));
        self.insert(inst);
        result
    }

    // Other operations

    /// Create a function call
    pub fn build_call(&self, _func_ty: Type, func: Value, args: Vec<Value>, name: Option<String>) -> Value {
        // Determine return type from function type
        let return_type = self.context.void_type(); // Simplified
        let result = Value::instruction(return_type, Opcode::Call, name);
        let mut operands = vec![func];
        operands.extend(args);
        let inst = Instruction::new(Opcode::Call, operands, Some(result.clone()));
        self.insert(inst);
        result
    }

    /// Create a select (ternary conditional)
    pub fn build_select(&self, cond: Value, then_val: Value, else_val: Value, name: Option<String>) -> Value {
        let result_type = then_val.get_type().clone();
        let result = Value::instruction(result_type, Opcode::Select, name);
        let inst = Instruction::new(Opcode::Select, vec![cond, then_val, else_val], Some(result.clone()));
        self.insert(inst);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let ctx = Context::new();
        let builder = Builder::new(ctx);
        assert!(builder.insertion_point().is_none());
    }

    #[test]
    fn test_position_at_end() {
        let ctx = Context::new();
        let mut builder = Builder::new(ctx);
        let bb = BasicBlock::new(Some("entry".to_string()));

        builder.position_at_end(bb);
        assert!(builder.insertion_point().is_some());
    }

    #[test]
    fn test_build_add() {
        let ctx = Context::new();
        let mut builder = Builder::new(ctx.clone());
        let bb = BasicBlock::new(Some("entry".to_string()));
        builder.position_at_end(bb.clone());

        let i32_type = ctx.int32_type();
        let lhs = Value::const_int(i32_type.clone(), 10, None);
        let rhs = Value::const_int(i32_type, 20, None);

        let _result = builder.build_add(lhs, rhs, Some("sum".to_string()));
        assert_eq!(bb.instruction_count(), 1);
    }
}
