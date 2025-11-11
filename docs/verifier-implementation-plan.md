# LLVM-Rust Verifier Implementation Plan
**Date:** 2025-11-11
**Goal:** Reach 80%+ negative test coverage (213+ of 266 tests)
**Current Status:** 18/266 (6.8%)

## Executive Summary

This document provides a systematic roadmap for implementing LLVM IR verification rules in the llvm-rust project. The plan is organized into 3 phases targeting specific validation categories, with estimated test coverage and implementation complexity for each.

### Current Achievement (Phase 1.1 Complete)
- ✅ PHI node grouping validation
- ✅ Self-referential value detection
- ✅ Ambiguous PHI detection
- ✅ Intrinsic definition ban
- ✅ Token type restrictions in PHI nodes

**Result:** 18/266 negative tests passing (6.8%)

---

## Phase 1: Foundation (Target: 103 tests, 38%)

### Phase 1.1: PHI Node Rules ✅ COMPLETE
**Status:** Implemented (Commit: 3099fa2)
**Tests Caught:** ~5 tests
**Implementation:** src/verification.rs lines 273-332, 969-1008

### Phase 1.2: Type System Constraints
**Estimated Tests:** ~25 tests
**Complexity:** Medium
**Implementation Time:** 4-6 hours

#### Rules to Implement:

1. **Return Type Validation** (Already done, verify completeness)
   - Functions must return declared type
   - Void functions can't return values
   - Non-void functions must return values

2. **Unsized Type Restrictions**
   ```rust
   // Cannot alloca/load/store unsized types (void, function, label, metadata)
   Opcode::Alloca => {
       if allocated_type.is_void() || allocated_type.is_function() ||
          allocated_type.is_label() || allocated_type.is_metadata() {
           error!("Cannot allocate unsized type");
       }
   }
   ```

3. **Token Type Restrictions** (Partially done, expand)
   - Tokens can't be stored/loaded
   - Tokens can't be in aggregates
   - Tokens only from specific intrinsics

4. **Scalable Vector Constraints**
   - Scalable vectors can't be in structs (except as pointers)
   - Size must be power of 2
   - Valid element types only

5. **Cast Address Space Restrictions**
   - Bitcast cannot change address spaces
   - Only addrspacecast can change address spaces
   - Validate pointee types match in bitcast

**Test Files:**
- `unsized-types-alloca.ll`
- `token1-with-asserts.ll` (expand coverage)
- `token2.ll`
- `scalable-vector-struct-*.ll` (4 files)
- `bitcast-address-space*.ll` (8+ files)

### Phase 1.3: Parameter/Function Attributes
**Estimated Tests:** ~40 tests
**Complexity:** Medium-High
**Implementation Time:** 8-12 hours
**⚠️ Requires:** Parser must extract and store parameter attributes

#### Rules to Implement:

1. **Attribute Type Compatibility**
   ```rust
   // byval, sret, inalloca, byref, preallocated: only on pointers
   if has_attr(param, "byval") && !param.type.is_pointer() {
       error!("byval only applies to pointer parameters");
   }

   // signext, zeroext: only on integers
   // align: only on pointers
   // nest: only on pointers
   ```

2. **Attribute Mutual Exclusivity**
   ```rust
   // These cannot coexist on same parameter:
   let exclusive_groups = [
       ["byval", "inalloca", "preallocated", "inreg", "nest", "byref", "sret"],
       ["byval", "sret"],
       ["byval", "nest"],
       ["byval", "byref"],
       // ... more combinations
   ];
   ```

3. **sret Specific Rules**
   - Only on void-returning functions
   - Must be on first or second parameter only
   - Only one sret per function
   - Must be pointer type

4. **inalloca Specific Rules**
   - Cannot be on vararg functions
   - Only one inalloca per function
   - Must be last parameter
   - Must be pointer type

5. **Alignment Constraints**
   - align attribute: power of 2, <= 2^29
   - byval/byref size limits

**Test Files:**
- `byval-*.ll` (6 files)
- `sret.ll`
- `inalloca*.ll` (4 files)
- `align.ll`
- `preallocated-*.ll` (2 files)
- `swifterror.ll`
- `2007-12-21-InvokeParamAttrs.ll`
- `2008-01-11-VarargAttrs.ll`

**Implementation Strategy:**
1. Extend Parser to capture attributes in Function/Parameter structures
2. Add `Attribute` enum and storage in IR
3. Implement validation in `verify_function_attributes()`

### Phase 1.4: Intrinsic Immediate Arguments (immarg)
**Estimated Tests:** ~30 tests
**Complexity:** Medium
**Implementation Time:** 6-8 hours
**⚠️ Requires:** Intrinsic signature database

#### Rules to Implement:

1. **Build Intrinsic Signature Database**
   ```rust
   // Map intrinsic name -> immarg positions
   let intrinsics = hashmap! {
       "llvm.memcpy" => vec![2, 3],  // align, isvolatile are immarg
       "llvm.memset" => vec![2, 3],
       "llvm.lifetime.start" => vec![0], // size is immarg
       "llvm.experimental.stackmap" => vec![0, 1], // id, numShadowBytes
       // ... 50+ more intrinsics
   };
   ```

2. **Validate Constant Arguments**
   ```rust
   Opcode::Call => {
       if let Some(intrinsic_name) = get_intrinsic_name(callee) {
           if let Some(immarg_positions) = INTRINSICS.get(intrinsic_name) {
               for &pos in immarg_positions {
                   if !args[pos].is_constant() {
                       error!("Intrinsic {} argument {} must be constant",
                              intrinsic_name, pos);
                   }
               }
           }
       }
   }
   ```

3. **Architecture-Specific Intrinsics**
   - AMDGPU intrinsics
   - ARM intrinsics
   - x86 intrinsics
   - Each has specific immarg requirements

**Test Files:**
- `intrinsic-immarg.ll` (245 lines, comprehensive)
- `AMDGPU/intrinsic-immarg.ll`
- `ARM/intrinsic-immarg.ll`
- `statepoint.ll` (gc.statepoint immargs)

**Implementation Strategy:**
1. Create `src/intrinsics.rs` with intrinsic definitions
2. Add is_constant() method to Value
3. Validate in Call instruction verification

---

## Phase 2: Advanced Control Flow (Target: +65 tests, 62% total)

**Estimated Time:** 10-15 hours
**Complexity:** Medium-High
**⚠️ Requires:** CFG construction, dominator tree

### Phase 2.1: Exception Handling Validation
**Estimated Tests:** ~35 tests
**Complexity:** High

#### Rules to Implement:

1. **Invoke/LandingPad Pairing**
   ```rust
   // Invoke's unwind destination must contain:
   // - landingpad (old-style), or
   // - catchswitch/catchpad/cleanuppad (new-style)

   if inst.opcode() == Opcode::Invoke {
       let unwind_dest = inst.unwind_destination();
       let first_inst = unwind_dest.first_instruction();
       if !is_eh_pad(first_inst) {
           error!("Unwind destination missing exception handling instruction");
       }
   }
   ```

2. **EH Pad Nesting Rules**
   - catchpad must have catchswitch parent
   - cleanuppad can be in any funclet
   - No EH pad cycles
   - Proper funclet operand bundle usage

3. **Landing Pad Constraints**
   - Must be first non-PHI instruction (already done)
   - Only one per block (already done)
   - Result type must be correct

4. **Resume Validation**
   - Must have exactly one aggregate operand
   - Must be in landing pad or cleanup block

**Test Files:**
- `invalid-eh.ll` (26 different violations!)
- `invoke.ll`
- `invalid-cleanuppad-*.ll`
- `landingpad.ll`
- `resume.ll`

**Implementation Strategy:**
1. Add CFG successor/predecessor tracking to BasicBlock
2. Implement `find_unwind_destination()` helper
3. Add EH pad type detection
4. Validate funclet relationships

### Phase 2.2: Dominance & SSA Validation
**Estimated Tests:** ~10 tests
**Complexity:** Medium

#### Rules to Implement:

1. **Build Dominator Tree**
   ```rust
   struct DominatorTree {
       idom: HashMap<BlockId, BlockId>,  // immediate dominator
   }

   impl DominatorTree {
       fn dominates(&self, a: BlockId, b: BlockId) -> bool {
           // Check if block A dominates block B
       }
   }
   ```

2. **Use-Def Dominance**
   ```rust
   // Every use of a value must be dominated by its definition
   for inst in all_instructions {
       for operand in inst.operands() {
           let def_block = operand.defining_block();
           let use_block = inst.parent_block();

           if !dominates(def_block, use_block) {
               error!("Use does not dominate definition");
           }
       }
   }
   ```

3. **Invoke Special Case**
   ```rust
   // Invoke results don't dominate the unwind destination
   if inst.opcode() == Opcode::Invoke {
       let result_uses = find_all_uses(inst.result());
       for use_site in result_uses {
           if is_in_unwind_path(use_site, inst.unwind_dest()) {
               error!("Invoke result used in unwind destination");
           }
       }
   }
   ```

**Test Files:**
- `dominates.ll`
- `2009-05-29-InvokeResult*.ll` (3 files)
- `operand-bundles.ll` (dominance of bundle operands)

**Implementation Strategy:**
1. Implement dominator tree construction (Cooper-Harvey-Kennedy algorithm)
2. Add block/instruction parent tracking
3. Validate during SSA verification pass

### Phase 2.3: Calling Convention Validation
**Estimated Tests:** ~20 tests
**Complexity:** Medium

#### Rules to Implement:

1. **musttail Constraints**
   ```rust
   // musttail call must:
   // - Immediately precede return
   // - Have same calling convention as parent function
   // - Have compatible parameter types
   // - Have compatible return types
   // - Match varargs-ness

   if inst.is_musttail() {
       let next_inst = inst.next_instruction();
       if next_inst.opcode() != Opcode::Ret {
           error!("musttail must be followed by return");
       }

       if inst.calling_convention() != function.calling_convention() {
           error!("musttail calling convention mismatch");
       }
   }
   ```

2. **Non-Callable Calling Conventions**
   ```rust
   // amdgpu_kernel functions cannot be called
   // amdgpu_gfx functions have restrictions

   if callee.calling_convention() == CC::AMDGPUKernel {
       error!("Cannot call amdgpu_kernel functions");
   }
   ```

3. **Vararg Validation**
   - Correct use of va_start/va_end/va_copy
   - Parameter count matching

**Test Files:**
- `musttail-invalid.ll`
- `tailcc-musttail.ll`
- `swifttailcc-musttail.ll`
- `call-to-non-callable-functions.ll` (530 lines!)
- `amdgpu-cc.ll`

---

## Phase 3: Metadata & Debug Info (Target: +75 tests, 89% total)

**Estimated Time:** 12-18 hours
**Complexity:** High
**⚠️ Requires:** Parser metadata preservation

### Phase 3.1: Metadata Attachments Validation
**Estimated Tests:** ~30 tests

#### Rules to Implement:

1. **fpmath Metadata**
   ```rust
   // fpmath requires:
   // - Floating-point result type
   // - Valid accuracy value (> 0.0)

   if let Some(fpmath) = inst.metadata("fpmath") {
       if !inst.result_type().is_float() {
           error!("fpmath requires floating-point result");
       }
       let accuracy = fpmath.get_float();
       if accuracy <= 0.0 {
           error!("fpmath accuracy must be positive");
       }
   }
   ```

2. **range Metadata**
   ```rust
   // range requires:
   // - Integer or pointer result
   // - Well-formed ranges (non-empty, no overlaps)
   // - Constant integer bounds

   if let Some(range) = inst.metadata("range") {
       validate_range_bounds(range, inst.result_type());
   }
   ```

3. **tbaa Metadata**
   ```rust
   // TBAA (Type-Based Alias Analysis) metadata:
   // - Proper node structure
   // - No cycles
   // - Valid parent chains

   validate_tbaa_structure(tbaa_node);
   ```

4. **branch_weights Metadata**
   ```rust
   // branch_weights must have:
   // - Correct number of weights for branch/switch
   // - All positive integer weights
   ```

**Test Files:**
- `fpmath.ll`
- `range-*.ll` (multiple files)
- `tbaa.ll`
- `branch-weight.ll`
- `align-md.ll`

### Phase 3.2: Debug Info (DISubprogram, DILocation, etc.)
**Estimated Tests:** ~45 tests
**Complexity:** High

#### Rules to Implement:

1. **DISubprogram Validation**
   ```rust
   // DISubprogram must have:
   // - Valid scope (DICompileUnit or DINamespace)
   // - Type must be DISubroutineType
   // - Variables must be DILocalVariable nodes
   // - Correct linkage name format
   ```

2. **DILocation Validation**
   ```rust
   // DILocation must have:
   // - Valid line/column numbers
   // - Scope must be DISubprogram or DILexicalBlock
   // - inlinedAt must be DILocation if present
   ```

3. **DICompositeType/DIDerivedType**
   ```rust
   // Address space restrictions
   // Size and alignment validation
   // Element type compatibility
   ```

4. **Debug Intrinsic Validation**
   ```rust
   // dbg.declare, dbg.value, dbg.addr validation
   // - First arg must be metadata
   // - Second arg must be DILocalVariable
   // - Third arg must be DIExpression
   ```

**Test Files:**
- `DISubprogram.ll`
- `DILocation-*.ll` (3 files)
- `diderivedtype-address-space-*.ll` (10+ files)
- `llvm.dbg.declare-address.ll`
- `diexpression-*.ll` (multiple files)
- `disubrange-*.ll` (multiple files)

---

## Phase 4: Specialized Features (Target: +30 tests, 100% total)

**Estimated Time:** 8-12 hours

### Phase 4.1: Garbage Collection & Statepoint
**Estimated Tests:** ~18 tests

#### Rules:
- gc.statepoint signature validation
- gc.relocate must relocate pointers
- gc.result type matching
- Token usage validation

**Test Files:**
- `statepoint.ll`
- `gc_relocate_*.ll` (4 files)
- `invalid-statepoint*.ll`

### Phase 4.2: Global/Module Level
**Estimated Tests:** ~25 tests

#### Rules:
- Alias cycle detection
- IFunc resolver validation
- Comdat restrictions
- Module flags validation
- Global constructor/destructor format

**Test Files:**
- `alias.ll`
- `ifunc.ll`
- `comdat.ll`
- `module-flags-*.ll`
- `global-ctors.ll`

### Phase 4.3: Remaining Categories
**Estimated Tests:** ~12 tests

- Operand bundles (uniqueness, dominance)
- Inline assembly (callbr validation)
- Atomic operations (valid types, ordering)
- Memory allocation attributes

---

## Implementation Infrastructure Needed

### 1. Parser Enhancements

**Priority: Critical**

```rust
// Add to Function structure
pub struct Function {
    // ... existing fields
    attributes: Vec<FunctionAttribute>,
    calling_convention: CallingConvention,
    metadata: HashMap<String, MetadataRef>,
}

// Add to Parameter structure
pub struct Parameter {
    type_: Type,
    attributes: Vec<ParameterAttribute>,
    name: Option<String>,
}

// Add to Instruction structure
pub struct Instruction {
    // ... existing fields
    metadata: HashMap<String, MetadataRef>,
    successors: Vec<BlockRef>,  // For CFG
}
```

**Files to modify:**
- `src/parser.rs` - Extract attributes, metadata, successors
- `src/function.rs` - Store attributes
- `src/instruction.rs` - Store metadata and CFG info

### 2. CFG Infrastructure

**Priority: High**

```rust
pub struct ControlFlowGraph {
    blocks: HashMap<BlockId, BasicBlock>,
    edges: HashMap<BlockId, Vec<BlockId>>,
    reverse_edges: HashMap<BlockId, Vec<BlockId>>,
}

impl ControlFlowGraph {
    fn successors(&self, block: BlockId) -> &[BlockId];
    fn predecessors(&self, block: BlockId) -> &[BlockId];
    fn dominators(&self) -> DominatorTree;
}
```

**New file:** `src/cfg.rs`

### 3. Intrinsic Database

**Priority: Medium**

```rust
pub struct IntrinsicSignature {
    name: &'static str,
    immarg_positions: &'static [usize],
    overloaded_types: Vec<TypeKind>,
    attributes: Vec<IntrinsicAttribute>,
}

lazy_static! {
    static ref INTRINSICS: HashMap<&'static str, IntrinsicSignature> = {
        // 100+ intrinsic definitions
    };
}
```

**New file:** `src/intrinsics.rs`

### 4. Dominator Tree

**Priority: Medium**

```rust
pub struct DominatorTree {
    idom: HashMap<BlockId, BlockId>,
    frontier: HashMap<BlockId, HashSet<BlockId>>,
}

impl DominatorTree {
    fn build(cfg: &ControlFlowGraph) -> Self;
    fn dominates(&self, a: BlockId, b: BlockId) -> bool;
    fn strictly_dominates(&self, a: BlockId, b: BlockId) -> bool;
}
```

**New file:** `src/dominance.rs`

---

## Testing Strategy

### 1. Incremental Testing

After each rule implementation:
```bash
cargo test --test level4_verifier_tests -- --nocapture
```

Track progress:
- Positive tests: Should remain 71/71 (100%)
- Negative tests: Track improvement from 18/266
- Overall: Target 80%+ (268+/337)

### 2. Per-Category Testing

Test specific categories:
```bash
# Test only PHI-related files
cargo test --test level4_verifier_tests -- phi

# Test only attribute files
cargo test --test level4_verifier_tests -- byval
```

### 3. Regression Prevention

- Run full test suite after each phase
- Ensure no regressions in positive tests
- Document any expected failures

---

## Estimated Timeline

### Fast Track (Focus on ROI)
- **Week 1:** Phase 1.2 + 1.3 (Type system + Attributes) → ~45% coverage
- **Week 2:** Phase 1.4 + 2.1 (Intrinsics + EH) → ~65% coverage
- **Week 3:** Phase 2.2 + 2.3 (Dominance + CC) → ~75% coverage
- **Week 4:** Phase 3.1 (Metadata) → ~85% coverage
- **Total: 4 weeks to 85%**

### Comprehensive (Full Implementation)
- **Weeks 1-2:** Phase 1 complete → 38% coverage
- **Weeks 3-4:** Phase 2 complete → 62% coverage
- **Weeks 5-6:** Phase 3 complete → 89% coverage
- **Week 7:** Phase 4 complete → ~100% coverage
- **Week 8:** Polish and edge cases
- **Total: 8 weeks to 100%**

---

## Success Metrics

### Phase 1 Target (38%, 103 tests)
- [ ] PHI node rules (5 tests) ✅ DONE
- [ ] Type system constraints (25 tests)
- [ ] Parameter attributes (40 tests)
- [ ] Intrinsic immarg (30 tests)

### Phase 2 Target (62%, 168 tests)
- [ ] Exception handling (35 tests)
- [ ] Dominance & SSA (10 tests)
- [ ] Calling conventions (20 tests)

### Phase 3 Target (89%, 243 tests)
- [ ] Metadata attachments (30 tests)
- [ ] Debug info (45 tests)

### Phase 4 Target (100%, 273+ tests)
- [ ] GC/Statepoint (18 tests)
- [ ] Module-level (25 tests)
- [ ] Specialized (12 tests)

---

## Priority Recommendations

### Immediate (Highest ROI)
1. ✅ PHI node rules - DONE (5 tests for ~2 hours work)
2. Type system constraints (25 tests for ~4 hours work)
3. Intrinsic immarg (30 tests for ~6 hours work)

### Near-term (Good ROI)
4. Exception handling basics (20 tests for ~4 hours)
5. Calling conventions (20 tests for ~5 hours)

### Medium-term (Infrastructure dependent)
6. Parameter attributes (40 tests, requires parser work)
7. Dominance analysis (10 tests, requires CFG)

### Long-term (Lower priority)
8. Debug info validation (45 tests, very complex)
9. Advanced metadata (30 tests, parser dependent)

---

## Conclusion

This plan provides a systematic approach to implementing comprehensive LLVM IR verification. The phased approach allows for:

1. **Incremental Progress** - Each phase delivers measurable improvements
2. **Flexible Prioritization** - Can adjust based on ROI or requirements
3. **Infrastructure Building** - Early phases lay groundwork for later ones
4. **Clear Success Metrics** - Easy to track progress and completion

**Recommendation:** Follow the "Fast Track" timeline, focusing on rules with the highest test coverage per hour of work. This gets to 85% coverage in 4 weeks while building necessary infrastructure along the way.
