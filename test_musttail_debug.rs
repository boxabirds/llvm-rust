use llvm_rust::{Context, parse};
use std::fs;

fn main() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Assembler/musttail.ll")
        .expect("Failed to read file");

    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed musttail.ll"),
        Err(e) => println!("✗ Failed: {:?}", e),
    }
}
