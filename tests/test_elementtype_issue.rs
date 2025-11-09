use llvm_rust::{Context, parse};

#[test]
fn test_elementtype_in_call() {
    let content = r#"
declare void @some_function(ptr)

define void @test() {
  call void @some_function(ptr elementtype(i32) null)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed elementtype in call"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_elementtype_with_value() {
    let content = r#"
declare ptr @llvm.preserve.array.access.index.p0.p0(ptr, i32, i32)

define void @test() {
  call ptr @llvm.preserve.array.access.index.p0.p0(ptr null, i32 elementtype(i32) 0, i32 0)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed elementtype with value"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
