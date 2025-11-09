use llvm_rust::{Context, parse};

#[test]
fn test_byval_with_type() {
    let content = r#"
declare void @h(i32 byval(i32) %num)
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byval with type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
