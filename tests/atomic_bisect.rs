use llvm_rust::{Context, parse};

#[test]
fn test_cmpxchg_with_syncscope() {
    let content = r#"
define void @f(ptr %x) {
  cmpxchg ptr %x, i32 1, i32 0 syncscope("singlethread") monotonic monotonic
  ret void
}
"#;
    let ctx = Context::new();
    parse(content, ctx).expect("Failed to parse cmpxchg with syncscope");
}

#[test]
fn test_store_then_cmpxchg() {
    let content = r#"
define void @f(ptr %x) {
  store atomic i32 3, ptr %x release, align 4
  cmpxchg ptr %x, i32 1, i32 0 syncscope("singlethread") monotonic monotonic
  ret void
}
"#;
    let ctx = Context::new();
    parse(content, ctx).expect("Failed");
}

#[test]
fn test_multiple_cmpxchg() {
    let content = r#"
define void @f(ptr %x) {
  cmpxchg ptr %x, i32 1, i32 0 syncscope("singlethread") monotonic monotonic
  cmpxchg ptr %x, i32 1, i32 0 syncscope("workitem") monotonic monotonic
  ret void
}
"#;
    let ctx = Context::new();
    parse(content, ctx).expect("Failed");
}
