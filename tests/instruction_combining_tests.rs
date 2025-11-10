use llvm_rust::{Context, Function, BasicBlock, Instruction, Value};
use llvm_rust::instruction::Opcode;
use llvm_rust::transforms::InstructionCombiningPass;
use llvm_rust::passes::FunctionPass;

// Identity operations tests

#[test]
fn test_add_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_add_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x + 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let add_inst = Instruction::new(Opcode::Add, vec![arg, zero], None);
    entry.add_instruction(add_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x + 0 should be simplified to x");
}

#[test]
fn test_sub_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_sub_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x - 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let sub_inst = Instruction::new(Opcode::Sub, vec![arg, zero], None);
    entry.add_instruction(sub_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x - 0 should be simplified to x");
}

#[test]
fn test_mul_one_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_mul_one".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x * 1 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let one = Value::const_int(i32_type.clone(), 1, None);

    let mul_inst = Instruction::new(Opcode::Mul, vec![arg, one], None);
    entry.add_instruction(mul_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x * 1 should be simplified to x");
}

#[test]
fn test_div_one_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_div_one".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x / 1 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let one = Value::const_int(i32_type.clone(), 1, None);

    let div_inst = Instruction::new(Opcode::SDiv, vec![arg, one], None);
    entry.add_instruction(div_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x / 1 should be simplified to x");
}

// Annihilation operations tests

#[test]
fn test_mul_zero_annihilation() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_mul_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x * 0 = 0
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let mul_inst = Instruction::new(Opcode::Mul, vec![arg, zero], None);
    entry.add_instruction(mul_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x * 0 should be simplified to 0");
}

#[test]
fn test_and_zero_annihilation() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_and_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x & 0 = 0
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let and_inst = Instruction::new(Opcode::And, vec![arg, zero], None);
    entry.add_instruction(and_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x & 0 should be simplified to 0");
}

#[test]
fn test_or_all_ones_annihilation() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_or_all_ones".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x | -1 = -1
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let all_ones = Value::const_int(i32_type.clone(), -1, None);

    let or_inst = Instruction::new(Opcode::Or, vec![arg, all_ones], None);
    entry.add_instruction(or_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x | -1 should be simplified to -1");
}

// Bitwise identity operations

#[test]
fn test_and_all_ones_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_and_all_ones".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x & -1 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let all_ones = Value::const_int(i32_type.clone(), -1, None);

    let and_inst = Instruction::new(Opcode::And, vec![arg, all_ones], None);
    entry.add_instruction(and_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x & -1 should be simplified to x");
}

#[test]
fn test_or_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_or_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x | 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let or_inst = Instruction::new(Opcode::Or, vec![arg, zero], None);
    entry.add_instruction(or_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x | 0 should be simplified to x");
}

#[test]
fn test_xor_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_xor_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x ^ 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let xor_inst = Instruction::new(Opcode::Xor, vec![arg, zero], None);
    entry.add_instruction(xor_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x ^ 0 should be simplified to x");
}

// Shift identity operations

#[test]
fn test_shl_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_shl_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x << 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let shl_inst = Instruction::new(Opcode::Shl, vec![arg, zero], None);
    entry.add_instruction(shl_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x << 0 should be simplified to x");
}

#[test]
fn test_lshr_zero_identity() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_lshr_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x >> 0 = x
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    let lshr_inst = Instruction::new(Opcode::LShr, vec![arg, zero], None);
    entry.add_instruction(lshr_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x >> 0 should be simplified to x");
}

// Shift annihilation (0 shifted is still 0)

#[test]
fn test_zero_shl_annihilation() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_zero_shl".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: 0 << x = 0
    let zero = Value::const_int(i32_type.clone(), 0, None);
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));

    let shl_inst = Instruction::new(Opcode::Shl, vec![zero, arg], None);
    entry.add_instruction(shl_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "0 << x should be simplified to 0");
}

// Remainder with 1

#[test]
fn test_rem_one_simplification() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_rem_one".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x % 1 = 0
    let arg = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let one = Value::const_int(i32_type.clone(), 1, None);

    let rem_inst = Instruction::new(Opcode::SRem, vec![arg, one], None);
    entry.add_instruction(rem_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x % 1 should be simplified to 0");
}

// Floating point operations

#[test]
fn test_fadd_zero_identity() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let fn_type = ctx.function_type(float_type.clone(), vec![float_type.clone()], false);
    let mut func = Function::new("test_fadd_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x + 0.0 = x
    let arg = Value::argument(float_type.clone(), 0, Some("x".to_string()));
    let zero = Value::const_float(float_type.clone(), 0.0, None);

    let fadd_inst = Instruction::new(Opcode::FAdd, vec![arg, zero], None);
    entry.add_instruction(fadd_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x + 0.0 should be simplified to x");
}

#[test]
fn test_fmul_one_identity() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let fn_type = ctx.function_type(float_type.clone(), vec![float_type.clone()], false);
    let mut func = Function::new("test_fmul_one".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x * 1.0 = x
    let arg = Value::argument(float_type.clone(), 0, Some("x".to_string()));
    let one = Value::const_float(float_type.clone(), 1.0, None);

    let fmul_inst = Instruction::new(Opcode::FMul, vec![arg, one], None);
    entry.add_instruction(fmul_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "x * 1.0 should be simplified to x");
}

// Test that non-simplifiable operations are not changed

#[test]
fn test_no_simplification() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
    let mut func = Function::new("test_no_simplify".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: x + y (cannot simplify)
    let arg1 = Value::argument(i32_type.clone(), 0, Some("x".to_string()));
    let arg2 = Value::argument(i32_type.clone(), 1, Some("y".to_string()));

    let add_inst = Instruction::new(Opcode::Add, vec![arg1, arg2], None);
    entry.add_instruction(add_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = InstructionCombiningPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(!changed, "x + y should not be simplified");
}
