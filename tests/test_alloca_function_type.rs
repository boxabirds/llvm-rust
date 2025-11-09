use llvm_rust::{Context, parse};

#[test]
fn test_alloca_function_type() {
    let content = r#"
define void @test() {
	%A = alloca void()
	ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed alloca with function type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
