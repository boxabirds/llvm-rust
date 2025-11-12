#!/bin/bash
# List all negative tests that incorrectly pass

TEST_DIR="llvm-tests/llvm-project/llvm/test/Assembler"
PARSER="./target/debug/test_parser"

for file in "$TEST_DIR"/*.ll; do
    filename=$(basename "$file")

    # Check if negative test
    if ! head -20 "$file" | grep -q "RUN:.*not.*llvm-as"; then
        continue
    fi

    # Check if parser incorrectly accepts it
    if timeout 2 "$PARSER" "$file" >/dev/null 2>&1; then
        echo "$filename"
    fi
done
