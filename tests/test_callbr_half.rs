use llvm_rust::{Context, parse};
use std::fs;

#[test]
#[ignore] // Debug test - splits negative test file arbitrarily, produces incomplete IR
fn test_callbr_first_half() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/callbr.ll")
        .expect("Failed to read file");

    // Take only first 61 lines
    let lines: Vec<&str> = content.lines().take(61).collect();
    let first_half = lines.join("\n");

    let ctx = Context::new();
    match parse(&first_half, ctx) {
        Ok(_) => println!("✓ Parsed first half"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
