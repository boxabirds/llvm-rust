use llvm_rust::{Context, parse, verification::verify_module};
use std::fs;
use std::path::Path;

#[test]
fn test_level4_verifier() {
    let test_dir = Path::new("/home/user/llvm-rust/llvm-tests/llvm/test/Verifier");

    if !test_dir.exists() {
        eprintln!("Test directory doesn't exist, skipping");
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut positive_passed = 0;
    let mut positive_failed = 0;
    let mut negative_passed = 0;
    let mut negative_failed = 0;
    let mut failures = Vec::new();

    // Get all .ll files
    let entries = fs::read_dir(test_dir).expect("Failed to read test directory");
    let mut test_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("ll"))
        .collect();

    test_files.sort_by_key(|e| e.path());

    for entry in test_files.iter() {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", filename, e);
                failed += 1;
                failures.push((filename.to_string(), format!("Read error: {}", e), false));
                continue;
            }
        };

        // Determine if this is a negative test (should fail verification)
        let is_negative = content.contains("RUN: not llvm-as") ||
                         content.contains("RUN: not opt");

        let ctx = Context::new();
        match parse(&content, ctx) {
            Ok(module) => {
                match verify_module(&module) {
                    Ok(()) => {
                        if is_negative {
                            // Negative test should have failed but passed
                            failed += 1;
                            negative_failed += 1;
                            failures.push((filename.to_string(), "Should have failed verification but passed".to_string(), true));
                            println!("✗ {} (negative test passed verification, should have failed)", filename);
                        } else {
                            // Positive test passed
                            passed += 1;
                            positive_passed += 1;
                            println!("✓ {} (positive)", filename);
                        }
                    }
                    Err(errors) => {
                        if is_negative {
                            // Negative test correctly failed
                            passed += 1;
                            negative_passed += 1;
                            println!("✓ {} (negative, correctly failed with {} errors)", filename, errors.len());
                        } else {
                            // Positive test incorrectly failed
                            failed += 1;
                            positive_failed += 1;
                            let error_msg = format!("{} verification errors", errors.len());
                            failures.push((filename.to_string(), error_msg, false));
                            println!("✗ {} (positive test failed verification): {:?}", filename, errors.first());
                        }
                    }
                }
            }
            Err(e) => {
                // Parse failed
                // Check if this is a verification error during parsing
                let error_str = format!("{:?}", e);
                let is_verification_error = error_str.contains("Verification failed:");

                if is_negative && is_verification_error {
                    // Negative test failed during parse due to verification - this is correct!
                    passed += 1;
                    negative_passed += 1;
                    println!("✓ {} (negative, correctly failed during parse)", filename);
                } else {
                    // Either positive test failed, or negative test failed for wrong reason
                    failed += 1;
                    if is_negative {
                        negative_failed += 1;
                    } else {
                        positive_failed += 1;
                    }
                    let error_msg = format!("Parse error: {:?}", e);
                    failures.push((filename.to_string(), error_msg, is_negative));
                    println!("✗ {} (parse failed): {:?}", filename, e);
                }
            }
        }
    }

    println!("\n=== LEVEL 4 RESULTS ===");
    println!("Total: {}/{} ({:.1}%)", passed, passed + failed,
             (passed as f64) / ((passed + failed) as f64) * 100.0);
    println!("Positive tests: {}/{} ({:.1}%)", positive_passed, positive_passed + positive_failed,
             if positive_passed + positive_failed > 0 { (positive_passed as f64) / ((positive_passed + positive_failed) as f64) * 100.0 } else { 0.0 });
    println!("Negative tests: {}/{} ({:.1}%)", negative_passed, negative_passed + negative_failed,
             if negative_passed + negative_failed > 0 { (negative_passed as f64) / ((negative_passed + negative_failed) as f64) * 100.0 } else { 0.0 });

    if !failures.is_empty() {
        println!("\n=== FAILURES ({}) ===", failures.len());
        for (filename, error, is_negative) in failures.iter().take(20) {
            let test_type = if *is_negative { "NEG" } else { "POS" };
            println!("[{}] {}: {}", test_type, filename, error);
        }
        if failures.len() > 20 {
            println!("... and {} more", failures.len() - 20);
        }
    }
}
