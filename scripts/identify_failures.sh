#!/bin/bash
# Script to identify specific failing LLVM test files

set -euo pipefail

TEST_DIR="llvm-tests/llvm-project/llvm/test/Assembler"
FAILURES_FILE="test_failures.txt"
SUCCESS_FILE="test_successes.txt"

if [ ! -d "$TEST_DIR" ]; then
    echo "Error: Test directory not found: $TEST_DIR"
    exit 1
fi

echo "Testing LLVM Assembler files..."
echo "Output will be saved to $FAILURES_FILE and $SUCCESS_FILE"
echo ""

# Clear previous results
> "$FAILURES_FILE"
> "$SUCCESS_FILE"

total=0
passed=0
failed=0

for file in "$TEST_DIR"/*.ll; do
    if [ ! -f "$file" ]; then
        continue
    fi

    total=$((total + 1))
    filename=$(basename "$file")

    # Try to parse the file using our parser test binary
    if timeout 5 cargo run --quiet --bin test_parser "$file" 2>/dev/null >/dev/null; then
        passed=$((passed + 1))
        echo "$filename" >> "$SUCCESS_FILE"
    else
        failed=$((failed + 1))
        echo "$filename" >> "$FAILURES_FILE"
        echo "FAIL: $filename"
    fi

    # Progress indicator
    if [ $((total % 50)) -eq 0 ]; then
        echo "Progress: $total files tested, $passed passed, $failed failed"
    fi
done

echo ""
echo "=== Final Results ==="
echo "Total: $total"
echo "Passed: $passed ($((passed * 100 / total))%)"
echo "Failed: $failed ($((failed * 100 / total))%)"
echo ""
echo "Failed files saved to: $FAILURES_FILE"
echo "Successful files saved to: $SUCCESS_FILE"
