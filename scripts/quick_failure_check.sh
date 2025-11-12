#!/bin/bash
# Quick script to identify failing LLVM test files

TEST_DIR="llvm-tests/llvm-project/llvm/test/Assembler"
PARSER="./target/debug/test_parser"

if [ ! -f "$PARSER" ]; then
    echo "Building test_parser..."
    cargo build --quiet --bin test_parser
fi

echo "Checking all LLVM Assembler test files..."
echo ""

total=0
passed=0
failed=0
failures=()

for file in "$TEST_DIR"/*.ll; do
    if [ ! -f "$file" ]; then
        continue
    fi

    total=$((total + 1))
    filename=$(basename "$file")

    if timeout 2 "$PARSER" "$file" 2>/dev/null; then
        passed=$((passed + 1))
    else
        failed=$((failed + 1))
        failures+=("$filename")
        # Only show first 20 failures inline
        if [ $failed -le 20 ]; then
            echo "FAIL: $filename"
        fi
    fi
done

echo ""
echo "=== Results ==="
echo "Total: $total"
echo "Passed: $passed ($(echo "scale=1; $passed * 100 / $total" | bc)%)"
echo "Failed: $failed ($(echo "scale=1; $failed * 100 / $total" | bc)%)"

if [ $failed -gt 0 ]; then
    echo ""
    echo "=== All Failures (total: $failed) ==="
    printf '%s\n' "${failures[@]}"
fi
