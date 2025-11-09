use llvm_rust::{Context, parse};

#[test]
fn test_callbr_simple() {
    let content = r##"
define void @test() {
  callbr void asm sideeffect "#test", "!i"()
  to label %1 [label %2]
1:
  ret void
2:
  ret void
}
"##;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed callbr simple"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_callbr_first_function() {
    let content = r##"
define void @too_few_label_constraints() {
  callbr void asm sideeffect "#too_few_label_constraints", "!i"()
  to label %1 [label %2, label %3]
1:
  ret void
2:
  ret void
3:
  ret void
}
"##;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed first function from callbr.ll"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_numeric_labels() {
    let content = r#"
define void @test() {
entry:
  br label %1
1:
  br label %2
2:
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed numeric labels"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
