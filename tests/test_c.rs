use llvm_rust::{Context, parse};
use std::fs;

#[test]
#[ignore] // Temporary debug test - requires /tmp/test_full_c.ll to exist
fn test() {
    let content = fs::read_to_string("/tmp/test_full_c.ll").unwrap();
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ OK"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
