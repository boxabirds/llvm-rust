use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_elementtype_ll() {
    // This is a NEGATIVE test file in LLVM - contains invalid elementtype IR
    // Reference: LLVM test Verifier/elementtype.ll (RUN: not llvm-as)
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/elementtype.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => panic!("Expected parse to fail for elementtype.ll (contains invalid IR), but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected invalid IR from elementtype.ll: {:?}", e);
            // Just check that we got some error
            assert!(!format!("{:?}", e).is_empty(), "Expected some error message");
        }
    }
}
