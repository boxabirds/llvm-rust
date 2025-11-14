#!/bin/bash

# Analyze the failing positive tests
echo "Analyzing failing positive tests..."
echo "===================================="
echo ""

FAILING_TESTS=(
    "2004-02-27-SelfUseAssertError.ll"
    "ConstantExprNoFold.ll"
    "alias-use-list-order.ll"
    "amdgcn-unreachable.ll"
    "amdgpu-image-atomic-attributes.ll"
    "atomic.ll"
    "diexpression.ll"
    "fast-math-flags.ll"
    "getelementptr.ll"
    "immarg-param-attribute.ll"
    "invalid-vecreduce.ll"
    "target-types.ll"
    "uselistorder.ll"
)

for test in "${FAILING_TESTS[@]}"; do
    echo "Test: $test"
    echo "---"
    error=$(cargo run --quiet --bin test_parser -- "llvm-tests/llvm/test/Assembler/$test" 2>&1 | grep "Parse error" || echo "No error message")
    # Truncate very long errors
    if [ ${#error} -gt 200 ]; then
        echo "${error:0:200}..."
    else
        echo "$error"
    fi
    echo ""
done
