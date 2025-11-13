# Foundational Parser Changes to Reach 267/267 Tests

**Date:** 2025-11-13 11:43
**Current Status:** 161/522 tests passing (30.8%)
**Target:** 267/522 minimum, 522/522 ideal (100%)
**Gap:** 106+ tests needed

## Analysis of Failing Tests

Based on analysis of 361 failing tests, the foundational changes needed are:

### Categories (MECE)
1. **Attribute Parsing** (~100 tests) - Missing parameter/return attributes
2. **Type System** (~80 tests) - Opaque types, target types, scalable vectors
3. **Intrinsic Validation** (~60 tests) - Alignment checks, element type validation
4. **Metadata/Debug Info** (~100+ tests) - DIExpression, metadata attachments
5. **Alias Support** (~20 tests) - Cycle detection, linkage constraints

## Implementation Plan

### Phase 1: Quick Wins - Attribute Parsing (Est: 40+ tests)

#### 1.1 Return Attributes
- [ ] Add `swifterror` to ReturnAttributes struct
- [ ] Parse `swifterror` in parse_return_attributes()
- [ ] Validate swifterror cannot be on return types
- [ ] Add `writable` attribute support
- [ ] Add `dead_on_return` attribute support
- [ ] Add `dead_on_unwind` attribute support

#### 1.2 Parameter Attributes
- [ ] Add `writable` to ParameterAttributes
- [ ] Add `dead_on_return` to ParameterAttributes
- [ ] Add `dead_on_unwind` to ParameterAttributes
- [ ] Parse these attributes in parse_parameter_attributes()
- [ ] Add validation: writable incompatible with readonly/readnone
- [ ] Add validation: writable requires pointer type
- [ ] Add validation: dead_on_return/unwind require pointer type

#### 1.3 Call-Site Attributes
- [ ] Add `elementtype` attribute parsing on call arguments
- [ ] Add `preallocated` call-site attribute
- [ ] Add `musttail` instruction modifier parsing
- [ ] Add `tail` and `notail` modifiers
- [ ] Store call-site attributes on Instruction struct

### Phase 2: Calling Conventions (Est: 15+ tests)

#### 2.1 Missing Calling Conventions
- [ ] Add X86_INTR to CallingConvention enum
- [ ] Add GHC calling convention
- [ ] Add HHVM calling convention
- [ ] Add RISCV_VectorCall (with parameter)
- [ ] Parse calling conventions in parse_calling_convention()

#### 2.2 Calling Convention Validation
- [ ] X86_INTR: first param can be non-pointer, rest need byval
- [ ] X86_INTR: if first param is pointer, needs byval
- [ ] Validate musttail calling convention matching
- [ ] Validate musttail parameter count matching
- [ ] Validate musttail varargs matching
- [ ] Validate musttail ABI attribute matching (byval, inreg, sret)

### Phase 3: Type System Enhancements (Est: 30+ tests)

#### 3.1 Opaque Type Detection
- [ ] Add `is_opaque()` method to Type
- [ ] Track named opaque types during parsing (%X = type opaque)
- [ ] Store opaque type names in Context
- [ ] Implement opaque type size checks (always unsized)

#### 3.2 Scalable Vector Support
- [ ] Add ScalableVector variant to TypeData enum
- [ ] Parse `<vscale x N x type>` syntax
- [ ] Add `is_scalable_vector()` method
- [ ] Add `contains_scalable_vector()` recursive check for structs
- [ ] Validate: globals cannot contain scalable vectors
- [ ] Validate: cannot allocate structs with scalable vectors
- [ ] Validate: cannot GEP into structs with scalable vectors

#### 3.3 Target Extension Types
- [ ] Improve target type parsing to preserve type properties
- [ ] Add token-like type detection for target types
- [ ] Validate target types cannot be in PHI nodes
- [ ] Validate target types in function parameters (non-intrinsic)
- [ ] Validate target types in return values (non-intrinsic)

#### 3.4 X86_AMX Type Support
- [ ] Add X86_AMX variant to TypeData or recognize as special
- [ ] Parse `x86_amx` keyword
- [ ] Validate: cannot be array element
- [ ] Validate: cannot be vector element
- [ ] Validate: cannot be global variable type
- [ ] Validate: cannot allocate x86_amx
- [ ] Validate: non-intrinsic functions cannot take/return x86_amx
- [ ] Validate: indirect calls cannot return x86_amx

### Phase 4: Intrinsic Validation (Est: 25+ tests)

#### 4.1 Alignment Validation
- [ ] Parse `align N` on call operands (e.g., memcpy align 3)
- [ ] Store alignment on call instruction operands
- [ ] Validate alignment is power of 2 for memcpy/memmove/memset
- [ ] Validate alignment for memcpy.inline
- [ ] Validate alignment for memset.inline/pattern

#### 4.2 Element Type Attributes
- [ ] Parse `elementtype(type)` attribute on intrinsic calls
- [ ] Validate llvm.aarch64.ldxr/stxr requires elementtype on args
- [ ] Validate llvm.arm.* intrinsics require elementtype
- [ ] Validate gc.statepoint callee requires elementtype

#### 4.3 Specific Intrinsic Validations
- [ ] llvm.memset.pattern: value must be sized type
- [ ] llvm.threadlocal.address: arg must be GlobalValue
- [ ] llvm.threadlocal.address: arg must be thread_local
- [ ] llvm.ptrmask: first arg must be pointer/vector of pointers
- [ ] llvm.ptrmask: args must both be scalars or both vectors
- [ ] llvm.used/compiler.used: cannot be zeroinitializer
- [ ] llvm.used/compiler.used: members cannot be null
- [ ] Preallocated intrinsics validation

### Phase 5: Instruction Modifiers (Est: 10+ tests)

#### 5.1 Tail Call Modifiers
- [ ] Add `tail_call_kind` field to Instruction (None/Tail/MustTail/NoTail)
- [ ] Parse `musttail` before call instruction
- [ ] Parse `tail` before call instruction
- [ ] Parse `notail` before call instruction
- [ ] Validate musttail: must precede ret with optional bitcast
- [ ] Validate musttail: cannot use with inline asm
- [ ] Validate musttail: calling convention must match
- [ ] Validate musttail: parameter types must match
- [ ] Validate musttail: return type must match
- [ ] Validate musttail: varargs must match

#### 5.2 FP Math Metadata
- [ ] Parse `!fpmath` metadata attachment on instructions
- [ ] Validate fpmath only on floating point instructions
- [ ] Validate fpmath format: !{ float N }
- [ ] Validate fpmath accuracy must be positive
- [ ] Validate fpmath accuracy must be float type (not double)

### Phase 6: Validation Rules (Est: 20+ tests)

#### 6.1 Global Variable Validation
- [ ] Validate llvm.used cannot be zeroinitializer
- [ ] Validate llvm.used members cannot be null
- [ ] Validate invalid uses of llvm.used (cannot be referenced)
- [ ] Validate globals cannot contain scalable types

#### 6.2 Instruction Validation
- [ ] Validate inalloca on varargs: must be on last fixed param
- [ ] Validate inalloca argument must match preallocated alloca
- [ ] Validate speculatable cannot be on call sites (function attr only)
- [ ] Validate invariant.group only on loads/stores
- [ ] Add validation for switch: condition type must match case types

#### 6.3 Attribute Compatibility
- [ ] Validate writable incompatible with readnone
- [ ] Validate writable incompatible with readonly
- [ ] Validate writable + memory(argmem:read) incompatible
- [ ] Validate huge alignments not supported (>2^32)
- [ ] Validate parameter alignment: must be power of 2
- [ ] Validate inreg not allowed in tailcc musttail
- [ ] Validate inalloca not allowed in tailcc musttail
- [ ] Validate inreg not allowed in swifttailcc musttail
- [ ] Validate inalloca not allowed in swifttailcc musttail

### Phase 7: Alias Support (Est: 15+ tests)

#### 7.1 Alias Parsing Improvements
- [ ] Ensure aliases are fully parsed and stored
- [ ] Track alias linkage constraints
- [ ] Store alias targets properly

#### 7.2 Alias Validation
- [ ] Validate alias must point to definition (not declaration)
- [ ] Validate available_externally functions not valid alias targets
- [ ] Validate aliases cannot form cycles (graph traversal)
- [ ] Validate alias cannot point to interposable alias
- [ ] Validate available_externally alias must point to available_externally value

### Phase 8: Testing & Verification

- [ ] Run full test suite: `bash scripts/quick_failure_check.sh`
- [ ] Verify Assembler tests: target 150+/259 (58%)
- [ ] Verify Verifier tests: target 150+/263 (57%)
- [ ] Overall target: 267+/522 (51%+)
- [ ] Commit and push all changes

## Implementation Order (Priority)

1. **Attributes** (Phases 1.1-1.3) - Highest ROI, ~40 tests
2. **Type System** (Phase 3) - Critical foundation, ~30 tests
3. **Calling Conventions** (Phase 2) - Medium complexity, ~15 tests
4. **Intrinsics** (Phase 4) - Specific validations, ~25 tests
5. **Instruction Modifiers** (Phase 5) - Musttail/tail, ~10 tests
6. **Validation Rules** (Phase 6) - Edge cases, ~20 tests
7. **Aliases** (Phase 7) - Lower priority, ~15 tests

## Success Criteria

- [ ] Reach minimum 267/522 tests passing (51%)
- [ ] All changes committed with clear messages
- [ ] All changes pushed to remote
- [ ] No compilation errors
- [ ] No test regressions

## Notes

- Focus on highest ROI items first (attributes, type system)
- Many tests may pass with each foundational change
- Commit frequently to avoid losing work
- Some tests may require metadata parsing which is beyond current scope
