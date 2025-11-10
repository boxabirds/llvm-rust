use llvm_rust::{Context, parse, verification::verify_module};

fn main() {
    let ir = std::fs::read_to_string("/home/user/llvm-rust/test_nvvm_atomic.ll")
        .expect("Failed to read file");

    let ctx = Context::new();
    let result = parse(&ir, ctx);

    match &result {
        Ok(module) => {
            println!("✓ Parsed successfully");
            match verify_module(module) {
                Ok(()) => println!("✓ Verification passed"),
                Err(errors) => {
                    println!("✗ Verification failed:");
                    for error in &errors {
                        println!("  {:?}", error);
                    }
                }
            }
        },
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
