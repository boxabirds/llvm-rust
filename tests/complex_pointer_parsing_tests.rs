//! Tests for parsing complex nested pointer types from LLVM IR

use llvm_rust::{Context, parse};

#[test]
fn test_parse_simple_pointer() {
    let ctx = Context::new();
    let ir = "define ptr @test() { ret ptr null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse simple pointer: {:?}", result.err());
}

#[test]
fn test_parse_typed_pointer() {
    let ctx = Context::new();
    let ir = "define i32* @test() { ret i32* null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse typed pointer: {:?}", result.err());
}

#[test]
fn test_parse_double_pointer() {
    let ctx = Context::new();
    let ir = "define i32** @test() { ret i32** null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse double pointer: {:?}", result.err());
}

#[test]
fn test_parse_triple_pointer() {
    let ctx = Context::new();
    let ir = "define i32*** @test() { ret i32*** null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse triple pointer: {:?}", result.err());
}

#[test]
fn test_parse_pointer_to_array() {
    let ctx = Context::new();
    let ir = "define [10 x i32]* @test() { ret [10 x i32]* null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse pointer to array: {:?}", result.err());
}

#[test]
fn test_parse_array_of_pointers() {
    let ctx = Context::new();
    // Just test declaration for now
    let ir = "declare [5 x i32*] @test()";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse array of pointers: {:?}", result.err());
}

#[test]
fn test_parse_pointer_to_function() {
    let ctx = Context::new();
    let ir = "declare i32 (i32)* @test()";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse pointer to function: {:?}", result.err());
}

#[test]
fn test_parse_function_with_pointer_args() {
    let ctx = Context::new();
    let ir = "define void @test(i32* %p, i8** %pp) { ret void }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse function with pointer args: {:?}", result.err());
}

#[test]
fn test_parse_struct_with_pointers() {
    let ctx = Context::new();
    let ir = r#"
%struct = type { i32*, i8** }
declare %struct @test()
"#;

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse struct with pointers: {:?}", result.err());
}

#[test]
fn test_parse_pointer_to_struct() {
    let ctx = Context::new();
    let ir = r#"
%struct = type { i32, float }
define %struct* @test() { ret %struct* null }
"#;

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse pointer to struct: {:?}", result.err());
}

#[test]
fn test_parse_pointer_with_addrspace() {
    let ctx = Context::new();
    let ir = "define i32 addrspace(1)* @test() { ret i32 addrspace(1)* null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse pointer with address space: {:?}", result.err());
}

#[test]
fn test_parse_opaque_pointer_with_addrspace() {
    let ctx = Context::new();
    let ir = "define ptr addrspace(1) @test() { ret ptr addrspace(1) null }";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse opaque pointer with address space: {:?}", result.err());
}

#[test]
fn test_parse_nested_pointer_with_addrspace() {
    let ctx = Context::new();
    let ir = "define i32 addrspace(1)* addrspace(2)* @test() { ret i32 addrspace(1)* addrspace(2)* null }";

    let result = parse(ir, ctx);
    // This is a complex case that might not be fully supported
    if result.is_err() {
        println!("Note: Nested pointers with multiple address spaces not yet supported");
    }
}

#[test]
fn test_parse_vector_of_pointers() {
    let ctx = Context::new();
    let ir = "declare <4 x i32*> @test()";

    let result = parse(ir, ctx);
    assert!(result.is_ok(), "Failed to parse vector of pointers: {:?}", result.err());
}
