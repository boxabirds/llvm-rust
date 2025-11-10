use llvm_rust::{Context, parse};

#[test]
fn test_escape_file() {
    let ir = r#"
define i32 @test() {
entry:
	%tmp = tail call i32 @Func64( ptr null )
	%tmp1 = tail call i32 @Func64( ptr null )
	ret i32 undef
}

define i32 @Func64(ptr %B) {
entry:
	ret i32 0
}
"#;

    let ctx = Context::new();
    let result = parse(ir, ctx);

    match &result {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_2007_05_21_escape() {
    let content = std::fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm/test/Assembler/2007-05-21-Escape.ll")
        .expect("Failed to read file");

    let ctx = Context::new();
    let result = parse(&content, ctx);

    match &result {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
