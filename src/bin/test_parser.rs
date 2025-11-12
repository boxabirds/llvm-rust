/// Simple binary to test parsing LLVM IR files
/// Exits with 0 on success, 1 on failure

use llvm_rust::{Context, parse};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: test_parser <file.ll>");
        process::exit(1);
    }

    let file_path = &args[1];

    // Read the file
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    };

    // Parse the file with verification
    let ctx = Context::new();
    match parse(&content, ctx) {
        Ok(_module) => {
            // Success - silent exit
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            process::exit(1);
        }
    }
}
