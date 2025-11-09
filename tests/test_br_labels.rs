use llvm_rust::{Context, parse};

#[test]
fn test_br_with_identifier_labels() {
    let content = r#"
define void @test(i1 %cond) {
entry:
  br i1 %cond, label %if, label %else

if:
  ret void

else:
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed br with identifier labels"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
