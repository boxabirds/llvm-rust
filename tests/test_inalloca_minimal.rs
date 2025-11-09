use llvm_rust::{Context, parse};

#[test]
fn test_just_declare() {
    let content = r#"
declare void @doit(ptr inalloca(i64) %a)
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed declare"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_first_function() {
    let content = r#"
declare void @doit(ptr inalloca(i64) %a)

define void @a() {
entry:
  %a = alloca inalloca [2 x i32]
  call void @doit(ptr inalloca(i64) %a)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed first function"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_phi_function() {
    let content = r#"
declare void @doit(ptr inalloca(i64) %a)

define void @c(i1 %cond) {
entry:
  br i1 %cond, label %if, label %else

if:
  %a = alloca inalloca i64
  br label %call

else:
  %b = alloca inalloca i64
  br label %call

call:
  %args = phi ptr [ %a, %if ], [ %b, %else ]
  call void @doit(ptr inalloca(i64) %args)
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed phi function"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
