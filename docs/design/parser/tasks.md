# Parser Enhancement Tasks

This document breaks down the parser enhancement design into trackable, dependency-ordered tasks. Each task is linked to specific LLVM test suite tests it will enable.

**Current Status**: 200/338 tests passing (59.2%)
**Target**: 338/338 tests passing (100%)

**Session Summary** (200 → 200, test infrastructure improvements):
- ✅ Fixed test runner negative test detection (127 → 128 negative tests)
- ✅ Test runner now handles whitespace variations in RUN directives
- ✅ Added unsized type load validation (unsized-types-load.ll passing)
- ✅ Added contains_scalable_type() helper (scaffolding for future)
- ⚠️ Net neutral on test count due to improved test classification

**Previous Session** (194 → 200, +6 tests, +1.7%):
- ✅ Operand bundle integration COMPLETE
- ✅ deoptimize-intrinsic.ll and guard-intrinsic.ll passing
- ✅ Alias verification, writable-attr, x86_intr, array preservation

**Major Milestones**:
- ✅ **OPERAND BUNDLE INTEGRATION COMPLETE!**
  - Refactored parse_instruction_operands() return type
  - Parsed bundles attached to Call/Invoke instructions
  - Added deoptimize/guard intrinsic validation
  - Foundation for ~18 more bundle tests
- ✅ All planned calling conventions added (11 new CCs)
- ✅ Extended AMDGPU CC call restrictions

**Phase Status**:
- Phase 1: 9/15 tasks done, need +9 tests for target (209/338)
- Phase 2: Ready to begin (operand bundles ✅, metadata next)

---

## Phase 1: Foundational Parser Enhancements (Target: +15 tests → 209/338)

### 1.1 Alias Support Enhancement

- [x] **Task 1.1.1**: Add `aliases` field to Module struct (already existed)
- [x] **Task 1.1.2**: Store parsed aliases in module during parsing (already existed)
- [x] **Task 1.1.3**: Add `get_alias()` method to Module (already existed)
- [x] **Task 1.1.4**: Implement alias cycle detection in verifier (COMPLETED)
- [x] **Task 1.1.5**: Implement alias-to-declaration validation (COMPLETED)
- [x] **Task 1.1.6**: Implement interposable alias validation (COMPLETED)
- [x] **Task 1.1.7**: Implement available_externally alias validation (COMPLETED)
- [x] **Task 1.1.8**: Add thread-local alias validation (not needed - no test exists)
- [x] **Task 1.1.9**: Enable alias verification in verify_module (COMPLETED - critical fix!)
  - Test passing: alias.ll ✅

### 1.2 Calling Convention Support

- [x] **Task 1.2.1**: Add missing x86 calling conventions (COMPLETED - X86_INTR added)
- [x] **Task 1.2.2**: Add RISC-V calling conventions (COMPLETED - RISCV_VectorCall)
- [x] **Task 1.2.3**: Add M68k calling conventions (COMPLETED - M68k_INTR, M68k_RTD)
- [x] **Task 1.2.4**: Add AVR calling conventions (COMPLETED - AVR_INTR, AVR_SIGNAL)
- [x] **Task 1.2.5**: Add MSP430 calling conventions (COMPLETED - MSP430_INTR)
- [x] **Task 1.2.6**: Add AArch64 SVE preserve calling convention (COMPLETED - AArch64_SVE_Vector_PCS_Preserve)
- [x] **Task 1.2.7**: Update parser to recognize new calling conventions (COMPLETED)
  - NOTE: Tests require additional validation logic (e.g., x86_intr requires byval params)

### 1.3 Array Initializer Representation

- [x] **Task 1.3.1**: Add ConstantArray variant to ValueKind enum (COMPLETED - already existed)
- [x] **Task 1.3.2**: Modify parser to preserve array elements (COMPLETED)
  - Parser now creates ConstantArray with actual elements
  - Commit: 17960a1

- [ ] **Task 1.3.3**: Add validation for array bounds in GEP
  - File: `src/verification.rs`
  - In GEP validation, check array indices against actual array size
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/geparray-out-of-bounds.ll` (all test cases)

- [ ] **Task 1.3.4**: Add validation for array element type matching
  - File: `src/verification.rs`
  - Check that all elements in ConstantArray match declared element_type
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/callsite-dbgloc.ll` (array type validation)

---

## Phase 2: Operand Bundles & Basic Metadata (Target: +20 tests → 229/338)

### 2.1 Operand Bundle Support ✅ COMPLETE

- [x] **Task 2.1.1**: Define OperandBundle struct (COMPLETED)
  - Created OperandBundle { tag, inputs } in src/instruction.rs

- [x] **Task 2.1.2**: Add operand_bundles field to Instruction (COMPLETED)
  - Added Vec<OperandBundle> field to Instruction struct

- [x] **Task 2.1.3**: Parser integration (COMPLETED)
  - Refactored parse_instruction_operands() to return bundles
  - Updated Call and Invoke parsing to use parse_operand_bundles()
  - Attached bundles to instructions after creation

- [x] **Task 2.1.4**: Parse operand bundle syntax (COMPLETED)
  - Implemented parse_operand_bundles() method
  - Parses [ "tag"(type val, ...) ] syntax correctly

- [x] **Task 2.1.5**: Add accessor methods (COMPLETED)
  - add_operand_bundle()
  - operand_bundles()
  - get_operand_bundle(tag)

- [x] **Task 2.1.6**: Implement deopt bundle validation (COMPLETED)
  - Validates llvm.experimental.deoptimize has exactly one "deopt" bundle
  - Validates cannot be invoked
  - Test passing: deoptimize-intrinsic.ll ✅
  - Also implemented for llvm.experimental.guard
  - Test passing: guard-intrinsic.ll ✅

- [ ] **Task 2.1.7**: Implement gc-live bundle validation for statepoint
  - File: `src/verification.rs`
  - Check gc-live bundle inputs are valid for statepoint
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/statepoint.ll` (gc-live validation cases)

- [ ] **Task 2.1.8**: Implement funclet bundle validation
  - File: `src/verification.rs`
  - Check funclet bundle used correctly with EH pads
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/operand-bundles.ll` (funclet cases)

### 2.2 Basic Metadata Structure Preservation

- [ ] **Task 2.2.1**: Create MetadataNode enum for typed metadata
  - File: `src/metadata.rs`
  - Add enum with variants: DILocation, DISubrange, DICompositeType, DIBasicType, DILocalVariable, DIExpression, DISubprogram, DIFile, DICompileUnit, Generic
  - Tests enabled: (prerequisite for metadata tests)

- [ ] **Task 2.2.2**: Update Metadata struct to use MetadataNode
  - File: `src/metadata.rs`
  - Replace generic HashMap with typed MetadataNode enum
  - Tests enabled: (prerequisite for metadata tests)

- [ ] **Task 2.2.3**: Define DILocation struct and parser
  - File: `src/metadata.rs`
  - Add struct with line, column, scope fields
  - Parse `!DILocation(line: X, column: Y, scope: !Z)`
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/disubrange-missing-lowerBound.ll` (requires DILocation context)

- [ ] **Task 2.2.4**: Define DISubrange struct and parser
  - File: `src/metadata.rs`
  - Add struct with count, lowerBound, upperBound, stride fields
  - Parse `!DISubrange(count: X, lowerBound: Y)`
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/disubrange-missing-lowerBound.ll` (lowerBound validation)
    - `llvm-tests/llvm-project/llvm/test/Verifier/disubrange-invalid-bound-type.ll` (bound type validation)

- [ ] **Task 2.2.5**: Implement DISubrange lowerBound validation
  - File: `src/verification.rs`
  - Check if count is not -1, lowerBound must be present
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/disubrange-missing-lowerBound.ll` (all test cases)

- [ ] **Task 2.2.6**: Implement DISubrange bound type validation
  - File: `src/verification.rs`
  - Check bounds are integer constants or DIVariable/DIExpression
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/disubrange-invalid-bound-type.ll` (all test cases)

- [ ] **Task 2.2.7**: Define DICompositeType struct and parser
  - File: `src/metadata.rs`
  - Add struct with tag, name, size, flags, elements fields
  - Parse `!DICompositeType(tag: DW_TAG_structure_type, ...)`
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/cc-flags.ll` (flag validation)

- [ ] **Task 2.2.8**: Implement DICompositeType flag validation
  - File: `src/verification.rs`
  - Check DIFlagTypePassByReference and DIFlagTypePassByValue are mutually exclusive
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/cc-flags.ll` (all test cases)

---

## Phase 3: Complete Metadata Types (Target: +30 tests → 259/338)

### 3.1 Debug Info Metadata Types

- [ ] **Task 3.1.1**: Define DIBasicType struct and parser
  - File: `src/metadata.rs`
  - Add struct with tag, name, size, encoding fields
  - Parse `!DIBasicType(tag: DW_TAG_base_type, name: "int", size: 32, encoding: DW_ATE_signed)`
  - Tests enabled: (prerequisite for type validation tests)

- [ ] **Task 3.1.2**: Define DILocalVariable struct and parser
  - File: `src/metadata.rs`
  - Add struct with name, scope, file, line, type fields
  - Parse `!DILocalVariable(name: "x", scope: !1, file: !2, line: 10, type: !3)`
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/di-subroutine-localvar.ll` (local variable validation)

- [ ] **Task 3.1.3**: Define DIExpression struct and parser
  - File: `src/metadata.rs`
  - Add struct with operations vector
  - Parse `!DIExpression(DW_OP_deref, DW_OP_plus_uconst, 4)`
  - Tests enabled: (prerequisite for expression validation tests)

- [ ] **Task 3.1.4**: Define DISubprogram struct and parser
  - File: `src/metadata.rs`
  - Add struct with name, linkageName, scope, file, line, type, unit fields
  - Parse `!DISubprogram(name: "foo", ...)`
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/di-subroutine-localvar.ll` (scope validation)

- [ ] **Task 3.1.5**: Define DIFile struct and parser
  - File: `src/metadata.rs`
  - Add struct with filename, directory, checksumkind, checksum fields
  - Parse `!DIFile(filename: "test.c", directory: "/home")`
  - Tests enabled: (prerequisite for file validation tests)

- [ ] **Task 3.1.6**: Define DICompileUnit struct and parser
  - File: `src/metadata.rs`
  - Add struct with language, file, producer, isOptimized, flags, runtimeVersion, emissionKind fields
  - Parse `!DICompileUnit(language: DW_LANG_C99, file: !1, ...)`
  - Tests enabled: (prerequisite for compile unit validation tests)

### 3.2 Debug Info Validation

- [ ] **Task 3.2.1**: Implement DILocalVariable scope validation
  - File: `src/verification.rs`
  - Check DILocalVariable scope is DISubprogram or DILexicalBlock, not DICompositeType
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/di-subroutine-localvar.ll` (all test cases)

- [ ] **Task 3.2.2**: Implement DISubprogram type validation
  - File: `src/verification.rs`
  - Check DISubprogram type is DISubroutineType
  - Tests enabled: (various DISubprogram tests)

- [ ] **Task 3.2.3**: Implement DIFile checksum validation
  - File: `src/verification.rs`
  - Check if checksumkind is present, checksum must be present (and vice versa)
  - Tests enabled: (DIFile validation tests)

- [ ] **Task 3.2.4**: Implement dbg.declare intrinsic validation
  - File: `src/verification.rs`
  - Check first argument is valid variable reference
  - Check second argument is DILocalVariable
  - Check third argument is DIExpression
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/dbg-difile-crash.ll`
    - `llvm-tests/llvm-project/llvm/test/Verifier/dbg-declare-address-expr.ll`

- [ ] **Task 3.2.5**: Implement dbg.value intrinsic validation
  - File: `src/verification.rs`
  - Check metadata arguments are correct types
  - Tests enabled: (dbg.value validation tests)

### 3.3 Specialized Attribute Validation

- [x] **Task 3.3.1**: Implement writable attribute validation (COMPLETED ✅)
  - Added writable, readonly, readnone, memory attributes to ParameterAttributes
  - Implemented all validation rules
  - Test passing: writable-attr.ll ✅

- [x] **Task 3.3.2**: Implement vscale_range attribute validation (PARTIAL ⚠️)
  - Added vscale_range to FunctionAttributes
  - Implemented all validation rules (min > 0, min <= max, power-of-two checks)
  - Parser implemented
  - Status: Test still failing - needs debugging (may be tokenization issue)

- [ ] **Task 3.3.3**: Implement dead_on_unwind attribute validation
  - File: `src/verification.rs`
  - Check only on pointer parameters
  - Check not on varargs
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/dead_on_unwind-invalid.ll` (all test cases)

---

## Phase 4: Edge Cases & Remaining Tests (Target: +79 tests → 338/338)

### 4.1 Advanced Intrinsic Validation

- [ ] **Task 4.1.1**: Implement llvm.experimental.guard validation
  - File: `src/verification.rs`
  - Check condition is i1 type
  - Check widening bundle validation
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/experimental-guard.ll`

- [ ] **Task 4.1.2**: Implement llvm.memcpy.inline validation
  - File: `src/verification.rs`
  - Check length is immediate constant (immarg)
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/memcpy-inline-invalid.ll`

- [ ] **Task 4.1.3**: Implement llvm.stepvector validation
  - File: `src/verification.rs`
  - Check return type is scalable vector of integers
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/stepvector-intrinsic.ll`

- [ ] **Task 4.1.4**: Implement llvm.experimental.convergence.* validation
  - File: `src/verification.rs`
  - Check convergence token usage rules
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/convergence-intrinsics.ll`

- [ ] **Task 4.1.5**: Implement architecture-specific intrinsic validation
  - File: `src/verification.rs`
  - Add validation for ARM, AArch64, x86, AMDGPU intrinsics
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/aarch64-invalid-intrinsics.ll`
    - `llvm-tests/llvm-project/llvm/test/Verifier/arm-invalid-intrinsics.ll`

### 4.2 Control Flow Validation

- [ ] **Task 4.2.1**: Implement invoke reachability validation
  - File: `src/verification.rs`
  - Check invoke normal destination can reach return
  - Check invoke normal destination can't reach another invoke
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/invoke.ll` (all test cases)

- [ ] **Task 4.2.2**: Implement callbr validation
  - File: `src/verification.rs`
  - Check indirect destinations are valid
  - Check inline asm constraints
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/callbr.ll`
    - `llvm-tests/llvm-project/llvm/test/Verifier/callbr-asm-branch-1.ll`
    - `llvm-tests/llvm-project/llvm/test/Verifier/callbr-asm-branch-2.ll`

- [ ] **Task 4.2.3**: Implement catchswitch validation
  - File: `src/verification.rs`
  - Check handler destinations are valid
  - Check catchswitch is only instruction in block
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/catchswitch.ll`

- [ ] **Task 4.2.4**: Implement landingpad validation
  - File: `src/verification.rs`
  - Check landingpad is first non-PHI instruction
  - Check clause types
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/landingpad.ll`

### 4.3 Constant Expression Validation

- [ ] **Task 4.3.1**: Implement constant expression cycle detection
  - File: `src/verification.rs`
  - Detect cycles in constant expressions (GEP, bitcast, etc.)
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/constant-expr-cycles.ll`

- [ ] **Task 4.3.2**: Implement constant GEP bounds validation
  - File: `src/verification.rs`
  - Check GEP indices are in bounds for constant arrays/structs
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/geparray-out-of-bounds.ll`

- [ ] **Task 4.3.3**: Implement constant type consistency validation
  - File: `src/verification.rs`
  - Check constant expressions preserve type consistency
  - Tests enabled: (various constant expression tests)

### 4.4 Global Object Validation

- [ ] **Task 4.4.1**: Implement global variable initializer validation
  - File: `src/verification.rs`
  - Check initializer type matches variable type
  - Check initializer is constant
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/global-init.ll`

- [ ] **Task 4.4.2**: Implement global variable section validation
  - File: `src/verification.rs`
  - Check section name validity
  - Tests enabled: (global section tests)

- [ ] **Task 4.4.3**: Implement global variable alignment validation
  - File: `src/verification.rs`
  - Check alignment is power of 2
  - Check alignment doesn't exceed type requirements
  - Tests enabled: (alignment validation tests)

### 4.5 Type System Validation

- [ ] **Task 4.5.1**: Implement opaque pointer validation
  - File: `src/verification.rs`
  - Check opaque pointers used correctly
  - Tests enabled: (opaque pointer tests)

- [ ] **Task 4.5.2**: Implement vector type validation
  - File: `src/verification.rs`
  - Check scalable vector constraints
  - Check vector element type validity
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/vector-type.ll`

- [ ] **Task 4.5.3**: Implement structure type validation
  - File: `src/verification.rs`
  - Check struct field types
  - Check packed struct constraints
  - Tests enabled: (struct validation tests)

### 4.6 Inline Assembly Validation

- [ ] **Task 4.6.1**: Implement inline asm constraint validation
  - File: `src/verification.rs`
  - Parse and validate constraint strings
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/asm-constraint.ll`

- [ ] **Task 4.6.2**: Implement inline asm type validation
  - File: `src/verification.rs`
  - Check asm result types match constraints
  - Tests enabled: (inline asm tests)

### 4.7 Old LLVM IR Syntax Support

- [ ] **Task 4.7.1**: Add backward compatibility for old intrinsic names
  - File: `src/parser.rs`
  - Map old intrinsic names to new equivalents
  - Tests enabled:
    - `llvm-tests/llvm-project/llvm/test/Verifier/memcpy.ll` (old llvm.memcpy.*)
    - `llvm-tests/llvm-project/llvm/test/Verifier/memmove.ll` (old llvm.memmove.*)
    - ~10 other old syntax tests

- [ ] **Task 4.7.2**: Support old GEP syntax (inbounds before indices)
  - File: `src/parser.rs`
  - Accept both `getelementptr inbounds` and modern syntax
  - Tests enabled: (old GEP syntax tests)

- [ ] **Task 4.7.3**: Support old attribute syntax
  - File: `src/parser.rs`
  - Parse old-style attribute groups
  - Tests enabled: (old attribute syntax tests)

### 4.8 Final Cleanup & Edge Cases

- [ ] **Task 4.8.1**: Fix remaining parser edge cases
  - File: `src/parser.rs`
  - Handle whitespace variations
  - Handle comment placements
  - Handle escaped identifiers
  - Tests enabled: (various edge case tests)

- [ ] **Task 4.8.2**: Add comprehensive error messages
  - Files: `src/verification.rs`, `src/parser.rs`
  - Improve error messages to match LLVM format
  - Tests enabled: (tests checking error message format)

- [ ] **Task 4.8.3**: Verify all 338 tests pass
  - Run complete test suite
  - Investigate any remaining failures
  - Fix final edge cases
  - Tests enabled: ALL (338/338)

- [ ] **Task 4.8.4**: Performance optimization pass
  - Profile test execution
  - Optimize hotspots
  - Ensure tests run efficiently

---

## Test Tracking Summary

### Phase 1 Target Tests (15 tests)
- alias.ll (5 tests)
- callbr-asm-branch-1.ll (2 tests)
- callbr-asm-branch-2.ll (2 tests)
- geparray-out-of-bounds.ll (4 tests)
- callsite-dbgloc.ll (2 tests)

### Phase 2 Target Tests (20 tests)
- deoptimize-intrinsic.ll (4 tests)
- statepoint.ll (3 tests)
- operand-bundles.ll (2 tests)
- disubrange-missing-lowerBound.ll (3 tests)
- disubrange-invalid-bound-type.ll (3 tests)
- cc-flags.ll (1 test)
- di-subroutine-localvar.ll (2 tests)
- dbg-difile-crash.ll (1 test)
- dbg-declare-address-expr.ll (1 test)

### Phase 3 Target Tests (30 tests)
- writable-attr.ll (5 tests)
- vscale_range.ll (4 tests)
- dead_on_unwind-invalid.ll (3 tests)
- Various debug info tests (10 tests)
- Various attribute tests (8 tests)

### Phase 4 Target Tests (79 tests)
- All remaining 142 critical failures not covered in phases 1-3
- Plus improvements to currently passing tests

---

## Progress Tracking

Update this section after completing each task:

**Phase 1 Progress**: 0/29 tasks completed
**Phase 2 Progress**: 0/15 tasks completed
**Phase 3 Progress**: 0/14 tasks completed
**Phase 4 Progress**: 0/26 tasks completed

**Overall Progress**: 0/84 tasks completed (0%)
**Test Pass Rate**: 194/338 (57.4%)

**Next Task**: Task 1.1.1 - Add aliases field to Module struct
