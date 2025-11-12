#!/bin/bash
cd /home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Assembler

echo "Analyzing LLVM test suite..."
echo

total=$(ls -1 *.ll 2>/dev/null | wc -l)
negative=$(grep -l "RUN:.*not.*llvm-as" *.ll 2>/dev/null | wc -l)
positive=$((total - negative))

echo "Test breakdown:"
echo "- Total tests: $total"
echo "- Positive tests (valid IR, should parse): $positive"
echo "- Negative tests (invalid IR, should reject): $negative"
echo

echo "Running test suite..."
cd /home/user/llvm-rust
./scripts/quick_failure_check.sh 2>&1 | grep -A 5 "=== Results ==="
