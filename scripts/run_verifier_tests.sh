#!/bin/bash
# Run all Verifier tests and report results

TEST_DIR="llvm-tests/llvm-project/llvm/test/Verifier"
PARSER="./target/debug/test_parser"

passed=0
failed=0
total=0

echo "Running Verifier tests..."
echo

for file in "$TEST_DIR"/*.ll; do
    total=$((total + 1))
    filename=$(basename "$file")

    # Run the parser
    output=$("$PARSER" "$file" 2>&1)
    exit_code=$?

    # Check if this is a negative test (should fail)
    if grep -q "expected to fail\|invalid\|verify-\|bad-" <<< "$filename"; then
        # Negative test - should reject
        if [ $exit_code -ne 0 ]; then
            passed=$((passed + 1))
            echo "✓ $filename (correctly rejected)"
        else
            failed=$((failed + 1))
            echo "✗ $filename (incorrectly accepted)"
        fi
    else
        # Positive test - should parse
        if [ $exit_code -eq 0 ]; then
            passed=$((passed + 1))
            echo "✓ $filename"
        else
            failed=$((failed + 1))
            echo "✗ $filename: $output"
        fi
    fi
done

echo
echo "=== Summary ==="
echo "Total: $total"
echo "Passed: $passed"
echo "Failed: $failed"
echo "Pass rate: $(echo "scale=1; 100*$passed/$total" | bc)%"
