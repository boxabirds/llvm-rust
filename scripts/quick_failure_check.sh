#!/bin/bash
# Script to test LLVM Assembler files, properly handling both positive and negative tests

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
positive_tests=0
negative_tests=0
positive_passed=0
negative_passed=0
failures=()

for file in "$TEST_DIR"/*.ll; do
    if [ ! -f "$file" ]; then
        continue
    fi

    total=$((total + 1))
    filename=$(basename "$file")

    # Check if this is a negative test (should fail to parse)
    is_negative=0
    if head -20 "$file" | grep -qE "RUN:.*not.*(llvm-as|opt|llc)"; then
        is_negative=1
        negative_tests=$((negative_tests + 1))
    else
        positive_tests=$((positive_tests + 1))
    fi

    # Run the parser
    if timeout 2 "$PARSER" "$file" >/dev/null 2>&1; then
        parser_succeeded=1
    else
        parser_succeeded=0
    fi

    # Determine if test passed based on expectation
    if [ $is_negative -eq 1 ]; then
        # Negative test: should fail
        if [ $parser_succeeded -eq 0 ]; then
            passed=$((passed + 1))
            negative_passed=$((negative_passed + 1))
        else
            failed=$((failed + 1))
            failures+=("$filename (NEGATIVE TEST - should have failed)")
            if [ $failed -le 20 ]; then
                echo "FAIL: $filename (negative test incorrectly accepted)"
            fi
        fi
    else
        # Positive test: should succeed
        if [ $parser_succeeded -eq 1 ]; then
            passed=$((passed + 1))
            positive_passed=$((positive_passed + 1))
        else
            failed=$((failed + 1))
            failures+=("$filename (POSITIVE TEST - should have passed)")
            if [ $failed -le 20 ]; then
                echo "FAIL: $filename (positive test incorrectly rejected)"
            fi
        fi
    fi
done

echo ""
echo "=== Results ==="
echo "Total: $total"
echo "Passed: $passed ($(echo "scale=1; $passed * 100 / $total" | bc)%)"
echo "Failed: $failed ($(echo "scale=1; $failed * 100 / $total" | bc)%)"
echo ""
echo "Positive tests (should parse): $positive_passed/$positive_tests passed"
echo "Negative tests (should reject): $negative_passed/$negative_tests passed"

if [ $failed -gt 0 ]; then
    echo ""
    echo "=== All Failures (total: $failed) ==="
    printf '%s\n' "${failures[@]}"
fi
