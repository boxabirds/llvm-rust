use llvm_rust::{Context, parse};

#[test]
fn test_atomic_load_unordered() {
    let llvm_ir = r#"
define void @test(ptr %x) {
  load atomic i32, ptr %x unordered, align 4
  ret void
}
"#;

    let ctx = Context::new();
    let result = parse(llvm_ir, ctx);

    // This test should pass, but currently it fails with infinite loop
    assert!(result.is_ok(), "Failed to parse atomic load: {:?}", result.err());
}
