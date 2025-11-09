use llvm_rust::{Context, parse};

#[test]
fn test_alloca_with_metadata() {
    let content = r#"
define void @use_alloca() {
  %alloca_scalar_no_align_metadata = alloca i32, addrspace(0), !foo !0
  ret void
}

!0 = !{}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Alloca with metadata test passed"),
        Err(e) => panic!("Failed to parse alloca with metadata: {:?}", e),
    }
}

#[test]
fn test_alloca_align_and_addrspace_and_metadata() {
    let content = r#"
define void @use_alloca() {
  %alloca_scalar_align4_metadata = alloca i32, align 4, addrspace(0), !foo !0
  ret void
}

!0 = !{}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Alloca with align, addrspace, and metadata test passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
