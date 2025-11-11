# Level 4 Verifier Progress Report

## Current Status
- **Overall**: 38.0% (128/337 tests passing)
- **Positive tests**: 98.8% (79/80 passing)
- **Negative tests**: 19.1% (49/257 passing)
- **Starting point**: 23.4% (79/337)
- **Improvement**: +49 tests fixed (+14.6 percentage points)

## Completed Validations

### 1. GEP Pointer Indexing Validation (+46 tests)
- **Implementation**: `verify_gep_no_pointer_indexing()` in `src/verification.rs`
- **What it checks**: Validates that GEP doesn't try to index through a pointer within an aggregate
- **Example**: Catches `getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0` (indexing field 1 which is ptr, then trying to index further)
- **Infrastructure added**:
  - Parser now captures GEP source type
  - `Instruction.gep_source_type` field populated during parsing
  - Parser returns 3-tuple: `(operands, result_type, gep_source_type)`

### 2. Atomic Operation Type Validation (+3 tests)
- **Implementation**: `verify_atomic_instruction()` in `src/verification.rs`
- **What it checks**: Atomic load/store/cmpxchg/rmw can only work with int, pointer, float, or vector types
- **Example**: Catches `store atomic %struct %v, ptr %P` (cannot atomically store struct types)
- **Tests fixed**: atomics.ll and related tests

### 3. Module Flags Validation (infrastructure)
- **Implementation**: `verify_module_flags()` in `src/verification.rs`
- **What it checks**:
  - Module flags must be metadata tuples with exactly 3 operands
  - Behavior field must be integer 1-8
  - ID field must be metadata string
  - Flag IDs must be unique (except 'require' type)
  - Value constraints based on behavior type
- **Note**: Module flags parsing not fully implemented in parser, so not catching tests yet

### 4. Test Harness Improvements
- **File**: `tests/level4_verifier.rs`
- **Enhancement**: Now recognizes verification errors during parsing as correct failures for negative tests
- **Impact**: Changed how we count tests - many validations trigger during parsing phase

## Infrastructure Blockers

The remaining 209 failing tests require major infrastructure components that don't exist yet:

### 1. Metadata Parsing and Validation (~50+ tests)
**Currently**: Parser skips metadata values, only stores attachment names

**Needed**:
- Parse metadata node structures (!{}, !DILocation(), etc.)
- Store actual metadata content, not just references
- Validate metadata node structure:
  - `llvm.access.group` must contain only distinct MDNodes
  - `prof` (branch_weights) must have correct operand counts
  - `absolute_symbol` range validation
  - Debug info metadata (DICompositeType, DISubrange) constraints

**Affected tests**: access_group.ll, branch-weight.ll, absolute_symbol.ll, array_allocated.ll, array_associated.ll, annotation-metadata.ll, assume-bundles.ll, associated-metadata.ll, and 40+ more

### 2. Constant Expression Validation (~30+ tests)
**Currently**: Constant expressions in global initializers not validated

**Needed**:
- Constant expression evaluator/validator
- Track address space changes through constant GEPs and bitcasts
- Validate constant bitcast address space constraints
- Check constant expressions in global variable initializers

**Affected tests**: All bitcast-address-space-*.ll tests (12+), bitcast-pointer-vector-*.ll tests, bitcast-alias-address-space.ll, and others

**Example**: `@global = global %struct.Foo { ptr bitcast (ptr addrspace(2) @array to ptr addrspace(1)) }`
This should fail because bitcast changes address space in constant expression.

### 3. Control Flow Analysis (~10+ tests)
**Currently**: No CFG analysis, basic block relationships not tracked

**Needed**:
- Build CFG during or after parsing
- Track which basic blocks can reach which others
- Validate invoke result usage:
  - Invoke results can only be used in normal destination
  - Cannot use invoke results in unwind (exception) destination

**Affected tests**: 2009-05-29-InvokeResult*.ll (3 tests), dead-on-return.ll, invoke.ll, and others

### 4. Parameter/Function Attribute Validation (~20+ tests)
**Currently**: Attributes parsed but not validated for conflicts or requirements

**Needed**:
- Track which attributes are present on parameters
- Validate attribute combinations:
  - byref/byval/inalloca/sret/inreg/nest are mutually exclusive
  - byref requires sized types (not opaque)
  - Some intrinsics require elementtype on specific parameters
  - alloc-family attribute consistency

**Affected tests**: byref.ll, byval-*.ll, aarch64-ldstxr.ll, arm-intrinsics.ll, alloc-variant-zeroed.ll, and 15+ more

### 5. Intrinsic Signature Database (~15+ tests)
**Currently**: Limited intrinsic validation

**Needed**:
- Database of intrinsic signatures
- Parameter type checking for intrinsics
- Return type validation for intrinsics
- Overloading rules

**Affected tests**: Various intrinsic-*.ll files, reduction-intrinsics.ll, sat-intrinsics.ll, and others

### 6. Other Infrastructure
- **Switch constant type matching**: Need better type comparison (2004-05-21-SwitchConstantMismatch.ll)
- **Token type validation**: More checks for token type usage
- **Operand bundle validation**: Check bundle structure and constraints
- **Comdat validation**: More comprehensive comdat checks

## Tests Already Passing in Baseline

The 79 positive tests passing are mostly well-formed IR that doesn't trigger any errors. The baseline 79 tests from 23.4% includes tests that were passing during parsing (some verification happens in parser).

## Recommended Next Steps

To reach higher percentages:

### Short-term (10-20% improvement possible):
1. **Implement metadata parsing** - Would fix ~50 tests
   - Parse metadata node structures
   - Store metadata content
   - Implement metadata validation rules

2. **Add parameter attribute validation** - Would fix ~20 tests
   - Check attribute combinations
   - Validate attribute requirements

### Medium-term (additional 10-15%):
3. **Implement constant expression validator** - Would fix ~30 tests
   - Build constant evaluator
   - Track types through constant operations
   - Validate address space constraints

4. **Build CFG analysis** - Would fix ~10 tests
   - Track basic block relationships
   - Validate value usage across blocks

### Long-term (final 25-30%):
5. **Intrinsic signature database** - Would fix ~15 tests
6. **Complete all validation rules** - Remaining edge cases

## Files Modified

### Core Implementation
- `src/verification.rs`: Added GEP validation, atomic validation, module flags validation
- `src/parser.rs`: Enhanced to capture GEP source types, return 3-tuple from `parse_instruction_operands()`
- `src/instruction.rs`: Added `gep_source_type` field and accessor methods
- `src/metadata.rs`: Added introspection API (is_string, is_int, operands, etc.)
- `src/module.rs`: Added module_flags storage

### Testing
- `tests/level4_verifier.rs`: Main test harness with improved counting
- `tests/test_gep_specific.rs`: Specific GEP validation test

## Commits
1. Initial comdat and self-referential validation
2. Address space tracking and bitcast validation
3. Metadata introspection API
4. GEP pointer indexing validation
5. Module flags validation
6. Atomic operation type validation

Total: 6 commits to branch `claude/llvm-rust-level-4-verifier-011CV2ApR4PLLcSCnFePz4kJ`
