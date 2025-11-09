use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_callbr_ll() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/callbr.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed callbr.ll"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
