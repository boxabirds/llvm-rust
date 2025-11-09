use llvm_rust::{Context, parse};
use std::time::Instant;

#[test]
fn test_parse_verifier_tests() {
    let test_dir = "/home/user/llvm-rust/llvm-tests/llvm-project/test/Verifier";

    let mut entries: Vec<_> = std::fs::read_dir(test_dir)
        .expect("Failed to read test directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "ll")
        })
        .collect();

    entries.sort_by_key(|e| e.path());
    let test_count = entries.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    println!("\n=== LEVEL 7: VERIFIER TESTS ===\n");

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

        let start = Instant::now();
        let ctx = Context::new();

        match parse(&content, ctx) {
            Ok(_) => {
                println!("✓ {} ({:.2}s)", filename, start.elapsed().as_secs_f64());
                passed += 1;
            }
            Err(e) => {
                println!("✗ {}: {:?}", filename, e);
                failed += 1;
                failures.push((filename.to_string(), format!("{:?}", e)));
            }
        }
    }

    println!("\n=== LEVEL 7 RESULTS ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    let success_rate = (passed as f64 / test_count as f64) * 100.0;
    println!("\nLevel 7 Success rate: {:.1}%", success_rate);

    if success_rate < 100.0 {
        println!("\n⚠️  {} files need fixing to reach 100%", failed);
    } else {
        println!("\n🎉 LEVEL 7 COMPLETE - 100% SUCCESS! 🎉");
    }
}
