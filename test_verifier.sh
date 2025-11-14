#!/bin/bash
VERIFIER_DIR="llvm-tests/llvm-project/llvm/test/Verifier"
BINARY="./target/release/test_verify"

if [ ! -f "$BINARY" ]; then
    echo "Building test_verify..."
    cargo build --release --bin test_verify
fi

total=0
passed=0
failed=0

for file in "$VERIFIER_DIR"/*.ll; do
    filename=$(basename "$file")
    total=$((total + 1))
    
    # Determine if this is a negative test (should fail)
    is_negative=0
    if grep -q "RUN:.*not llvm-as" "$file" || grep -q "RUN:.*not opt" "$file"; then
        is_negative=1
    fi
    
    # Run the test
    if timeout 5 "$BINARY" "$file" > /dev/null 2>&1; then
        # Test passed (no errors)
        if [ $is_negative -eq 1 ]; then
            failed=$((failed + 1))
        else
            passed=$((passed + 1))
        fi
    else
        # Test failed (errors detected)
        if [ $is_negative -eq 1 ]; then
            passed=$((passed + 1))
        else
            failed=$((failed + 1))
        fi
    fi
done

echo "Verifier Tests: $passed/$total passing ($(echo "scale=1; $passed*100/$total" | bc)%)"
echo "Failed: $failed"
