use llvm_rust::{Context, parse};

#[test]
fn test_byref_on_non_pointer() {
    let content = r#"
define void @byref_non_pointer(i32 byref(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref on non-pointer type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
