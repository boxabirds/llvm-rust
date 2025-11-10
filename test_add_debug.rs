use llvm_rust::{Context, parse};

fn main() {
    let ir = std::fs::read_to_string("/home/user/llvm-rust/test_add_metadata.ll")
        .expect("Failed to read file");

    let ctx = Context::new();
    let result = parse(&ir, ctx);

    match &result {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
