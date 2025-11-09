use llvm_rust::{Context, parse};

fn main() {
    let content = r#"
declare ptr @f(ptr, ...)

define ptr @f_thunk(ptr %this, ...) {
  %rv = musttail call ptr (ptr, ...) @f(ptr %this, ...)
  ret ptr %rv
}
"#;

    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("✗ Failed: {:?}", e),
    }
}
