use llvm_rust::{Context, parse};
use std::fs;

fn main() {
    let file = "/home/user/llvm-rust/llvm-tests/llvm-project/test/Bitcode/miscInstructions.3.2.ll";
    println!("Testing {}", file);
    
    let content = fs::read_to_string(file).expect("Failed to read file");
    let ctx = Context::new();
    
    match parse(&content, ctx) {
        Ok(_) => println!("✓ SUCCESS"),
        Err(e) => println!("✗ FAILED: {:?}", e),
    }
}
