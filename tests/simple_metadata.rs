use llvm_rust::{Context, parse};

#[test]
fn test_alloca_no_metadata() {
    let content = r#"
define void @use_alloca() {
  %x = alloca i32
  ret void
}
"#;
    let ctx = Context::new();
    parse(content, ctx).expect("Failed to parse simple alloca");
}

#[test]
fn test_metadata_at_end() {
    let content = r#"
define void @use_alloca() {
  %x = alloca i32, !foo !0
  ret void
}

!0 = !{}
"#;
    let ctx = Context::new();
    parse(content, ctx).expect("Failed");
}
