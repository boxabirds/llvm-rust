use llvm_rust::{Context, parse};

#[test]
fn test_opaque_type_definition() {
    let content = r#"
%opaque.ty = type opaque
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed opaque type definition"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_opaque_type_with_function() {
    let content = r#"
%opaque.ty = type opaque

define void @test(ptr byref(%opaque.ty)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed opaque type with function"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
