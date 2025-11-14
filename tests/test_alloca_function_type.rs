use llvm_rust::{Context, parse};

#[test]
fn test_alloca_function_type() {
    // This is a NEGATIVE test - alloca with function type should be rejected
    // Reference: LLVM test Verifier/2008-03-01-AllocaSized.ll
    let content = r#"
define void @test() {
	%A = alloca void()
	ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for alloca with function type, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected alloca with function type: {:?}", e);
            assert!(format!("{:?}", e).contains("invalid type for alloca"));
        }
    }
}
