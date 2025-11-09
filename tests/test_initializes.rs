use llvm_rust::{Context, parse};

#[test]
fn test_initializes_attr() {
    let content = r#"
define void @lower_greater_than_upper1(ptr initializes((4, 0)) %a) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed initializes attribute"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
