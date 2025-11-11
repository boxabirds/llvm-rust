# LLVM IR Verifier Test Analysis - First 50 Tests

**Analysis Date:** 2025-11-11
**Current Status:** 18/266 negative tests passing (6.8%)
**Tests Analyzed:** First 50 tests alphabetically from `/home/user/llvm-rust/llvm-tests/llvm/test/Verifier/`

## Executive Summary

Out of 50 tests analyzed:
- **44 are negative tests** (should fail verification)
- **6 are positive tests** (should pass verification)

Of the 44 negative tests:
- ✓ **11 tests (26%) are fully implementable** with current data structures
- ~ **21 tests (50%) are partially implementable** (need attribute/metadata parsing)
- ✗ **10 tests (23%) are not implementable** yet (require major parser enhancements)
- ? **2 tests (5%)** uncategorized

---

## Category Breakdown

### ✓ FULLY IMPLEMENTABLE (11 tests)

#### 1. Self-Referential Instructions
- **Tests:** 1
- **File:** `SelfReferential.ll`
- **Validation Rule:** Only PHI nodes may reference their own result value
- **Implementation:**
  - Check if any instruction operand references the instruction's own result
  - Can use Value comparison: `operand.name() == inst.result().name()`
  - Already have: `Instruction`, `Value` APIs
- **Implementable:** YES
- **Priority:** HIGH (quick win)

#### 2. Atomic Type Validation
- **Tests:** 1
- **File:** `atomics.ll`
- **Validation Rule:** Atomic load/store operands must have integer, pointer, floating point, or vector type
- **Implementation:**
  - Check operand types for Load/Store with atomic ordering
  - Use `Type.is_integer()`, `Type.is_pointer()`, `Type.is_float()`, `Type.is_vector()`
  - Already have: Instruction operands, Type methods
- **Implementable:** YES
- **Priority:** HIGH (quick win)

#### 3. GEP Index Validation
- **Tests:** 1
- **File:** `2002-11-05-GetelementptrPointers.ll`
- **Validation Rule:** GEP cannot index through a pointer within an aggregate type
- **Example Invalid:**
  ```llvm
  getelementptr {i32, ptr}, ptr %X, i32 0, i32 1, i32 0
  ; Cannot index into the ptr at position 1 with another i32
  ```
- **Implementation:**
  - Track type progression through GEP indices
  - After indexing into struct field, check if result is pointer
  - If pointer, cannot have more indices (pointers are opaque)
  - Already have: Type.struct_field_type(), GEP operands
- **Implementable:** YES
- **Priority:** HIGH

#### 4. Return Type Validation
- **Tests:** 2
- **Files:** `2002-04-13-RetTypes.ll`, `2008-11-15-RetVoid.ll`
- **Status:** ✅ Already implemented (verification.rs:202-252)
- **Implementable:** YES (done)

#### 5. PHI Node Validation
- **Tests:** 2
- **Files:** `AmbiguousPhi.ll`, `PhiGrouping.ll`
- **Status:** ✅ Already implemented (verification.rs:291-305, 934-963)
- **Implementable:** YES (done)

#### 6. Intrinsic Definition
- **Tests:** 1
- **File:** `2006-12-12-IntrinsicDefine.ll`
- **Status:** ✅ Already implemented (verification.rs:159)
- **Implementable:** YES (done)

#### 7. Alloca Type Validation
- **Tests:** 1
- **File:** `2008-03-01-AllocaSized.ll`
- **Status:** ✅ Already implemented (verification.rs:885-899)
- **Implementable:** YES (done)

#### 8. Switch Type Validation
- **Tests:** 1
- **File:** `2004-05-21-SwitchConstantMismatch.ll`
- **Status:** ✅ Already implemented (verification.rs:900-920)
- **Implementable:** YES (done)

#### 9. Calling Convention Validation
- **Tests:** 1
- **File:** `amdgpu-cc.ll`
- **Status:** ✅ Already implemented (verification.rs:1383-1446)
- **Implementable:** YES (done)

---

### ~ PARTIALLY IMPLEMENTABLE (21 tests)

#### 1. Address Space Validation (9 tests)
- **Files:**
  - `bitcast-address-space-*.ll` (9 files)
- **Validation Rule:** Cannot bitcast between pointers with different address spaces
- **Example:**
  ```llvm
  %cast = bitcast ptr %p to ptr addrspace(1)  ; INVALID
  ```
- **Why Partial:**
  - Need address space information in Type
  - Parser may already preserve this in LLVM-C API
  - Type structure exists, just needs address space field
- **What's Needed:**
  - Add `address_space: Option<u32>` to Type
  - Parse address space from LLVM-C `LLVMGetPointerAddressSpace()`
  - Check in Bitcast validation that address spaces match
- **Priority:** MEDIUM-HIGH

#### 2. Function Attribute Validation (4 tests)
- **Files:**
  - `2008-09-02-FunctionNotes2.ll` (conflicting noinline/alwaysinline)
  - `alloc-size-failedparse.ll` (allocsize indices)
  - `alloc-variant-zeroed.ll` (alloc-variant-zeroed must not be empty)
  - `allocsize.ll` (allocsize parameter validation)
- **Validation Rules:**
  - noinline and alwaysinline are mutually exclusive
  - allocsize parameter indices must be valid and refer to integer parameters
  - alloc-variant-zeroed attribute must not be empty
- **Why Partial:**
  - Function attributes exist but need expansion
  - Can access Function and parameters
  - Need to parse more attribute types
- **What's Needed:**
  - Expand Function.attributes() to include: noinline, alwaysinline, allocsize, etc.
  - Validate allocsize indices against parameter list
  - Check for conflicting attribute combinations
- **Priority:** MEDIUM

#### 3. Parameter Attribute Validation (3 tests)
- **Files:**
  - `2007-12-21-InvokeParamAttrs.ll` (signext on invoke without declaration)
  - `2008-01-11-VarargAttrs.ll` (sret with varargs - already implemented)
  - `align.ll` (align attribute on non-pointer types)
- **Validation Rules:**
  - signext/zeroext only on integer parameters
  - align only on pointer types
  - Attributes on call site must match declaration
- **Why Partial:**
  - Function.attributes() exists but limited
  - Need to parse signext, zeroext, align from LLVM-C
- **What's Needed:**
  - Expand ParameterAttributes struct
  - Parse more attributes from LLVM-C
  - Validate attribute compatibility with parameter types
- **Priority:** MEDIUM

#### 4. Intrinsic Attributes (2 tests)
- **Files:**
  - `aarch64-ldstxr.ll`
  - `arm-intrinsics.ll`
- **Validation Rule:** ARM load-exclusive/store-exclusive intrinsics require elementtype attribute
- **Example:**
  ```llvm
  ; INVALID - missing elementtype
  %a = call i32 @llvm.arm.ldrex.p0(ptr %p)

  ; VALID
  %a = call i32 @llvm.arm.ldrex.p0(ptr elementtype(i32) %p)
  ```
- **Why Partial:**
  - Need elementtype attribute parsing
  - Can access intrinsic name and parameters
- **What's Needed:**
  - Parse elementtype attribute
  - Check specific intrinsics that require it
- **Priority:** LOW (ARM-specific, niche)

#### 5. Invoke Validation (3 tests)
- **Files:**
  - `2009-05-29-InvokeResult1.ll`
  - `2009-05-29-InvokeResult2.ll`
  - `2009-05-29-InvokeResult3.ll`
- **Validation Rule:** Invoke result cannot be used in unwind destination (exception path)
- **Example:**
  ```llvm
  %r = invoke i32 @v() to label %normal unwind label %unwind
  normal:
    ret i32 %r  ; OK
  unwind:
    ret i32 %r  ; INVALID - %r not available in unwind path
  ```
- **Why Partial:**
  - Need CFG edge information (successors)
  - Can track value usage
- **What's Needed:**
  - Parser preserve Invoke successors (normal_dest, unwind_dest)
  - Check if result is used in unwind destination block
- **Priority:** MEDIUM-LOW

---

### ✗ NOT IMPLEMENTABLE YET (10 tests)

#### 1. Metadata Validation (5 tests)
- **Files:**
  - `access_group.ll` (access scope list validation)
  - `alias-scope-metadata.ll` (scope list must be MDNodes)
  - `annotation-metadata.ll` (annotation must have operands)
  - `associated-metadata.ll` (associated value must be pointer typed)
  - `assume-bundles.ll` (assume operand bundle validation)
- **Why Not Implementable:**
  - Requires full metadata parsing infrastructure
  - Need metadata nodes, references, operand bundles
  - Not in current Module/Function/Instruction data structures
- **What's Needed:**
  - Major parser work to preserve metadata
  - Metadata data structures (MDNode, MDString, etc.)
- **Priority:** LOW (requires significant infrastructure)

#### 2. Debug Info Validation (4 tests)
- **Files:**
  - `array_allocated.ll`
  - `array_associated.ll`
  - `array_dataLocation.ll`
  - `array_rank.ll`
- **Why Not Implementable:**
  - Requires debug info metadata parsing (DIFile, DISubprogram, etc.)
  - Complex metadata relationships
- **What's Needed:**
  - Debug info data structures
  - Metadata parsing
- **Priority:** LOW (requires metadata infrastructure)

#### 3. Absolute Symbol Validation (1 test)
- **File:** `absolute_symbol.ll`
- **Why Not Implementable:**
  - Requires module-level symbol metadata
  - Not exposed in current API
- **Priority:** LOW (rare feature)

---

## Implementation Recommendations

### Phase 1: Quick Wins (3 tests) ⚡
**Estimated effort:** 2-4 hours

1. **Self-Referential Instructions** (SelfReferential.ll)
   ```rust
   // In verify_instruction()
   if inst.opcode() != Opcode::PHI {
       if let Some(result) = inst.result() {
           for operand in inst.operands() {
               if operand.name() == result.name() {
                   // Error: Only PHI nodes may reference themselves
               }
           }
       }
   }
   ```

2. **Atomic Type Validation** (atomics.ll)
   ```rust
   // In verify_instruction() for Load/Store with atomic
   if inst.has_atomic_ordering() {
       let value_type = operands[0].get_type();
       if !is_valid_atomic_type(&value_type) {
           // Error: atomic operand must have int/ptr/float/vector type
       }
   }
   ```

3. **GEP Index Validation** (2002-11-05-GetelementptrPointers.ll)
   ```rust
   // Track type through GEP indices
   // When we reach a pointer type, no more indices allowed
   ```

### Phase 2: Address Space Support (9 tests) 🔧
**Estimated effort:** 4-8 hours

1. Add address space to Type:
   ```rust
   pub struct Type {
       // existing fields...
       address_space: Option<u32>,
   }
   ```

2. Parse from LLVM-C: `LLVMGetPointerAddressSpace()`

3. Validate in Bitcast:
   ```rust
   if src_addr_space != dst_addr_space {
       // Error: invalid cast between address spaces
   }
   ```

### Phase 3: Attribute Expansion (7 tests) 📝
**Estimated effort:** 8-16 hours

1. Expand Function attributes
2. Expand Parameter attributes
3. Validate attribute constraints

---

## Test Files Reference

### Fully Implementable Tests (11)
1. ✓ `2002-04-13-RetTypes.ll` - Return type mismatch (implemented)
2. ✓ `2004-05-21-SwitchConstantMismatch.ll` - Switch type (implemented)
3. ✓ `2006-12-12-IntrinsicDefine.ll` - Intrinsic definition (implemented)
4. ✓ `2008-03-01-AllocaSized.ll` - Alloca sized type (implemented)
5. ✓ `2008-11-15-RetVoid.ll` - Return void mismatch (implemented)
6. ✓ `amdgpu-cc.ll` - Calling convention (implemented)
7. ✓ `AmbiguousPhi.ll` - PHI duplicate blocks (implemented)
8. ✓ `PhiGrouping.ll` - PHI grouping (implemented)
9. 🔨 `SelfReferential.ll` - Self-reference check (TODO)
10. 🔨 `atomics.ll` - Atomic types (TODO)
11. 🔨 `2002-11-05-GetelementptrPointers.ll` - GEP indexing (TODO)

### Partially Implementable Tests (21)
Address Space (9):
- `bitcast-address-space-nested-global-cycle.ll`
- `bitcast-address-space-nested-global.ll`
- `bitcast-address-space-through-constant-inttoptr-inside-gep-instruction.ll`
- `bitcast-address-space-through-constant-inttoptr.ll`
- `bitcast-address-space-through-gep-2.ll`
- `bitcast-address-space-through-gep.ll`
- `bitcast-address-space-through-inttoptr.ll`
- `bitcast-address-spaces.ll`
- `bitcast-alias-address-space.ll`

Attributes (7):
- `2007-12-21-InvokeParamAttrs.ll`
- `2008-01-11-VarargAttrs.ll`
- `2008-09-02-FunctionNotes2.ll`
- `aarch64-ldstxr.ll`
- `align.ll`
- `alloc-size-failedparse.ll`
- `alloc-variant-zeroed.ll`
- `allocsize.ll`
- `arm-intrinsics.ll`

Invoke (3):
- `2009-05-29-InvokeResult1.ll`
- `2009-05-29-InvokeResult2.ll`
- `2009-05-29-InvokeResult3.ll`

### Not Implementable Tests (10)
Metadata (5):
- `access_group.ll`
- `alias-scope-metadata.ll`
- `annotation-metadata.ll`
- `associated-metadata.ll`
- `assume-bundles.ll`

Debug Info (4):
- `array_allocated.ll`
- `array_associated.ll`
- `array_dataLocation.ll`
- `array_rank.ll`

Other (1):
- `absolute_symbol.ll`

---

## Next Steps

1. **Implement Phase 1** (3 quick wins) to get to 21/266 (7.9%)
2. **Implement Phase 2** (address spaces) to get to 30/266 (11.3%)
3. **Implement Phase 3** (attributes) to get to 37/266 (13.9%)

Total potential: **37/266 tests passing (13.9%)** with current data structures.

To go further, will need:
- Metadata parsing infrastructure
- Debug info data structures
- Module-level global/alias tracking
