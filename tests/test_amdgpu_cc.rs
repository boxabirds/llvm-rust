use llvm_rust::{Context, parse};

#[test]
fn test_amdgpu_sret() {
    // This is a NEGATIVE test - amdgpu_kernel calling convention does not allow sret
    // Reference: LLVM test Verifier/amdgpu-cc.ll lines 17-21
    let content = r#"
define amdgpu_kernel void @sret_cc_amdgpu_kernel_as0(ptr sret(i32) %ptr) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for amdgpu_kernel with sret, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected amdgpu_kernel with sret: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("calling convention") && err_msg.contains("sret"),
                    "Expected error about calling convention and sret, got: {:?}", e);
        }
    }
}
