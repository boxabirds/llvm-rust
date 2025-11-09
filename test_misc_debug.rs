use llvm_rust::lexer::Lexer;
use std::fs;

fn main() {
    let file = "/home/user/llvm-rust/llvm-tests/llvm-project/test/Bitcode/miscInstructions.3.2.ll";
    println!("Tokenizing {}\n", file);
    
    let content = fs::read_to_string(file).expect("Failed to read file");
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize().expect("Lexer failed");
    
    println!("Total tokens: {}\n", tokens.len());
    
    // Show tokens around position 586 (±40 tokens)
    let start = if 586 > 40 { 586 - 40 } else { 0 };
    let end = std::cmp::min(586 + 40, tokens.len());
    
    println!("Tokens from position {} to {} (around error position 586):\n", start, end);
    for i in start..end {
        let marker = if i == 586 { " >>> " } else { "     " };
        println!("{} [{}] {:?}", marker, i, tokens[i]);
    }
    
    if 586 < tokens.len() {
        println!("\n=== Token at position 586 (where error occurs) ===");
        println!("{:?}", tokens[586]);
    }
}
