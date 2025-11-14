use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_callbr_ll() {
    // This is a NEGATIVE test file in LLVM - contains invalid callbr IR
    // Reference: LLVM test Verifier/callbr.ll (RUN: not opt)
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/callbr.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => panic!("Expected parse to fail for callbr.ll (contains invalid IR), but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected invalid IR from callbr.ll: {:?}", e);
            // The file contains multiple negative test cases, just check that we got some error
            assert!(!format!("{:?}", e).is_empty(), "Expected some error message");
        }
    }
}
