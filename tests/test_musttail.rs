use llvm_rust::{Context, parse};

#[test]
fn test_varargs_declare() {
    let content = r#"
declare ptr @f(ptr, ...)
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed varargs declare"),
        Err(e) => panic!("Failed declare: {:?}", e),
    }
}

#[test]
fn test_varargs_define() {
    let content = r#"
define ptr @f_thunk(ptr %this, ...) {
  ret ptr null
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed varargs define"),
        Err(e) => panic!("Failed define: {:?}", e),
    }
}
