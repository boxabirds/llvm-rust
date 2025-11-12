#!/bin/bash
# Comprehensive test report for Levels 1-9

echo "================================================================"
echo "LLVM-Rust Test Report - Levels 1-9"
echo "================================================================"
echo ""

# Function to run tests and count results
run_test_suite() {
    local name="$1"
    local command="$2"
    echo "Testing: $name"
    echo "----------------------------------------"

    result=$(eval "$command" 2>&1)
    passed=$(echo "$result" | grep "test result:" | grep -oP '\d+ passed' | grep -oP '\d+' || echo "0")
    failed=$(echo "$result" | grep "test result:" | grep -oP '\d+ failed' | grep -oP '\d+' || echo "0")
    ignored=$(echo "$result" | grep "test result:" | grep -oP '\d+ ignored' | grep -oP '\d+' || echo "0")

    echo "  Passed: $passed"
    echo "  Failed: $failed"
    echo "  Ignored: $ignored"
    echo ""

    # Return values for accumulation
    echo "$passed $failed $ignored"
}

# Level 1-3: Parsing Tests
echo "LEVEL 1-3: Parsing & Type System"
echo "================================================================"
parsing_result=$(run_test_suite "Parser unit tests" "cargo test --lib parser::tests 2>&1")
type_result=$(run_test_suite "Type system tests" "cargo test --lib types::tests 2>&1")
pointer_result=$(run_test_suite "Pointer parsing" "cargo test --test complex_pointer_parsing_tests 2>&1")

# Level 4: Verification
echo "LEVEL 4: Verification"
echo "================================================================"
type_check_result=$(run_test_suite "Type checking" "cargo test --test type_checking_tests 2>&1")
metadata_result=$(run_test_suite "Metadata validation" "cargo test --test metadata_validation_tests 2>&1")
cfg_result=$(run_test_suite "CFG validation" "cargo test --test cfg_landingpad_validation_tests 2>&1")

# Level 5: Optimizations
echo "LEVEL 5: Optimizations"
echo "================================================================"
const_fold_result=$(run_test_suite "Constant folding" "cargo test --test constant_folding_tests 2>&1")
dce_result=$(run_test_suite "Dead code elimination" "cargo test --test dce_tests 2>&1")
instcombine_result=$(run_test_suite "Instruction combining" "cargo test --test instruction_combining_tests 2>&1")
pass_reg_result=$(run_test_suite "Pass registry" "cargo test --test pass_registry_tests 2>&1")

# Level 6: CFG & Analysis
echo "LEVEL 6: CFG & Analysis"
echo "================================================================"
analysis_result=$(run_test_suite "Analysis tests" "cargo test --lib analysis::tests 2>&1")
cfg_lib_result=$(run_test_suite "CFG tests" "cargo test --lib cfg::tests 2>&1")

# Level 7: Codegen
echo "LEVEL 7: x86-64 Codegen"
echo "================================================================"
codegen_result=$(run_test_suite "Codegen integration" "cargo test --test codegen_integration_test 2>&1")
stack_result=$(run_test_suite "Stack frame tests" "cargo test --lib codegen::stack_frame::tests 2>&1")
reg_alloc_result=$(run_test_suite "Register allocation" "cargo test --lib codegen::register_allocator::tests 2>&1")

# Level 8-9: Linking & Execution
echo "LEVEL 8-9: Linking & Execution"
echo "================================================================"
end_to_end_result=$(run_test_suite "End-to-end tests" "cargo test --test end_to_end_test 2>&1")
linker_result=$(run_test_suite "Linker tests" "cargo test --lib codegen::linker::tests 2>&1")
runtime_result=$(run_test_suite "Runtime tests" "cargo test --lib codegen::runtime::tests 2>&1")
external_result=$(run_test_suite "External functions" "cargo test --lib codegen::external_functions::tests 2>&1")

# Summary
echo "================================================================"
echo "SUMMARY"
echo "================================================================"
echo ""
echo "All integration tests completed!"
echo "See above for detailed results per level."
