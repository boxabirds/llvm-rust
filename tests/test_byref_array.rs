use llvm_rust::{Context, parse};

#[test]
fn test_byref_with_array_type() {
    let content = r#"
define void @byref_callee(ptr byref([64 x i8])) {
  ret void
}

define void @musttail_byref_caller(ptr %ptr) {
  musttail call void @byref_callee(ptr byref([64 x i8]) %ptr)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref with array type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
