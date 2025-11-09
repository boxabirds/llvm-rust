use llvm_rust::{Context, parse};

#[test]
fn test_special_labels() {
    let content = r#"
define void @test() {
entry:
  br label %-3
-3:
  br label %-N-
-N-:
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed special labels"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_numeric_string_label() {
    let content = r#"
define void @test() {
  br label %3
3:
  br label %"2"
"2":
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed numeric and string labels"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
