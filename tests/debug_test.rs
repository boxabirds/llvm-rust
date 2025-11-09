use llvm_rust::{Context, parse};

#[test]
fn test_atomic_load() {
    let content = r#"
define void @test(ptr %x) {
  load atomic i32, ptr %x unordered, align 4
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Atomic load test passed"),
        Err(e) => panic!("Failed to parse atomic load: {:?}", e),
    }
}

#[test]
fn test_alloca_addrspace() {
    let content = r#"
define void @test() {
  %alloca_scalar_no_align = alloca i32, addrspace(0)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Alloca addrspace test passed"),
        Err(e) => panic!("Failed to parse alloca addrspace: {:?}", e),
    }
}
