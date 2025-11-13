use llvm_rust::{Context, parse};
use std::fs;

#[test]
#[ignore] // Temporary debug test - requires /tmp/byref_test.ll to exist
fn test_byref_first_half() {
    let content = fs::read_to_string("/tmp/byref_test.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed test segment"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
