use llvm_rust::{Context, parse};

#[test]
fn test_call_with_dbg() {
    let content = r#"
define void @test() {
entry:
  call void @foo(), !dbg !0
  ret void
}

declare void @foo()

!0 = !{}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Call with dbg passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_instruction_metadata_simple() {
    let content = r#"
define void @test() {
  %x = add i32 1, 2, !dbg !0
  ret void
}

!0 = !{}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Instruction metadata passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
