use std::fs;

fn main() {
    let content = fs::read_to_string("llvm-tests/llvm-project/llvm/test/Assembler/amdgcn-unreachable.ll").unwrap();
    let mut lexer = llvm_rust::lexer::Lexer::new(&content);
    
    let mut position = 0;
    while position < 200 {
        match lexer.next_token() {
            Ok(token) => {
                if position >= 180 && position <= 195 {
                    println!("Position {}: {:?}", position, token);
                }
                position += 1;
            }
            Err(e) => {
                println!("Error at position {}: {}", position, e);
                break;
            }
        }
    }
}
