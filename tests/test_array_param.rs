use llvm_rust::{Context, parse};

#[test]
fn test_array_param() {
    let content = r#"
declare void @llvm.test.immarg.intrinsic.2ai32([2 x i32] immarg)

define void @test() {
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed array parameter"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
