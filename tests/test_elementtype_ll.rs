use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_elementtype_ll() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/elementtype.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed elementtype.ll"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
