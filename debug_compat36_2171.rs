use llvm_rust::lexer::Lexer;
use std::fs;

fn main() {
    let file = "/home/user/llvm-rust/llvm-tests/llvm-project/test/Bitcode/compatibility-3.6.ll";
    println!("Tokenizing {}\n", file);

    let content = fs::read_to_string(file).expect("Failed to read file");
    let mut lexer = Lexer::new(&content);
    let tokens = lexer.tokenize().expect("Lexer failed");

    println!("Total tokens: {}\n", tokens.len());

    // Show tokens 2165-2175 (10 tokens centered on 2171)
    let start = 2165;
    let end = std::cmp::min(2176, tokens.len());

    println!("=== Tokens from position {} to {} (centered on error position 2171) ===\n", start, end);
    for i in start..end {
        let marker = if i == 2171 { " >>> ERROR HERE >>> " } else { "                    " };
        println!("{} [{}] {:?}", marker, i, tokens[i]);
    }

    if 2171 < tokens.len() {
        println!("\n=== Token at position 2171 (where error occurs) ===");
        println!("{:?}", tokens[2171]);
    }

    // Also show a wider context (±20 tokens)
    println!("\n\n=== WIDER CONTEXT (±20 tokens around 2171) ===\n");
    let wide_start = if 2171 > 20 { 2171 - 20 } else { 0 };
    let wide_end = std::cmp::min(2171 + 21, tokens.len());

    for i in wide_start..wide_end {
        let marker = if i == 2171 { " >>> ERROR >>> " } else { "               " };
        println!("{} [{}] {:?}", marker, i, tokens[i]);
    }
}
