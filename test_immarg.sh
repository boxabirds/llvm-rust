#!/bin/bash
for test in invalid-immarg.ll invalid-immarg2.ll invalid-immarg3.ll; do
  echo "=== $test ==="
  cargo run --quiet --bin test_parser -- "llvm-tests/llvm/test/Assembler/$test" 2>&1 | grep "Parse error" || echo "STILL ACCEPTING"
done
