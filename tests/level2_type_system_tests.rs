use llvm_rust::{Context, parse};
use std::time::Instant;

#[test]
fn test_parse_assembler_all_tests() {
    let test_dir = "/home/user/llvm-rust/llvm-tests/llvm/test/Assembler";

    let mut entries: Vec<_> = std::fs::read_dir(test_dir)
        .expect("Failed to read test directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "ll")
        })
        .collect();

    // Sort by filename for consistency
    entries.sort_by_key(|e| e.path());

    let test_count = entries.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped_negative = 0;
    let mut failures = Vec::new();

    println!("\n=== LEVEL 2: TYPE SYSTEM TESTS (all {} Assembler files) ===\n", test_count);

    for entry in &entries {
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

        // Skip negative tests
        let is_negative_test = content.contains("RUN: not") ||
                               content.contains("XFAIL") ||
                               filename.contains("invalid") ||
                               filename.contains("bad-") ||
                               filename.contains("-bad");

        if is_negative_test {
            skipped_negative += 1;
            continue;
        }

        let start = Instant::now();
        let ctx = Context::new();

        match parse(&content, ctx) {
            Ok(_module) => {
                println!("✓ {} ({:.2}s)", filename, start.elapsed().as_secs_f64());
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}: {:?}", filename, e);
                failed += 1;
                let error_msg = format!("{:?}", e);
                let error_short = if error_msg.len() > 200 {
                    format!("{}...", &error_msg[..200])
                } else {
                    error_msg
                };
                failures.push((filename.to_string(), error_short));
            }
        }
    }

    println!("\n=== LEVEL 2 RESULTS ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Skipped (negative tests): {}", skipped_negative);
    println!("Total tested: {}", passed + failed);
    println!("Total files: {}", test_count);

    if !failures.is_empty() {
        println!("\n=== FAILURES (first 30) ===");
        for (filename, error) in failures.iter().take(30) {
            println!("{}: {}", filename, error);
        }
    }

    let success_rate = if passed + failed > 0 {
        (passed as f64 / (passed + failed) as f64) * 100.0
    } else {
        0.0
    };
    println!("\nLevel 2 Success rate: {:.1}%", success_rate);
    println!("Target: 100.0%");

    if success_rate < 100.0 {
        println!("\n⚠️  {} files need fixing to reach 100%", failed);
    } else {
        println!("\n🎉 LEVEL 2 COMPLETE - 100% SUCCESS! 🎉");
    }
}
