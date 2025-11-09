use llvm_rust::{Context, parse};

#[test]
fn test_inalloca_with_type() {
    let content = r#"
define void @test(ptr inalloca(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed inalloca with type"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_byref_inalloca() {
    let content = r#"
define void @byref_inalloca(ptr byref(i32) inalloca(i32)) {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed byref and inalloca"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
