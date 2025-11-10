use llvm_rust::{Context, Function, BasicBlock, Instruction, Value};
use llvm_rust::instruction::Opcode;
use llvm_rust::transforms::ConstantFoldingPass;
use llvm_rust::passes::FunctionPass;

#[test]
fn test_fold_add() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_add".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: add i32 2, 3  (should fold to 5)
    let const2 = Value::const_int(i32_type.clone(), 2, None);
    let const3 = Value::const_int(i32_type.clone(), 3, None);

    let add_inst = Instruction::new(Opcode::Add, vec![const2, const3], None);
    entry.add_instruction(add_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_sub() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_sub".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: sub i32 10, 3  (should fold to 7)
    let const10 = Value::const_int(i32_type.clone(), 10, None);
    let const3 = Value::const_int(i32_type.clone(), 3, None);

    let sub_inst = Instruction::new(Opcode::Sub, vec![const10, const3], None);
    entry.add_instruction(sub_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_mul() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_mul".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: mul i32 4, 5  (should fold to 20)
    let const4 = Value::const_int(i32_type.clone(), 4, None);
    let const5 = Value::const_int(i32_type.clone(), 5, None);

    let mul_inst = Instruction::new(Opcode::Mul, vec![const4, const5], None);
    entry.add_instruction(mul_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_div() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_div".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: sdiv i32 20, 4  (should fold to 5)
    let const20 = Value::const_int(i32_type.clone(), 20, None);
    let const4 = Value::const_int(i32_type.clone(), 4, None);

    let div_inst = Instruction::new(Opcode::SDiv, vec![const20, const4], None);
    entry.add_instruction(div_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_and() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_and".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: and i32 15, 7  (should fold to 7)
    let const15 = Value::const_int(i32_type.clone(), 15, None);
    let const7 = Value::const_int(i32_type.clone(), 7, None);

    let and_inst = Instruction::new(Opcode::And, vec![const15, const7], None);
    entry.add_instruction(and_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_or() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_or".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: or i32 8, 4  (should fold to 12)
    let const8 = Value::const_int(i32_type.clone(), 8, None);
    let const4 = Value::const_int(i32_type.clone(), 4, None);

    let or_inst = Instruction::new(Opcode::Or, vec![const8, const4], None);
    entry.add_instruction(or_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_xor() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_xor".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: xor i32 15, 7  (should fold to 8)
    let const15 = Value::const_int(i32_type.clone(), 15, None);
    let const7 = Value::const_int(i32_type.clone(), 7, None);

    let xor_inst = Instruction::new(Opcode::Xor, vec![const15, const7], None);
    entry.add_instruction(xor_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_shift_left() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_shl".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: shl i32 3, 2  (should fold to 12)
    let const3 = Value::const_int(i32_type.clone(), 3, None);
    let const2 = Value::const_int(i32_type.clone(), 2, None);

    let shl_inst = Instruction::new(Opcode::Shl, vec![const3, const2], None);
    entry.add_instruction(shl_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_fold_float_add() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let fn_type = ctx.function_type(float_type.clone(), vec![], false);
    let mut func = Function::new("test_fadd".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: fadd float 2.5, 3.5  (should fold to 6.0)
    let const2_5 = Value::const_float(float_type.clone(), 2.5, None);
    let const3_5 = Value::const_float(float_type.clone(), 3.5, None);

    let fadd_inst = Instruction::new(Opcode::FAdd, vec![const2_5, const3_5], None);
    entry.add_instruction(fadd_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(changed, "Constant folding should have made changes");
}

#[test]
fn test_no_fold_non_constants() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_no_fold".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: add i32 %arg0, 3  (cannot fold - arg0 is not constant)
    let arg0 = Value::argument(i32_type.clone(), 0, Some("arg0".to_string()));
    let const3 = Value::const_int(i32_type.clone(), 3, None);

    let add_inst = Instruction::new(Opcode::Add, vec![arg0, const3], None);
    entry.add_instruction(add_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(!changed, "Should not fold non-constant operations");
}

#[test]
fn test_division_by_zero_not_folded() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![], false);
    let mut func = Function::new("test_div_zero".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create: sdiv i32 10, 0  (should NOT fold - division by zero)
    let const10 = Value::const_int(i32_type.clone(), 10, None);
    let const0 = Value::const_int(i32_type.clone(), 0, None);

    let div_inst = Instruction::new(Opcode::SDiv, vec![const10, const0], None);
    entry.add_instruction(div_inst);
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    func.add_basic_block(entry);

    let mut pass = ConstantFoldingPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    assert!(!changed, "Should not fold division by zero");
}
