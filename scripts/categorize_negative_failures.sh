#!/bin/bash
# Categorize what kinds of validation errors we're missing

TEST_DIR="llvm-tests/llvm-project/llvm/test/Assembler"
PARSER="./target/debug/test_parser"

echo "Analyzing negative tests that incorrectly pass..."
echo

# Categories based on test names and content patterns
undefined_type=0
type_conflict=0
alignment_error=0
visibility_error=0
attribute_error=0
parse_error=0
other=0

for file in "$TEST_DIR"/*.ll; do
    filename=$(basename "$file")

    # Check if negative test
    if ! head -20 "$file" | grep -q "RUN:.*not.*llvm-as"; then
        continue
    fi

    # Check if parser incorrectly accepts it
    if timeout 2 "$PARSER" "$file" >/dev/null 2>&1; then
        # Categorize based on filename and content
        if echo "$filename" | grep -qi "type\|undefined"; then
            ((undefined_type++))
        elif echo "$filename" | grep -qi "align"; then
            ((alignment_error++))
        elif echo "$filename" | grep -qi "visibility\|hidden\|protected\|private"; then
            ((visibility_error++))
        elif echo "$filename" | grep -qi "attr\|attribute"; then
            ((attribute_error++))
        elif echo "$filename" | grep -qi "parse"; then
            ((parse_error++))
        else
            ((other++))
        fi
    fi
done

total=$((undefined_type + type_conflict + alignment_error + visibility_error + attribute_error + parse_error + other))

echo "Negative tests incorrectly accepted (total: $total)"
echo "  Undefined/invalid types: $undefined_type"
echo "  Alignment errors: $alignment_error"
echo "  Visibility/linkage errors: $visibility_error"
echo "  Attribute errors: $attribute_error"
echo "  Parse errors: $parse_error"
echo "  Other validation: $other"
