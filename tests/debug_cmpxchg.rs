use llvm_rust::{Context, parse};

#[test]
fn test_cmpxchg_weak() {
    let content = r#"
define void @test(ptr %x) {
  cmpxchg weak ptr %x, i32 13, i32 0 seq_cst monotonic
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Cmpxchg weak test passed"),
        Err(e) => panic!("Failed to parse cmpxchg weak: {:?}", e),
    }
}

#[test]
fn test_alloca_addrspace() {
    let content = r#"
target datalayout = "A0"
define void @use_alloca() {
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
