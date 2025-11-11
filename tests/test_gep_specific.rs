use llvm_rust::{Context, parse, verification::verify_module};

#[test]
fn test_gep_pointer_indexing() {
    let input = r#"
define void @test(ptr %X) {
	getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0
	ret void
}
"#;

    let ctx = Context::new();
    let module = parse(input, ctx).expect("Parse should succeed");
    
    // This should fail verification because we're indexing through a pointer
    match verify_module(&module) {
        Ok(()) => panic!("Verification should have failed!"),
        Err(errors) => {
            println!("Correctly failed with errors:");
            for err in &errors {
                println!("  - {}", err);
            }
            assert!(!errors.is_empty(), "Should have at least one error");
        }
    }
}
