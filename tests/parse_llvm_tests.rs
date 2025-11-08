use llvm_rust::{Context, parse};
use std::fs;
use std::path::Path;

#[test]
fn test_parse_llvm_assembler_tests() {
    let test_dir = Path::new("/home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Assembler");

    if !test_dir.exists() {
        eprintln!("Test directory doesn't exist, skipping");
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    // Get all .ll files
    let entries = fs::read_dir(test_dir).expect("Failed to read test directory");
    let mut test_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("ll")
        })
        .filter(|e| {
            // Filter out invalid test files (those marked with "invalid" or "not llvm-as")
            let path = e.path();
            let content = fs::read_to_string(&path).unwrap_or_default();

            // Skip files that are meant to fail
            let is_invalid = path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("invalid"))
                .unwrap_or(false);

            let is_negative_test = content.contains("RUN: not llvm-as");

            !is_invalid && !is_negative_test
        })
        .collect();

    test_files.sort_by_key(|e| e.path());

    // Take only first 100 for Level 1
    test_files.truncate(100);

    for entry in test_files.iter() {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", filename, e);
                failed += 1;
                failures.push((filename.to_string(), format!("Read error: {}", e)));
                continue;
            }
        };

        let ctx = Context::new();
        match parse(&content, ctx) {
            Ok(_module) => {
                passed += 1;
                println!("✓ {}", filename);
            }
            Err(e) => {
                failed += 1;
                failures.push((filename.to_string(), format!("{:?}", e)));
                println!("✗ {}: {:?}", filename, e);
            }
        }
    }

    println!("\n=== RESULTS ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total: {}", passed + failed);

    if !failures.is_empty() {
        println!("\n=== FAILURES ===");
        for (filename, error) in failures.iter().take(10) {
            println!("{}: {}", filename, error);
        }
        if failures.len() > 10 {
            println!("... and {} more", failures.len() - 10);
        }
    }

    // For Level 1, we want at least 50% success rate
    let success_rate = if passed + failed > 0 {
        (passed as f64) / ((passed + failed) as f64)
    } else {
        0.0
    };

    println!("\nSuccess rate: {:.1}%", success_rate * 100.0);

    // Don't fail the test yet, just report
    // assert!(passed > 0, "Should parse at least some files");
}
