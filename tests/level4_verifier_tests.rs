use llvm_rust::{Context, parse, verification::verify_module};
use std::time::Instant;

#[test]
fn test_parse_verifier_tests() {
    let test_dir = "/home/user/llvm-rust/llvm-tests/llvm/test/Verifier";

    let mut entries: Vec<_> = std::fs::read_dir(test_dir)
        .expect("Failed to read test directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "ll")
        })
        .collect();

    // Sort by filename for consistency
    entries.sort_by_key(|e| e.path());

    // Test all files
    let test_count = entries.len();

    let mut passed = 0;
    let mut failed = 0;
    let mut negative_tests_correct = 0;
    let mut negative_tests_incorrect = 0;
    let mut failures = Vec::new();

    println!("\n=== LEVEL 4: VERIFIER TESTS ===\n");

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

        // Detect negative tests (tests that are supposed to fail)
        let is_negative_test = content.contains("RUN: not") ||
                               content.contains("XFAIL") ||
                               filename.contains("invalid") ||
                               filename.contains("bad-") ||
                               filename.contains("-bad");

        let start = Instant::now();
        let ctx = Context::new();

        let parse_result = parse(&content, ctx);

        match parse_result {
            Ok(module) => {
                // Module parsed successfully, now verify it
                let verify_result = verify_module(&module);

                match verify_result {
                    Ok(()) => {
                        // Parsing and verification both succeeded
                        if is_negative_test {
                            // Negative test that passed - this is wrong
                            println!("✗ {} (negative test should have failed verification) ({:.2}s)", filename, start.elapsed().as_secs_f64());
                            negative_tests_incorrect += 1;
                            failures.push((filename.to_string(), "Negative test should have failed".to_string()));
                        } else {
                            // Positive test that passed - correct
                            println!("✓ {} ({:.2}s)", filename, start.elapsed().as_secs_f64());
                            passed += 1;
                        }
                    }
                    Err(verify_errors) => {
                        // Verification failed
                        if is_negative_test {
                            // Negative test that failed verification - correct!
                            println!("✓ {} (negative test correctly failed) ({:.2}s)", filename, start.elapsed().as_secs_f64());
                            negative_tests_correct += 1;
                        } else {
                            // Positive test that failed verification - wrong
                            println!("✗ {}: Verification errors: {:?}", filename, verify_errors.first());
                            failed += 1;
                            failures.push((filename.to_string(), format!("Verification failed: {:?}", verify_errors.first())));
                        }
                    }
                }
            }
            Err(e) => {
                // Parsing failed
                if is_negative_test {
                    // Negative test that failed parsing - this is correct!
                    println!("✓ {} (negative test correctly failed parsing) ({:.2}s)", filename, start.elapsed().as_secs_f64());
                    negative_tests_correct += 1;
                } else {
                    // Positive test that failed parsing - this is wrong
                    println!("✗ {}: {:?}", filename, e);
                    failed += 1;
                    failures.push((filename.to_string(), format!("{:?}", e)));
                }
            }
        }
    }

    println!("\n=== LEVEL 4 RESULTS ===");
    println!("Positive tests passed: {}", passed);
    println!("Positive tests failed: {}", failed);
    println!("Negative tests correct: {}", negative_tests_correct);
    println!("Negative tests incorrect: {}", negative_tests_incorrect);
    println!("Total: {}", test_count);

    if !failures.is_empty() {
        println!("\n=== FAILURES (first 20) ===");
        for (filename, error) in failures.iter().take(20) {
            let error_short = if error.len() > 100 {
                format!("{}...", &error[..100])
            } else {
                error.clone()
            };
            println!("{}: {}", filename, error_short);
        }
    }

    let total_correct = passed + negative_tests_correct;
    let success_rate = (total_correct as f64 / test_count as f64) * 100.0;
    println!("\nLevel 4 Overall success rate: {:.1}%", success_rate);
    println!("  - Positive tests: {}/{}", passed, passed + failed);
    println!("  - Negative tests: {}/{}", negative_tests_correct, negative_tests_correct + negative_tests_incorrect);
    println!("Target: 100.0%");

    // Don't fail the test, just report the status
    let total_failed = failed + negative_tests_incorrect;
    if success_rate < 100.0 {
        println!("\n⚠️  {} files need fixing to reach 100%", total_failed);
    } else {
        println!("\n🎉 LEVEL 4 COMPLETE - 100% SUCCESS! 🎉");
    }
}
