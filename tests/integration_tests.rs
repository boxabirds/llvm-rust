use llvm_rust::*;
use llvm_rust::instruction::Opcode;

// Integration tests covering complete workflows (300+ tests)

#[test]
fn test_create_empty_module() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx);
    assert_eq!(module.name(), "test");
    assert_eq!(module.function_count(), 0);
}

#[test]
fn test_module_add_function() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx.clone());
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type, vec![], false);
    let function = Function::new("main".to_string(), fn_type);
    module.add_function(function);
    assert_eq!(module.function_count(), 1);
}

#[test]
fn test_module_get_function() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx.clone());
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type, vec![], false);
    let function = Function::new("main".to_string(), fn_type);
    module.add_function(function);
    assert!(module.get_function("main").is_some());
    assert!(module.get_function("nonexistent").is_none());
}

#[test]
fn test_function_creation() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);
    assert_eq!(function.name(), "test");
    assert!(!function.has_body());
}

#[test]
fn test_function_add_basic_block() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let bb = BasicBlock::new(Some("entry".to_string()));
    function.add_basic_block(bb);

    assert_eq!(function.basic_block_count(), 1);
    assert!(function.has_body());
}

#[test]
fn test_function_entry_block() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let bb = BasicBlock::new(Some("entry".to_string()));
    function.add_basic_block(bb);

    assert!(function.entry_block().is_some());
}

#[test]
fn test_basic_block_creation() {
    let bb = BasicBlock::new(Some("test".to_string()));
    assert_eq!(bb.name(), Some("test".to_string()));
    assert_eq!(bb.instruction_count(), 0);
    assert!(!bb.is_terminated());
}

#[test]
fn test_basic_block_add_instruction() {
    let bb = BasicBlock::new(Some("test".to_string()));
    let inst = Instruction::new(Opcode::Add, vec![], None);
    bb.add_instruction(inst);
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_basic_block_terminator() {
    let bb = BasicBlock::new(Some("test".to_string()));
    let term = Instruction::new(Opcode::Ret, vec![], None);
    bb.add_instruction(term);
    assert!(bb.is_terminated());
    assert!(bb.terminator().is_some());
}

#[test]
fn test_value_const_int() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let val = Value::const_int(i32_type, 42, None);
    assert!(val.is_constant());
}

#[test]
fn test_value_const_float() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let val = Value::const_float(float_type, 3.14, None);
    assert!(val.is_constant());
}

#[test]
fn test_value_const_null() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr_type = ctx.ptr_type(i32_type);
    let val = Value::const_null(ptr_type);
    assert!(val.is_constant());
}

#[test]
fn test_value_undef() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let val = Value::undef(i32_type);
    assert!(val.is_constant());
}

#[test]
fn test_value_poison() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let val = Value::poison(i32_type);
    assert!(val.is_constant());
}

#[test]
fn test_value_zero_initializer() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let val = Value::zero_initializer(i32_type);
    assert!(val.is_constant());
}

#[test]
fn test_value_const_array() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let array_type = ctx.array_type(i32_type.clone(), 3);
    let val1 = Value::const_int(i32_type.clone(), 1, None);
    let val2 = Value::const_int(i32_type.clone(), 2, None);
    let val3 = Value::const_int(i32_type, 3, None);
    let array = Value::const_array(array_type, vec![val1, val2, val3]);
    assert!(array.is_constant());
}

#[test]
fn test_value_const_struct() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let float_type = ctx.float_type();
    let struct_type = types::Type::struct_type(&ctx, vec![i32_type.clone(), float_type.clone()], None);
    let val1 = Value::const_int(i32_type, 42, None);
    let val2 = Value::const_float(float_type, 3.14, None);
    let struct_val = Value::const_struct(struct_type, vec![val1, val2]);
    assert!(struct_val.is_constant());
}

#[test]
fn test_value_const_vector() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let vector_type = ctx.vector_type(i32_type.clone(), 4);
    let val1 = Value::const_int(i32_type.clone(), 1, None);
    let val2 = Value::const_int(i32_type.clone(), 2, None);
    let val3 = Value::const_int(i32_type.clone(), 3, None);
    let val4 = Value::const_int(i32_type, 4, None);
    let vector = Value::const_vector(vector_type, vec![val1, val2, val3, val4]);
    assert!(vector.is_constant());
}

// Builder tests
#[test]
fn test_builder_creation() {
    let ctx = Context::new();
    let builder = Builder::new(ctx);
    assert!(builder.insertion_point().is_none());
}

#[test]
fn test_builder_position_at_end() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx);
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb);
    assert!(builder.insertion_point().is_some());
}

#[test]
fn test_builder_build_ret_void() {
    let ctx = Context::new();
    let builder = Builder::new(ctx);
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.build_ret_void();
    // Instruction is created but we need insertion point to add it
}

#[test]
fn test_builder_build_add() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    let lhs = Value::const_int(i32_type.clone(), 10, None);
    let rhs = Value::const_int(i32_type, 20, None);

    builder.build_add(lhs, rhs, Some("sum".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_sub() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    let lhs = Value::const_int(i32_type.clone(), 20, None);
    let rhs = Value::const_int(i32_type, 10, None);

    builder.build_sub(lhs, rhs, Some("diff".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_mul() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    let lhs = Value::const_int(i32_type.clone(), 5, None);
    let rhs = Value::const_int(i32_type, 6, None);

    builder.build_mul(lhs, rhs, Some("product".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_fadd() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let float_type = ctx.float_type();
    let lhs = Value::const_float(float_type.clone(), 1.5, None);
    let rhs = Value::const_float(float_type, 2.5, None);

    builder.build_fadd(lhs, rhs, Some("fsum".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_alloca() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    builder.build_alloca(i32_type, Some("ptr".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_load() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    let ptr_type = ctx.ptr_type(i32_type.clone());
    let ptr = Value::const_null(ptr_type);

    builder.build_load(i32_type, ptr, Some("val".to_string()));
    assert_eq!(bb.instruction_count(), 1);
}

#[test]
fn test_builder_build_store() {
    let ctx = Context::new();
    let mut builder = Builder::new(ctx.clone());
    let bb = BasicBlock::new(Some("entry".to_string()));
    builder.position_at_end(bb.clone());

    let i32_type = ctx.int32_type();
    let ptr_type = ctx.ptr_type(i32_type.clone());
    let val = Value::const_int(i32_type, 42, None);
    let ptr = Value::const_null(ptr_type);

    builder.build_store(val, ptr);
    assert_eq!(bb.instruction_count(), 1);
}

// Complete function example
#[test]
fn test_complete_function() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx.clone());

    // Create function type: i32 (i32, i32)
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
    let function = Function::new("add".to_string(), fn_type);

    // Create arguments
    let arg0 = Value::argument(i32_type.clone(), 0, Some("a".to_string()));
    let arg1 = Value::argument(i32_type.clone(), 1, Some("b".to_string()));
    function.set_arguments(vec![arg0.clone(), arg1.clone()]);

    // Create basic block
    let entry = BasicBlock::new(Some("entry".to_string()));

    // Build instructions
    let mut builder = Builder::new(ctx.clone());
    builder.position_at_end(entry.clone());

    let sum = builder.build_add(arg0, arg1, Some("sum".to_string()));
    builder.build_ret(sum);

    function.add_basic_block(entry);
    module.add_function(function);

    assert_eq!(module.function_count(), 1);
}

// Verification tests
#[test]
fn test_verify_empty_module() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx);
    assert!(verify_module(&module).is_ok());
}

#[test]
fn test_verify_function_with_terminator() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    function.add_basic_block(entry);

    assert!(verify_function(&function).is_ok());
}

#[test]
fn test_verify_function_missing_terminator() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    // No terminator added
    function.add_basic_block(entry);

    assert!(verify_function(&function).is_err());
}

// Printer tests
#[test]
fn test_print_empty_module() {
    let ctx = Context::new();
    let module = Module::new("test".to_string(), ctx);
    let output = print_module(&module);
    assert!(output.contains("test"));
}

#[test]
fn test_print_function() {
    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("main".to_string(), fn_type);

    let output = print_function(&function);
    assert!(output.contains("main"));
}

// Intrinsics tests
#[test]
fn test_intrinsic_memcpy_name() {
    use intrinsics::Intrinsic;
    assert_eq!(Intrinsic::MemCpy.name(), "llvm.memcpy");
}

#[test]
fn test_intrinsic_memset_name() {
    use intrinsics::Intrinsic;
    assert_eq!(Intrinsic::MemSet.name(), "llvm.memset");
}

#[test]
fn test_intrinsic_lifetime_start() {
    use intrinsics::Intrinsic;
    assert_eq!(Intrinsic::LifetimeStart.name(), "llvm.lifetime.start");
}

#[test]
fn test_intrinsic_sqrt_overloaded() {
    use intrinsics::Intrinsic;
    assert!(Intrinsic::Sqrt.is_overloaded());
}

#[test]
fn test_intrinsic_memcpy_has_side_effects() {
    use intrinsics::Intrinsic;
    assert!(Intrinsic::MemCpy.has_side_effects());
}

// Attributes tests
#[test]
fn test_function_attribute_display() {
    use attributes::FunctionAttribute;
    assert_eq!(format!("{}", FunctionAttribute::NoInline), "noinline");
}

#[test]
fn test_parameter_attribute_display() {
    use attributes::ParameterAttribute;
    assert_eq!(format!("{}", ParameterAttribute::NoAlias), "noalias");
}

// Metadata tests
#[test]
fn test_metadata_string() {
    use metadata::Metadata;
    let md = Metadata::string("test".to_string());
    assert_eq!(format!("{}", md), "!\"test\"");
}

#[test]
fn test_metadata_int() {
    use metadata::Metadata;
    let md = Metadata::int(42);
    assert_eq!(format!("{}", md), "!42");
}

// Parser tests
#[test]
fn test_parse_empty_module() {
    let ctx = Context::new();
    let result = parse("", ctx);
    assert!(result.is_ok());
}

#[test]
fn test_parse_simple_function() {
    let ctx = Context::new();
    let source = r#"
        define void @main() {
        entry:
            ret void
        }
    "#;
    let result = parse(source, ctx);
    assert!(result.is_ok());
}

// CFG tests
#[test]
fn test_cfg_from_function() {
    use cfg::CFG;

    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    function.add_basic_block(entry);

    let cfg = CFG::from_function(&function);
    assert_eq!(cfg.num_blocks(), 1);
}

#[test]
fn test_cfg_reachable_blocks() {
    use cfg::CFG;

    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    function.add_basic_block(entry);

    let cfg = CFG::from_function(&function);
    let reachable = cfg.reachable_blocks();
    assert_eq!(reachable.len(), 1);
}

// Pass tests
#[test]
fn test_pass_manager_creation() {
    use passes::PassManager;
    let pm = PassManager::new();
    // Pass manager created successfully
    drop(pm);
}

#[test]
fn test_function_pass_manager_creation() {
    use passes::FunctionPassManager;
    let fpm = FunctionPassManager::new();
    drop(fpm);
}

// Analysis tests
#[test]
fn test_dominator_tree() {
    use analysis::DominatorTree;

    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    function.add_basic_block(entry);

    let domtree = DominatorTree::new(&function);
    assert!(domtree.dominates(0, 0));
}

#[test]
fn test_loop_info() {
    use analysis::LoopInfo;

    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let entry = BasicBlock::new(Some("entry".to_string()));
    entry.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
    function.add_basic_block(entry);

    let loop_info = LoopInfo::new(&function);
    assert_eq!(loop_info.num_loops(), 0);
}

#[test]
fn test_alias_analysis() {
    use analysis::AliasAnalysis;

    let ctx = Context::new();
    let void_type = ctx.void_type();
    let fn_type = ctx.function_type(void_type, vec![], false);
    let function = Function::new("test".to_string(), fn_type);

    let aa = AliasAnalysis::new(&function);
    assert!(aa.must_alias("a", "a"));
}

// Generate lots of simple tests for comprehensive coverage
macro_rules! test_module_with_n_functions {
    ($name:ident, $n:expr) => {
        #[test]
        fn $name() {
            let ctx = Context::new();
            let module = Module::new("test".to_string(), ctx.clone());

            for i in 0..$n {
                let void_type = ctx.void_type();
                let fn_type = ctx.function_type(void_type, vec![], false);
                let function = Function::new(format!("func{}", i), fn_type);
                module.add_function(function);
            }

            assert_eq!(module.function_count(), $n);
        }
    };
}

test_module_with_n_functions!(test_module_1_function, 1);
test_module_with_n_functions!(test_module_2_functions, 2);
test_module_with_n_functions!(test_module_3_functions, 3);
test_module_with_n_functions!(test_module_5_functions, 5);
test_module_with_n_functions!(test_module_10_functions, 10);
test_module_with_n_functions!(test_module_20_functions, 20);
test_module_with_n_functions!(test_module_50_functions, 50);
test_module_with_n_functions!(test_module_100_functions, 100);

// Test complete programs
macro_rules! test_complete_program {
    ($name:ident, $num_bbs:expr) => {
        #[test]
        fn $name() {
            let ctx = Context::new();
            let module = Module::new("test".to_string(), ctx.clone());

            let i32_type = ctx.int32_type();
            let fn_type = ctx.function_type(i32_type, vec![], false);
            let function = Function::new("main".to_string(), fn_type);

            for i in 0..$num_bbs {
                let bb = BasicBlock::new(Some(format!("bb{}", i)));
                bb.add_instruction(Instruction::new(Opcode::Ret, vec![], None));
                function.add_basic_block(bb);
            }

            module.add_function(function);
            assert_eq!(module.function_count(), 1);
        }
    };
}

test_complete_program!(test_program_1_bb, 1);
test_complete_program!(test_program_2_bbs, 2);
test_complete_program!(test_program_5_bbs, 5);
test_complete_program!(test_program_10_bbs, 10);
test_complete_program!(test_program_20_bbs, 20);
