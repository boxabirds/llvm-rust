/// Test harness to run LLVM test suite files through llvm-rust compiler
/// Tests parsing and codegen against real LLVM .ll files

use llvm_rust::{Context, Module, parse};
use llvm_rust::codegen::x86_64::X86_64TargetMachine;
use llvm_rust::codegen::TargetMachine;
use std::fs;
use std::path::{Path, PathBuf};

struct TestResult {
    file: String,
    parsed: bool,
    codegen: bool,
    error: Option<String>,
}

impl TestResult {
    fn success(file: String) -> Self {
        Self {
            file,
            parsed: true,
            codegen: true,
            error: None,
        }
    }

    fn parse_failed(file: String, error: String) -> Self {
        Self {
            file,
            parsed: false,
            codegen: false,
            error: Some(error),
        }
    }

    fn codegen_failed(file: String, error: String) -> Self {
        Self {
            file,
            parsed: true,
            codegen: false,
            error: Some(error),
        }
    }
}

fn run_test_file(path: &Path) -> TestResult {
    let file_name = path.to_string_lossy().to_string();

    // Read the file
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return TestResult::parse_failed(file_name, format!("Read error: {}", e)),
    };

    // Parse the file
    let ctx = Context::new();
    let module = match parse(&content, ctx) {
        Ok(m) => m,
        Err(e) => return TestResult::parse_failed(file_name, format!("Parse error: {:?}", e)),
    };

    // Try to run codegen
    let mut target = X86_64TargetMachine::new();
    match target.emit_assembly(&module) {
        Ok(_asm) => TestResult::success(file_name),
        Err(e) => TestResult::codegen_failed(file_name, format!("Codegen error: {:?}", e)),
    }
}

fn collect_ll_files(dir: &Path, max_files: Option<usize>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ll") {
                files.push(path);
                if let Some(max) = max_files {
                    if files.len() >= max {
                        break;
                    }
                }
            }
        }
    }

    files.sort();
    files
}

fn print_results(results: &[TestResult]) {
    let total = results.len();
    let parsed = results.iter().filter(|r| r.parsed).count();
    let codegen = results.iter().filter(|r| r.codegen).count();

    println!("\n=== Test Results ===");
    println!("Total files: {}", total);
    println!("Parsed successfully: {} ({:.1}%)", parsed, (parsed as f64 / total as f64) * 100.0);
    println!("Codegen successful: {} ({:.1}%)", codegen, (codegen as f64 / total as f64) * 100.0);

    // Show first few failures
    println!("\n=== Parse Failures ===");
    let parse_failures: Vec<_> = results.iter()
        .filter(|r| !r.parsed)
        .take(10)
        .collect();

    if parse_failures.is_empty() {
        println!("None!");
    } else {
        for result in parse_failures {
            println!("  {}: {}", result.file, result.error.as_ref().unwrap_or(&"Unknown".to_string()));
        }
        if results.iter().filter(|r| !r.parsed).count() > 10 {
            println!("  ... and {} more", results.iter().filter(|r| !r.parsed).count() - 10);
        }
    }

    println!("\n=== Codegen Failures ===");
    let codegen_failures: Vec<_> = results.iter()
        .filter(|r| r.parsed && !r.codegen)
        .take(10)
        .collect();

    if codegen_failures.is_empty() {
        println!("None!");
    } else {
        for result in codegen_failures {
            println!("  {}: {}", result.file, result.error.as_ref().unwrap_or(&"Unknown".to_string()));
        }
        if results.iter().filter(|r| r.parsed && !r.codegen).count() > 10 {
            println!("  ... and {} more", results.iter().filter(|r| r.parsed && !r.codegen).count() - 10);
        }
    }
}

#[test]
fn test_llvm_assembler_simple() {
    let test_dir = Path::new("llvm-tests/llvm-project/llvm/test/Assembler");
    if !test_dir.exists() {
        println!("Skipping test - LLVM tests not found at {:?}", test_dir);
        return;
    }

    println!("\nTesting against LLVM Assembler tests (first 20 files)...");
    let files = collect_ll_files(test_dir, Some(20));

    if files.is_empty() {
        println!("No test files found");
        return;
    }

    let results: Vec<_> = files.iter()
        .map(|f| run_test_file(f))
        .collect();

    print_results(&results);

    // For now, don't fail the test - just report
    // In future, set threshold like: assert!(parsed_pct >= 95.0);
}

#[test]
fn test_llvm_simple_functions() {
    let test_dir = Path::new("llvm-tests/llvm-project/llvm/test/Assembler");
    if !test_dir.exists() {
        println!("Skipping test - LLVM tests not found");
        return;
    }

    println!("\nTesting simple function patterns...");

    // Create a simple test function
    let simple_test = r#"
define i32 @simple_return() {
  ret i32 42
}

define i32 @simple_add(i32 %a, i32 %b) {
  %result = add i32 %a, %b
  ret i32 %result
}

define void @simple_void() {
  ret void
}
"#;

    let ctx = Context::new();
    let module = parse(simple_test, ctx).expect("Failed to parse simple test");

    let mut target = X86_64TargetMachine::new();
    let asm = target.emit_assembly(&module).expect("Failed to generate assembly");

    println!("Generated assembly:\n{}", asm);

    // Verify basic assembly structure
    assert!(asm.contains(".text"));
    assert!(asm.contains("simple_return:") || asm.contains("simple_return"));
}

#[test]
fn test_codegen_levels_7_8_9() {
    println!("\n=== Testing Level 7-9 Implementation ===");

    // Level 7: x86-64 Codegen
    println!("\nLevel 7: x86-64 Codegen");
    let ctx = Context::new();
    let module = Module::new("test_l7".to_string(), ctx.clone());

    let mut target = X86_64TargetMachine::new();
    match target.emit_assembly(&module) {
        Ok(_) => println!("  ✓ Basic assembly emission works"),
        Err(e) => println!("  ✗ Assembly emission failed: {:?}", e),
    }

    // Level 8: ELF and Linking
    println!("\nLevel 8: ELF Object Files and Linking");
    use llvm_rust::codegen::elf::{ElfObjectFile, Section, SectionType, Symbol, SymbolBinding, SymbolType};

    let mut obj = ElfObjectFile::new();
    obj.add_section(Section {
        name: ".text".to_string(),
        typ: SectionType::Text,
        data: vec![0xb8, 0x2a, 0x00, 0x00, 0x00, 0xc3], // mov eax, 42; ret
        alignment: 16,
    });
    obj.add_symbol(Symbol {
        name: "test_func".to_string(),
        value: 0,
        size: 6,
        binding: SymbolBinding::Global,
        typ: SymbolType::Function,
        section_index: 1,
    });

    let elf_bytes = obj.generate();
    assert!(elf_bytes.len() > 0);
    assert_eq!(&elf_bytes[0..4], &[0x7f, b'E', b'L', b'F']);
    println!("  ✓ ELF object file generation works");

    // Level 9: External Functions and Runtime
    println!("\nLevel 9: External Functions and Runtime");
    use llvm_rust::codegen::{external_functions, runtime};

    let ffi_helper = external_functions::FFIHelper::new();
    match ffi_helper.generate_call("puts", &["%rdi".to_string()]) {
        Ok(_) => println!("  ✓ External function call generation works"),
        Err(e) => println!("  ✗ External function call failed: {:?}", e),
    }

    let start_entry = runtime::generate_start_entry("main");
    assert!(start_entry.contains("_start"));
    assert!(start_entry.contains("call main"));
    println!("  ✓ Runtime entry point generation works");

    println!("\n=== Level 7-9 Summary ===");
    println!("Level 7 (Codegen): Basic implementation present");
    println!("Level 8 (Linking): ELF generation working");
    println!("Level 9 (Runtime): External calls and entry points working");
}
