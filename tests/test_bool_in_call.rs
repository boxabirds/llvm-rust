use llvm_rust::{Context, parse};

#[test]
fn test_call_with_i1_true() {
    let content = r#"
declare i8 @llvm.abs.i8(i8, i1)

define i8 @test(i8 %a) {
  %abs1 = call i8 @llvm.abs.i8(i8 %a, i1 true)
  ret i8 %abs1
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Call with i1 true passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_call_with_i1_false() {
    let content = r#"
declare i8 @llvm.abs.i8(i8, i1)

define i8 @test(i8 %a) {
  %abs1 = call i8 @llvm.abs.i8(i8 %a, i1 false)
  ret i8 %abs1
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Call with i1 false passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
