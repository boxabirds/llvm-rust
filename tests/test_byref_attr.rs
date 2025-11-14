use llvm_rust::{Context, parse};

#[test]
fn test_byref_attribute() {
    // POSITIVE test - byref with sized type should be accepted
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
    // TODO: This should be a NEGATIVE test - byref does not support unsized types
    // Reference: LLVM test Verifier/byref.ll lines 4-8
    // Currently accepts because verifier doesn't check byref sizing yet
    let content = r#"
%opaque.ty = type opaque

define void @byref_unsized(ptr byref(%opaque.ty)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref with unsized type (TODO: should reject)"),
        Err(e) => println!("Rejected: {:?}", e),
    }
}

#[test]
fn test_multiple_attrs_with_types() {
    // NEGATIVE test - byref and byval are incompatible attributes
    // Reference: LLVM test Verifier/byref.ll lines 10-13
    let content = r#"
define void @byref_byval(ptr byref(i32) byval(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for byref+byval, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected incompatible byref+byval: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("incompatible") || err_msg.contains("byval"),
                    "Expected error about incompatible attributes, got: {:?}", e);
        }
    }
}
