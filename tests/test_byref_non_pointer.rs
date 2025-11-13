use llvm_rust::{Context, parse};

#[test]
fn test_byref_on_non_pointer() {
    // NEGATIVE test - byref can only be applied to pointer types
    // Reference: LLVM test Verifier/byref.ll
    let content = r#"
define void @byref_non_pointer(i32 byref(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for byref on non-pointer type, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected byref on non-pointer type: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("byref") || err_msg.contains("incompatible"),
                    "Expected error about byref or incompatible type, got: {:?}", e);
        }
    }
}
