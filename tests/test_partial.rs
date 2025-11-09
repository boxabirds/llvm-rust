use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test() {
    let content = fs::read_to_string("/tmp/test_inalloca_partial.ll").unwrap();
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ OK"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
