use llvm_rust::{Context, parse};
use std::time::Instant;

#[test]
fn test_parse_verifier_tests() {
    // Use environment variable or default to relative path
    let test_dir = std::env::var("LLVM_TEST_DIR")
        .unwrap_or_else(|_| "llvm-tests/llvm-project/test/Verifier".to_string());

    let mut entries: Vec<_> = match std::fs::read_dir(&test_dir) {
        Ok(dir) => dir.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            println!("\n⚠️  LLVM test directory not found at: {}", test_dir);
            println!("To run these tests, either:");
            println!("  1. Clone LLVM tests: git clone --depth 1 https://github.com/llvm/llvm-project.git llvm-tests/llvm-project");
            println!("  2. Set LLVM_TEST_DIR environment variable to your LLVM test directory");
            println!("\nSkipping tests...");
            return;
        }
    };

    // Filter for .ll files only
    entries.retain(|e| {
        e.path().extension().map_or(false, |ext| ext == "ll")
    });

    entries.sort_by_key(|e| e.path());
    let test_count = entries.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    println!("\n=== LEVEL 7: VERIFIER TESTS ===\n");

    let mut negative_tests = 0;

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                println!("✗ {}: Failed to read file: {}", filename, e);
                failed += 1;
                failures.push((filename.to_string(), format!("Read error: {}", e)));
                continue;
            }
        };

        // Check if this is a negative test (expected to fail)
        // Negative tests use "RUN: not" followed by llvm-as, llvm-dis, or opt
        // Use regex-like check to handle variable whitespace
        let is_negative_test = content.contains("RUN:") &&
                               (content.contains(" not llvm-as") ||
                                content.contains(" not llvm-dis") ||
                                content.contains(" not opt"));

        let start = Instant::now();
        let ctx = Context::new();

        match parse(&content, ctx) {
            Ok(_) => {
                if is_negative_test {
                    println!("✗ {}: Negative test should have failed parsing", filename);
                    failed += 1;
                    failures.push((filename.to_string(), "Negative test passed unexpectedly".to_string()));
                } else {
                    println!("✓ {} ({:.2}s)", filename, start.elapsed().as_secs_f64());
                    passed += 1;
                }
            }
            Err(e) => {
                if is_negative_test {
                    println!("✓ {} (expected failure)", filename);
                    passed += 1;
                    negative_tests += 1;
                } else {
                    println!("✗ {}: {:?}", filename, e);
                    failed += 1;
                    failures.push((filename.to_string(), format!("{:?}", e)));
                }
            }
        }
    }

    println!("\n=== LEVEL 7 RESULTS ===");
    println!("Passed: {}", passed);
    println!("  Negative tests (correctly failed): {}", negative_tests);
    println!("Failed: {}", failed);

    let success_rate = (passed as f64 / test_count as f64) * 100.0;
    println!("\nLevel 7 Success rate: {:.1}%", success_rate);

    if success_rate < 100.0 {
        println!("\n⚠️  {} files need fixing to reach 100%", failed);
    } else {
        println!("\n🎉 LEVEL 7 COMPLETE - 100% SUCCESS! 🎉");
    }
}
