use llvm_rust::{Context, parse};

#[test]
fn test_amdgpu_sret() {
    let content = r#"
define amdgpu_kernel void @sret_cc_amdgpu_kernel_as0(ptr sret(i32) %ptr) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed amdgpu sret"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
