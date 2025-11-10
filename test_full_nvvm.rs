use llvm_rust::{Context, parse};

fn main() {
    let ir = std::fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm/test/Assembler/auto_upgrade_nvvm_intrinsics.ll")
        .expect("Failed to read file");

    let ctx = Context::new();
    let result = parse(&ir, ctx);

    match &result {
        Ok(_) => println!("✓ Parsed and verified successfully"),
        Err(e) => println!("✗ Error: {:?}", e),
    }
}
