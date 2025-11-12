use llvm_rust::{Context, Module, Function, BasicBlock, Builder, codegen};
use llvm_rust::codegen::x86_64::X86_64TargetMachine;
use llvm_rust::codegen::{TargetMachine, elf, external_functions};

#[test]
fn test_x86_64_assembly_generation() {
    let ctx = Context::new();
    let module = Module::new("test_module".to_string(), ctx.clone());

    // Create a simple function: i32 add(i32, i32)
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);

    let function = Function::new("add".to_string(), fn_type);
    module.add_function(function);

    // Generate assembly
    let mut target = X86_64TargetMachine::new();
    let asm = target.emit_assembly(&module);

    assert!(asm.is_ok());
    let asm_code = asm.unwrap();

    // Check for expected assembly directives
    assert!(asm_code.contains(".text"));
    assert!(asm_code.contains(".globl add"));
    assert!(asm_code.contains("add:"));
}

#[test]
fn test_elf_object_file_creation() {
    let mut obj = elf::ElfObjectFile::new();

    // Add a text section with some machine code
    obj.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: vec![0x55, 0x48, 0x89, 0xe5], // push rbp; mov rbp, rsp
        alignment: 16,
    });

    // Add a symbol
    obj.add_symbol(elf::Symbol {
        name: "main".to_string(),
        value: 0,
        size: 4,
        binding: elf::SymbolBinding::Global,
        typ: elf::SymbolType::Function,
        section_index: 1,
    });

    // Generate ELF file
    let elf_data = obj.generate();

    // Check ELF magic number
    assert_eq!(&elf_data[0..4], &[0x7f, b'E', b'L', b'F']);

    // Check 64-bit format
    assert_eq!(elf_data[4], 2);

    // Check little endian
    assert_eq!(elf_data[5], 1);
}

#[test]
fn test_external_function_registry() {
    let registry = external_functions::ExternalFunctionRegistry::new_with_stdlib();

    // Test common libc functions are registered
    assert!(registry.is_external("printf"));
    assert!(registry.is_external("malloc"));
    assert!(registry.is_external("free"));
    assert!(registry.is_external("strlen"));
    assert!(registry.is_external("memcpy"));

    // Test function details
    let printf = registry.get("printf").unwrap();
    assert_eq!(printf.name, "printf");
    assert!(printf.is_variadic);
    assert_eq!(printf.return_type, external_functions::ExternalType::I32);

    let malloc = registry.get("malloc").unwrap();
    assert_eq!(malloc.name, "malloc");
    assert!(!malloc.is_variadic);
    assert_eq!(malloc.return_type, external_functions::ExternalType::Ptr);
    assert_eq!(malloc.param_types.len(), 1);
}

#[test]
fn test_ffi_call_generation() {
    let helper = external_functions::FFIHelper::new();

    // Test generating call to puts
    let result = helper.generate_call("puts", &["rax".to_string()]);
    assert!(result.is_ok());
    let asm = result.unwrap();
    assert!(asm.contains("mov rdi"));
    assert!(asm.contains("call puts@PLT"));

    // Test invalid function
    let result = helper.generate_call("nonexistent", &[]);
    assert!(result.is_err());
}

#[test]
fn test_linker_basic() {
    let mut linker = elf::Linker::new();

    // Create two object files
    let mut obj1 = elf::ElfObjectFile::new();
    obj1.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: vec![0x90, 0x90], // nop; nop
        alignment: 1,
    });

    let mut obj2 = elf::ElfObjectFile::new();
    obj2.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: vec![0xc3], // ret
        alignment: 1,
    });

    linker.add_object(obj1);
    linker.add_object(obj2);

    // Link them together
    let result = linker.link();
    assert!(result.is_ok());
}

#[test]
fn test_comprehensive_instruction_selection() {
    let ctx = Context::new();
    let module = Module::new("comprehensive".to_string(), ctx.clone());

    // Create a function that uses various operations
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], false);

    let function = Function::new("test_ops".to_string(), fn_type);
    let entry_bb = BasicBlock::new(Some("entry".to_string()));
    function.add_basic_block(entry_bb.clone());

    // Build some instructions using builder
    let mut builder = Builder::new(ctx.clone());
    builder.position_at_end(entry_bb.clone());

    let arg = llvm_rust::Value::argument(i32_type.clone(), 0, Some("arg".to_string()));
    let const_val = llvm_rust::Value::const_int(i32_type.clone(), 42, Some("const".to_string()));

    // Add instruction
    let sum = builder.build_add(arg.clone(), const_val.clone(), Some("sum".to_string()));

    // Return
    builder.build_ret(sum);

    module.add_function(function.clone());

    // Generate assembly
    let mut target = X86_64TargetMachine::new();
    let asm = target.emit_assembly(&module);

    assert!(asm.is_ok());
    let asm_code = asm.unwrap();

    // Should contain function and basic instructions
    assert!(asm_code.contains("test_ops:"));
    assert!(asm_code.contains("ret"));
}
