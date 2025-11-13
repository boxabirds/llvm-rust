use llvm_rust::{Context, parse};

#[test]
fn test_byval_with_type() {
    // NEGATIVE test - byval can only be applied to pointer types
    // Reference: LLVM test Verifier/byval-1.ll
    let content = r#"
declare void @h(i32 byval(i32) %num)
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for byval on non-pointer type, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected byval on non-pointer type: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("byval") || err_msg.contains("incompatible"),
                    "Expected error about byval or incompatible type, got: {:?}", e);
        }
    }
}
