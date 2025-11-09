use llvm_rust::lexer::Lexer;
use std::fs;

fn main() {
    let content = fs::read_to_string("/home/user/llvm-rust/llvm-tests/llvm-project/test/Bitcode/miscInstructions.3.2.ll").unwrap();
    
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize().unwrap();
    
    println!("Total tokens: {}", tokens.len());
    
    // Show tokens around position 586 (±35 tokens)
    let start = if 586 > 35 { 586 - 35 } else { 0 };
    let end = if 586 + 35 < tokens.len() { 586 + 35 } else { tokens.len() };
    
    println!("\nTokens from position {} to {}:", start, end);
    for i in start..end {
        let marker = if i == 586 { " >>> " } else { "     " };
        println!("{} [{:3}] {:?}", marker, i, tokens[i]);
    }
    
    if 586 < tokens.len() {
        println!("\nToken at position 586: {:?}", tokens[586]);
    }
}
