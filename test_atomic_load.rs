use llvm_rust::{Context, parse};

fn main() {
    let llvm_ir = r#"
define void @test(ptr %x) {
  load atomic i32, ptr %x unordered, align 4
  ret void
}
"#;

    println!("Testing atomic load parsing...");
    let ctx = Context::new();

    match parse(llvm_ir, ctx) {
        Ok(_) => println!("✓ SUCCESS"),
        Err(e) => println!("✗ FAILED: {:?}", e),
    }
}
