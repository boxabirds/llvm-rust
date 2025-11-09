use llvm_rust::lexer::Lexer;

fn main() {
    let content = r#"
define void @use_alloca() {
  %x = alloca i32, !foo !0
  ret void
}

!0 = !{}
"#;

    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();

    println!("Total tokens: {}", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }
}
