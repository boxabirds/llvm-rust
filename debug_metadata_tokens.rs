use llvm_rust::lexer::Lexer;

fn main() {
    let content = std::fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm/test/Assembler/metadata.ll")
        .expect("Failed to read file");

    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize().expect("Failed to tokenize");

    println!("Total tokens: {}", tokens.len());
    println!("\nTokens around position 88:");
    for i in 80..95 {
        if i < tokens.len() {
            println!("Token {}: {:?}", i, tokens[i]);
        }
    }

    println!("\nLast 20 tokens:");
    let start = tokens.len().saturating_sub(20);
    for i in start..tokens.len() {
        println!("Token {}: {:?}", i, tokens[i]);
    }
}
