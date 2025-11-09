use llvm_rust::{Context, parse};
use std::fs;

#[test]
fn test_inalloca2() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Verifier/inalloca2.ll")
        .expect("Failed to read file");
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed inalloca2.ll"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_inalloca_alloca() {
    let content = r#"
define void @test() {
  %a = alloca inalloca i64
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed alloca inalloca"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
