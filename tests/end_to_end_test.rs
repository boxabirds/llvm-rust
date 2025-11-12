use llvm_rust::{Context, Module, Function, Builder};
use llvm_rust::codegen::{elf, external_functions, runtime, stack_frame};
use llvm_rust::codegen::x86_64::X86_64TargetMachine;
use llvm_rust::codegen::TargetMachine;

#[test]
fn test_end_to_end_hello_world() {
    // Create a simple LLVM IR module that calls printf
    let ctx = Context::new();
    let module = Module::new("hello_world".to_string(), ctx.clone());

    // Create main function: i32 main()
    let i32_type = ctx.int32_type();
    let void_type = ctx.void_type();
    let main_type = ctx.function_type(i32_type.clone(), vec![], false);

    let main_func = Function::new("main".to_string(), main_type);
    module.add_function(main_func.clone());

    // Build function body
    let mut builder = Builder::new(ctx.clone());
    let entry_bb = llvm_rust::BasicBlock::new(Some("entry".to_string()));
    main_func.add_basic_block(entry_bb.clone());
    builder.position_at_end(entry_bb);

    // Return 0
    let zero = llvm_rust::Value::const_int(i32_type.clone(), 0, Some("zero".to_string()));
    builder.build_ret(zero);

    // Generate assembly
    let mut target = X86_64TargetMachine::new();
    let asm_result = target.emit_assembly(&module);
    assert!(asm_result.is_ok());

    let assembly = asm_result.unwrap();
    println!("Generated assembly:\n{}", assembly);

    // Verify assembly contains expected elements
    assert!(assembly.contains(".text"));
    assert!(assembly.contains("main:"));
    assert!(assembly.contains("ret"));
}

#[test]
fn test_external_function_call_generation() {
    // Test generating code that calls printf
    let ffi_helper = external_functions::FFIHelper::new();

    // Generate call to puts("Hello, World!")
    let call_asm = ffi_helper.generate_call("puts", &["%rax".to_string()]);
    assert!(call_asm.is_ok());

    let asm = call_asm.unwrap();
    println!("Generated FFI call:\n{}", asm);

    assert!(asm.contains("mov rdi") || asm.contains("mov %rdi"));
    assert!(asm.contains("call puts@PLT"));
}

#[test]
fn test_complete_executable_structure() {
    // Test creating a complete executable structure

    // 1. Create main function object code
    let mut main_obj = elf::ElfObjectFile::new();

    // Simple return 0 in assembly (mov eax, 0; ret)
    let main_code = vec![
        0xb8, 0x00, 0x00, 0x00, 0x00,  // mov eax, 0
        0xc3,                           // ret
    ];

    main_obj.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: main_code,
        alignment: 16,
    });

    main_obj.add_symbol(elf::Symbol {
        name: "main".to_string(),
        value: 0,
        size: 6,
        binding: elf::SymbolBinding::Global,
        typ: elf::SymbolType::Function,
        section_index: 1,
    });

    // 2. Create _start entry point
    let start_asm = runtime::generate_start_entry("main");
    println!("Generated _start:\n{}", start_asm);

    assert!(start_asm.contains("_start:"));
    assert!(start_asm.contains("call main"));
    assert!(start_asm.contains("syscall"));

    // 3. Link objects together
    let mut linker = elf::Linker::new();
    linker.add_object(main_obj);

    let executable = linker.link();
    assert!(executable.is_ok());

    let elf_bytes = executable.unwrap();

    // Verify ELF structure
    assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);
    println!("Generated executable: {} bytes", elf_bytes.len());
}

#[test]
fn test_stack_frame_with_locals() {
    // Test stack frame management for a function with local variables
    let mut frame = stack_frame::StackFrame::new();

    // Allocate some locals
    let x_offset = frame.allocate_local("x".to_string(), 8, 8);
    let y_offset = frame.allocate_local("y".to_string(), 4, 4);
    let z_offset = frame.allocate_local("z".to_string(), 8, 8);

    // All offsets should be negative (below frame pointer)
    assert!(x_offset < 0);
    assert!(y_offset < 0);
    assert!(z_offset < 0);

    // Generate prologue and epilogue
    let prologue = frame.gen_prologue();
    let epilogue = frame.gen_epilogue();

    println!("Prologue:");
    for line in &prologue {
        println!("{}", line);
    }

    println!("\nEpilogue:");
    for line in &epilogue {
        println!("{}", line);
    }

    // Verify frame size is aligned to 16 bytes
    assert_eq!(frame.total_size() % 16, 0);
}

#[test]
fn test_linker_symbol_resolution() {
    // Test linker's symbol resolution
    let mut linker = elf::Linker::new();

    // Object 1: defines foo
    let mut obj1 = elf::ElfObjectFile::new();
    obj1.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: vec![0x90, 0x90],  // nop; nop
        alignment: 1,
    });
    obj1.add_symbol(elf::Symbol {
        name: "foo".to_string(),
        value: 0,
        size: 2,
        binding: elf::SymbolBinding::Global,
        typ: elf::SymbolType::Function,
        section_index: 1,
    });

    // Object 2: calls foo
    let mut obj2 = elf::ElfObjectFile::new();
    obj2.add_section(elf::Section {
        name: ".text".to_string(),
        typ: elf::SectionType::Text,
        data: vec![0xe8, 0x00, 0x00, 0x00, 0x00],  // call offset
        alignment: 1,
    });
    obj2.add_relocation(elf::Relocation {
        offset: 1,
        symbol: "foo".to_string(),
        typ: elf::RelocationType::R_X86_64_PC32,
        addend: -4,
    });

    linker.add_object(obj1);
    linker.add_object(obj2);

    let result = linker.link();
    assert!(result.is_ok(), "Linking should succeed");

    // Verify entry point can be found
    linker.set_entry_point("foo".to_string());
    let entry_addr = linker.entry_point_address();
    assert!(entry_addr.is_some());
}

#[test]
fn test_calling_convention() {
    let cc = stack_frame::CallingConvention::system_v_amd64();

    // Test argument register mapping
    assert_eq!(cc.arg_register(0), Some("%rdi"));
    assert_eq!(cc.arg_register(1), Some("%rsi"));
    assert_eq!(cc.arg_register(2), Some("%rdx"));
    assert_eq!(cc.arg_register(3), Some("%rcx"));
    assert_eq!(cc.arg_register(4), Some("%r8"));
    assert_eq!(cc.arg_register(5), Some("%r9"));
    assert_eq!(cc.arg_register(6), None);  // 7th arg goes on stack

    // Test register save requirements
    assert!(cc.is_caller_saved("%rax"));
    assert!(cc.is_callee_saved("%rbx"));
    assert!(cc.is_callee_saved("%r12"));
}

#[test]
fn test_program_headers() {
    let headers = runtime::generate_program_headers();

    // Should have at least PT_LOAD segments
    assert!(!headers.is_empty());

    // First segment should be executable
    assert_eq!(headers[0].p_type, runtime::PT_LOAD);
    assert!(headers[0].p_flags & runtime::PF_X != 0);

    // Check that headers can be serialized
    for header in &headers {
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 56);  // 64-bit program header size
    }
}
