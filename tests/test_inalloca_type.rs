use llvm_rust::{Context, parse};

#[test]
fn test_inalloca_with_type() {
    let content = r#"
define void @test(ptr inalloca(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed inalloca with type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_byref_inalloca() {
    // NEGATIVE test - byref and inalloca are incompatible attributes
    // Reference: LLVM test Verifier/byref.ll
    let content = r#"
define void @byref_inalloca(ptr byref(i32) inalloca(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for byref+inalloca, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected incompatible byref+inalloca: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("incompatible"),
                    "Expected error about incompatible attributes, got: {:?}", e);
        }
    }
}
