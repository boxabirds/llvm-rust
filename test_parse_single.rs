use llvm_rust::{Context, parse};
use std::fs;

fn main() {
    let files = vec!["test_debug.ll", "test_alloca.ll"];
    
    for file in files {
        println!("\n=== Testing {} ===", file);
        let content = fs::read_to_string(file).expect("Failed to read file");
        let ctx = Context::new();
        
        match parse(&content, ctx) {
            Ok(_) => println!("✓ SUCCESS"),
            Err(e) => println!("✗ FAILED: {:?}", e),
        }
    }
}
