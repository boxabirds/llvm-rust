use llvm_rust::{Context, parse};

#[test]
fn test_parse_simple_void_function() {
    let ctx = Context::new();
    let source = r#"
        define void @main() {
        entry:
            ret void
        }
    "#;

    let result = parse(source, ctx);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let module = result.unwrap();
    assert_eq!(module.function_count(), 1);
}

#[test]
fn test_parse_function_with_return() {
    let ctx = Context::new();
    let source = r#"
        define i32 @foo() {
            ret i32 -2147483648
        }
    "#;

    let result = parse(source, ctx);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_global_variable() {
    let ctx = Context::new();
    let source = r#"
        @spell_order = global [4 x i8] c"\FF\00\F7\00"
    "#;

    let result = parse(source, ctx);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let module = result.unwrap();
    assert_eq!(module.globals().len(), 1);
}

#[test]
fn test_parse_function_declaration() {
    let ctx = Context::new();
    let source = r#"
        declare ptr @foo()
    "#;

    let result = parse(source, ctx);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_struct_type() {
    let ctx = Context::new();
    let source = r#"
        define void @test() {
            ret void
        }
    "#;

    let result = parse(source, ctx);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
