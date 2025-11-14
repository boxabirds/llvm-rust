#!/bin/bash
set -e

# Script to run LLVM test suite files through our parser
# Positive tests should parse successfully
# Negative tests (with "not llvm-as") should fail to parse

LLVM_TEST_DIR="llvm-tests/llvm/test/Assembler"
RESULTS_FILE="llvm_test_results.txt"
AUDIT_FILE="docs/20251114-0406-test-audit.md"

# Clear previous results
> "$RESULTS_FILE"

echo "Running LLVM Assembler test suite..."
echo "======================================"

TOTAL=0
PASSED=0
FAILED=0
POSITIVE_PASSED=0
POSITIVE_FAILED=0
NEGATIVE_PASSED=0
NEGATIVE_FAILED=0

# Arrays to store failing tests
POSITIVE_FAILURES=()
NEGATIVE_FAILURES=()

for test_file in "$LLVM_TEST_DIR"/*.ll; do
    TOTAL=$((TOTAL + 1))
    test_name=$(basename "$test_file")

    # Check if this is a negative test (expected to fail)
    is_negative=0
    if grep -q "RUN:.*not.*llvm-as" "$test_file" 2>/dev/null; then
        is_negative=1
    fi

    # Run the parser via our test binary
    if cargo run --bin test_parser -- "$test_file" > /dev/null 2>&1; then
        parse_success=1
    else
        parse_success=0
    fi

    # Check if result matches expectation
    if [ $is_negative -eq 1 ]; then
        # Negative test - should fail to parse
        if [ $parse_success -eq 0 ]; then
            NEGATIVE_PASSED=$((NEGATIVE_PASSED + 1))
            PASSED=$((PASSED + 1))
            echo "✓ NEGATIVE: $test_name" >> "$RESULTS_FILE"
        else
            NEGATIVE_FAILED=$((NEGATIVE_FAILED + 1))
            FAILED=$((FAILED + 1))
            NEGATIVE_FAILURES+=("$test_name")
            echo "✗ NEGATIVE (parsed but should fail): $test_name" >> "$RESULTS_FILE"
        fi
    else
        # Positive test - should parse successfully
        if [ $parse_success -eq 1 ]; then
            POSITIVE_PASSED=$((POSITIVE_PASSED + 1))
            PASSED=$((PASSED + 1))
            echo "✓ POSITIVE: $test_name" >> "$RESULTS_FILE"
        else
            POSITIVE_FAILED=$((POSITIVE_FAILED + 1))
            FAILED=$((FAILED + 1))
            POSITIVE_FAILURES+=("$test_name")
            echo "✗ POSITIVE (failed but should parse): $test_name" >> "$RESULTS_FILE"
        fi
    fi

    # Progress indicator every 10 tests
    if [ $((TOTAL % 10)) -eq 0 ]; then
        echo "Progress: $TOTAL/$TOTAL tests processed..."
    fi
done

# Print summary
echo ""
echo "======================================"
echo "LLVM Test Suite Results"
echo "======================================"
echo "Total tests: $TOTAL"
echo "Passed: $PASSED ($((PASSED * 100 / TOTAL))%)"
echo "Failed: $FAILED ($((FAILED * 100 / TOTAL))%)"
echo ""
echo "Positive tests (should parse):"
echo "  Passed: $POSITIVE_PASSED"
echo "  Failed: $POSITIVE_FAILED"
echo ""
echo "Negative tests (should fail):"
echo "  Passed: $NEGATIVE_PASSED"
echo "  Failed: $NEGATIVE_FAILED"
echo ""

if [ $FAILED -gt 0 ]; then
    echo "Failing positive tests:"
    for test in "${POSITIVE_FAILURES[@]}"; do
        echo "  - $test"
    done
    echo ""
    echo "Failing negative tests:"
    for test in "${NEGATIVE_FAILURES[@]}"; do
        echo "  - $test"
    done
fi

# Create audit document
cat > "$AUDIT_FILE" <<EOF
# LLVM Test Suite Audit - $(date '+%Y-%m-%d %H:%M:%S')

## Overview

This document tracks the status of running the LLVM Assembler test suite against the llvm-rust parser.

## Summary Statistics

- **Total Tests**: $TOTAL
- **Passed**: $PASSED ($((PASSED * 100 / TOTAL))%)
- **Failed**: $FAILED ($((FAILED * 100 / TOTAL))%)

### Breakdown by Type

#### Positive Tests (Expected to Parse Successfully)
- **Passed**: $POSITIVE_PASSED
- **Failed**: $POSITIVE_FAILED

#### Negative Tests (Expected to Fail Parsing)
- **Passed**: $NEGATIVE_PASSED
- **Failed**: $NEGATIVE_FAILED

## Failing Tests

### Positive Tests (Parser should accept but currently rejects)

EOF

if [ ${#POSITIVE_FAILURES[@]} -eq 0 ]; then
    echo "None - all positive tests passing! ✓" >> "$AUDIT_FILE"
else
    for test in "${POSITIVE_FAILURES[@]}"; do
        echo "- \`$test\`" >> "$AUDIT_FILE"
    done
fi

cat >> "$AUDIT_FILE" <<EOF

### Negative Tests (Parser should reject but currently accepts)

EOF

if [ ${#NEGATIVE_FAILURES[@]} -eq 0 ]; then
    echo "None - all negative tests passing! ✓" >> "$AUDIT_FILE"
else
    for test in "${NEGATIVE_FAILURES[@]}"; do
        echo "- \`$test\`" >> "$AUDIT_FILE"
    done
fi

cat >> "$AUDIT_FILE" <<EOF

## Next Steps

EOF

if [ $FAILED -eq 0 ]; then
    echo "All tests passing! The parser successfully handles all LLVM Assembler test cases." >> "$AUDIT_FILE"
else
    cat >> "$AUDIT_FILE" <<EOF
1. Fix failing positive tests (highest priority)
   - These are valid LLVM IR that our parser should accept
   - Review parser errors and update grammar/validation

2. Fix failing negative tests
   - These are invalid LLVM IR that our parser should reject
   - Add or improve validation rules

3. Re-run audit to verify fixes
EOF
fi

echo ""
echo "Audit document created: $AUDIT_FILE"
echo "Detailed results: $RESULTS_FILE"
