use llvm_rust::{Context, parse};

#[test]
fn test_addrspace_star_syntax() {
    let content = r#"
declare i32 addrspace(1)* @foo()
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed addrspace(1)* syntax"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_token_none_in_call() {
    let content = r#"
declare ptr @llvm.experimental.gc.relocate(token, i32, i32)

define void @test() {
    %token_call = call ptr @llvm.experimental.gc.relocate(token none, i32 0, i32 0)
    ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed token none in call"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
