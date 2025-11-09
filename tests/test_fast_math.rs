use llvm_rust::{Context, parse};

#[test]
fn test_fast_math_simple() {
    let content = r#"
define float @test(float %x, float %y) {
entry:
  %a = fadd nnan float %x, %y
  %b = fsub nnan float %x, %y
  %c = fmul nnan float %x, %y
  %d = fpext nnan float %x to double
  ret float %a
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed fast-math flags"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
