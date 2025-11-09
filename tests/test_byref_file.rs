use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_byref_ll_file() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/byref.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed byref.ll"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_byref_partial() {
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
        Ok(_) => println!("✓ Parsed byref partial"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
