use llvm_rust::{Context, parse};

#[test]
fn test_br_with_labels() {
    let content = r#"
declare i32 @llvm.abs.i32(i32, i1)

define i32 @abs_dom_cond_nopoison(i32 %x) {
  %cmp = icmp sge i32 %x, 0
  br i1 %cmp, label %true, label %false

true:
  %a1 = call i32 @llvm.abs.i32(i32 %x, i1 false)
  ret i32 %a1

false:
  %a2 = call i32 @llvm.abs.i32(i32 %x, i1 false)
  ret i32 %a2
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Branch with labels passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
