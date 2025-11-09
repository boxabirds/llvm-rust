use llvm_rust::{Context, parse};

#[test]
fn test_strictfp_attribute() {
    let content = r#"
define double @f2(double %a, double %b) #0 {
entry:
  ret double %a
}

attributes #0 = { strictfp }
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed strictfp attribute"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
