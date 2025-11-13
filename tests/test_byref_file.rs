use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_byref_ll_file() {
    // This is a NEGATIVE test file in LLVM - it contains invalid IR that should be rejected
    // Reference: LLVM test Verifier/byref.ll (RUN: not llvm-as)
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/byref.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => panic!("Expected parse to fail for byref.ll (contains invalid IR), but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected invalid IR from byref.ll: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("incompatible") || err_msg.contains("byref"),
                    "Expected error about incompatible attributes or byref, got: {:?}", e);
        }
    }
}

#[test]
fn test_byref_partial() {
    // This contains multiple negative test cases from LLVM's byref.ll
    // All of these should be rejected by the verifier
    let content = r#"
%opaque.ty = type opaque

define void @byref_unsized(ptr byref(%opaque.ty)) {
  ret void
}

define void @byref_byval(ptr byref(i32) byval(i32)) {
  ret void
}

define void @byref_inalloca(ptr byref(i32) inalloca(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => panic!("Expected parse to fail for invalid byref combinations, but it succeeded"),
        Err(e) => {
            println!("✓ Correctly rejected invalid byref combinations: {:?}", e);
            let err_msg = format!("{:?}", e).to_lowercase();
            assert!(err_msg.contains("incompatible") || err_msg.contains("byref"),
                    "Expected error about incompatible attributes or byref, got: {:?}", e);
        }
    }
}
