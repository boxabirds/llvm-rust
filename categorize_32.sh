#!/bin/bash
cd /home/user/llvm-rust/llvm-tests/llvm-project/llvm/test/Assembler

for f in 2002-12-15-GlobalResolve.ll 2003-04-15-ConstantInitAssertion.ll 2003-05-21-MalformedStructCrash.ll 2004-03-30-UnclosedFunctionCrash.ll alloca-addrspace-parse-error-0.ll alloca-invalid-type-2.ll alloca-invalid-type.ll byref-parse-error-1.ll byref-parse-error-5.ll byref-parse-error-6.ll byref-parse-error-7.ll call-invalid-1.ll captures-errors.ll constant-splat-diagnostics.ll getelementptr_invalid_ptr.ll initializes-attribute-invalid.ll inrange-errors.ll invalid-c-style-comment2.ll invalid-gep-missing-explicit-type.ll invalid-hexint.ll invalid-inline-constraint.ll invalid-ptrauth-const1.ll invalid-ptrauth-const2.ll invalid-ptrauth-const3.ll invalid-ptrauth-const4.ll invalid-ptrauth-const5.ll invalid-safestack-return.ll invalid-uselistorder-function-between-blocks.ll mustprogress-parse-error-1.ll nofpclass-invalid.ll ptrauth-const.ll unsupported-constexprs.ll; do
  first_line=$(head -1 "$f")
  if echo "$first_line" | grep -q "not llvm-as"; then
    echo "NEGATIVE: $f"
  elif echo "$first_line" | grep -q "llvm-as"; then
    echo "POSITIVE: $f"
  else
    echo "UNKNOWN: $f"
  fi
done | sort
