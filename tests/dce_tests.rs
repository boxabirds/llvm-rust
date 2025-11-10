use llvm_rust::{Context, Function, BasicBlock, Instruction, Value};
use llvm_rust::instruction::Opcode;
use llvm_rust::transforms::DeadCodeEliminationPass;
use llvm_rust::passes::FunctionPass;

#[test]
fn test_remove_unused_add() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_dce_add".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    // Create dead instruction: add with no result
    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let five = Value::const_int(i32_type.clone(), 5, None);

    let dead_add = Instruction::new(Opcode::Add, vec![arg.clone(), five], None);
    entry.add_instruction(dead_add);

    // Return the argument
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 2 instructions before DCE
    assert_eq!(entry.instruction_count(), 2);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should remove the unused add
    assert!(changed, "DCE should have removed dead code");

    // Should have 1 instruction after DCE (just the return)
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_remove_multiple_dead_instructions() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_multi_dead".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let five = Value::const_int(i32_type.clone(), 5, None);
    let ten = Value::const_int(i32_type.clone(), 10, None);

    // Dead instruction 1
    let inst1 = Instruction::new(Opcode::Add, vec![arg.clone(), five], None);
    entry.add_instruction(inst1);

    // Dead instruction 2
    let inst2 = Instruction::new(Opcode::Mul, vec![arg.clone(), ten], None);
    entry.add_instruction(inst2);

    // Return argument
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 3 instructions before DCE
    assert_eq!(entry.instruction_count(), 3);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should remove both dead instructions
    assert!(changed, "DCE should have removed dead code");

    // Should have 1 instruction after DCE (just the return)
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_keep_store_instruction() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr_type = ctx.ptr_type(i32_type.clone());
    let fn_type = ctx.function_type(ctx.void_type(), vec![ptr_type.clone(), i32_type.clone()], false);
    let mut func = Function::new("test_store".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let ptr = Value::argument(ptr_type, 0, Some("ptr".to_string()));
    let value = Value::argument(i32_type, 1, Some("value".to_string()));

    // Store has side effects and should never be removed
    let store = Instruction::new(Opcode::Store, vec![value, ptr], None);
    entry.add_instruction(store);

    let ret = Instruction::new(Opcode::Ret, vec![], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 2 instructions before DCE
    assert_eq!(entry.instruction_count(), 2);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should NOT remove the store
    assert!(!changed, "DCE should not remove side-effecting instructions");

    // Should still have 2 instructions
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 2);
}

#[test]
fn test_no_dead_code() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_no_dead".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));

    // Just return the argument - no dead code
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 1 instruction before DCE
    assert_eq!(entry.instruction_count(), 1);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should not change anything
    assert!(!changed, "DCE should not change function without dead code");

    // Should still have 1 instruction
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_remove_dead_comparison() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_dead_cmp".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let zero = Value::const_int(i32_type.clone(), 0, None);

    // Dead comparison with no result
    let dead_cmp = Instruction::new(Opcode::ICmp, vec![arg.clone(), zero], None);
    entry.add_instruction(dead_cmp);

    // Return argument
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 2 instructions before DCE
    assert_eq!(entry.instruction_count(), 2);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should remove the unused comparison
    assert!(changed, "DCE should have removed dead comparison");

    // Should have 1 instruction after DCE (just the return)
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_remove_dead_bitwise() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_dead_and".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let mask = Value::const_int(i32_type.clone(), 0xFF, None);

    // Dead bitwise op with no result
    let dead_and = Instruction::new(Opcode::And, vec![arg.clone(), mask], None);
    entry.add_instruction(dead_and);

    // Return original argument
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 2 instructions before DCE
    assert_eq!(entry.instruction_count(), 2);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should remove the unused and operation
    assert!(changed, "DCE should have removed dead bitwise operation");

    // Should have 1 instruction after DCE (just the return)
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_remove_all_dead_arithmetic() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);
    let mut func = Function::new("test_all_dead".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let arg = Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let two = Value::const_int(i32_type.clone(), 2, None);
    let three = Value::const_int(i32_type.clone(), 3, None);

    // Several dead arithmetic operations
    entry.add_instruction(Instruction::new(Opcode::Add, vec![arg.clone(), two.clone()], None));
    entry.add_instruction(Instruction::new(Opcode::Sub, vec![arg.clone(), two.clone()], None));
    entry.add_instruction(Instruction::new(Opcode::Mul, vec![arg.clone(), three.clone()], None));
    entry.add_instruction(Instruction::new(Opcode::SDiv, vec![arg.clone(), two], None));

    // Return argument
    let ret = Instruction::new(Opcode::Ret, vec![arg], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 5 instructions before DCE
    assert_eq!(entry.instruction_count(), 5);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should remove all 4 dead arithmetic operations
    assert!(changed, "DCE should have removed dead code");

    // Should have 1 instruction after DCE (just the return)
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}

#[test]
fn test_terminators_not_removed() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type, vec![], false);
    let mut func = Function::new("test_terminator".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));

    let zero = Value::const_int(ctx.int32_type(), 0, None);

    // Terminator should never be removed even if result not "used"
    let ret = Instruction::new(Opcode::Ret, vec![zero], None);
    entry.add_instruction(ret);

    func.add_basic_block(entry.clone());

    // Should have 1 instruction before DCE
    assert_eq!(entry.instruction_count(), 1);

    let mut pass = DeadCodeEliminationPass;
    let changed = pass.run_on_function(&mut func).unwrap();

    // DCE should not remove the terminator
    assert!(!changed, "DCE should not remove terminators");

    // Should still have 1 instruction
    let blocks = func.basic_blocks();
    assert_eq!(blocks[0].instruction_count(), 1);
}
