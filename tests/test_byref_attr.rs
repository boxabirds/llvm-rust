use llvm_rust::{Context, parse};

#[test]
fn test_byref_attribute() {
    let content = r#"
define void @byref_test(ptr byref(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref attribute"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_byref_with_local_type() {
    let content = r#"
%opaque.ty = type opaque

define void @byref_unsized(ptr byref(%opaque.ty)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref with local type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_multiple_attrs_with_types() {
    let content = r#"
define void @byref_byval(ptr byref(i32) byval(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed multiple attributes with types"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
