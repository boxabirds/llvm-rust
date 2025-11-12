#!/bin/bash
# Categorize LLVM test failures

TEST_DIR="llvm-tests/llvm-project/llvm/test/Assembler"

# Arrays for categorization
negative_tests=()
debug_info_tests=()
target_specific_tests=()
timeout_tests=()
other_failures=()

echo "Categorizing test failures..."
echo ""

for file in "$TEST_DIR"/*.ll; do
    filename=$(basename "$file")

    # Check if it's a negative test (RUN: not llvm-as)
    if head -5 "$file" | grep -q "RUN: not llvm-as"; then
        negative_tests+=("$filename")
        continue
    fi

    # Quick parse test with short timeout
    if timeout 1 ./target/debug/test_parser "$file" 2>/dev/null; then
        # Success - skip
        continue
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            # Timeout
            timeout_tests+=("$filename")
        elif [[ "$filename" == DI* ]] || [[ "$filename" == *debug* ]] || [[ "$filename" == *dbg* ]]; then
            debug_info_tests+=("$filename")
        elif [[ "$filename" == *amdgpu* ]] || [[ "$filename" == *amdgcn* ]] || [[ "$filename" == *nvptx* ]]; then
            target_specific_tests+=("$filename")
        else
            other_failures+=("$filename")
        fi
    fi
done

echo "=== Test Failure Categories ==="
echo ""

echo "1. Negative Tests (EXPECTED to fail - test error handling):"
echo "   Count: ${#negative_tests[@]}"
if [ ${#negative_tests[@]} -gt 0 ]; then
    printf '   - %s\n' "${negative_tests[@]::5}"
    if [ ${#negative_tests[@]} -gt 5 ]; then
        echo "   ... and $((${#negative_tests[@]} - 5)) more"
    fi
fi
echo ""

echo "2. Parser Timeouts/Hangs (BUGS - infinite loops):"
echo "   Count: ${#timeout_tests[@]}"
if [ ${#timeout_tests[@]} -gt 0 ]; then
    printf '   - %s\n' "${timeout_tests[@]}"
fi
echo ""

echo "3. Debug Info Tests (Missing metadata support):"
echo "   Count: ${#debug_info_tests[@]}"
if [ ${#debug_info_tests[@]} -gt 0 ]; then
    printf '   - %s\n' "${debug_info_tests[@]::5}"
    if [ ${#debug_info_tests[@]} -gt 5 ]; then
        echo "   ... and $((${#debug_info_tests[@]} - 5)) more"
    fi
fi
echo ""

echo "4. Target-Specific Tests (AMDGPU, NVPTX, etc.):"
echo "   Count: ${#target_specific_tests[@]}"
if [ ${#target_specific_tests[@]} -gt 0 ]; then
    printf '   - %s\n' "${target_specific_tests[@]}"
fi
echo ""

echo "5. Other Parse Failures:"
echo "   Count: ${#other_failures[@]}"
if [ ${#other_failures[@]} -gt 0 ]; then
    printf '   - %s\n' "${other_failures[@]::10}"
    if [ ${#other_failures[@]} -gt 10 ]; then
        echo "   ... and $((${#other_failures[@]} - 10)) more"
    fi
fi
echo ""

echo "=== Summary ==="
real_bugs=$((${#timeout_tests[@]} + ${#other_failures[@]}))
echo "Negative tests (expected failures): ${#negative_tests[@]}"
echo "Real bugs/missing features: $real_bugs"
echo "  - Timeouts/hangs: ${#timeout_tests[@]}"
echo "  - Missing debug info: ${#debug_info_tests[@]}"
echo "  - Target-specific: ${#target_specific_tests[@]}"
echo "  - Other: ${#other_failures[@]}"
