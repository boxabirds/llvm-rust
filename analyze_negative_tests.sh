#!/bin/bash

# Analyze patterns in failing negative tests
echo "Analyzing failing negative tests..."

# Get list of negative tests we're incorrectly accepting
NEGATIVE_TESTS=(
    "2003-11-11-ImplicitRename.ll"
    "2004-11-28-InvalidTypeCrash.ll"
    "invalid-immarg2.ll"
    "invalid-immarg3.ll"
    "invalid-immarg5.ll"
    "invalid-inttype.ll"
    "invalid-label-call-arg.ll"
)

for test in "${NEGATIVE_TESTS[@]}"; do
    echo "=== $test ==="
    echo "Expected: Should FAIL (negative test)"
    echo "First few lines:"
    head -10 "llvm-tests/llvm/test/Assembler/$test"
    echo ""
done
