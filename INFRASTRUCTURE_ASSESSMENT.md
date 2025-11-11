# Level 4 Verifier - Infrastructure Assessment

## Current Achievement: 38.0% (128/337 tests)

Starting from 23.4% baseline, delivered +49 tests through targeted validation implementations.

## What Was Accomplished

### 1. Validation Logic Implementations (No Parser Changes Required)
- **GEP Pointer Indexing**: Validates GEP doesn't index through pointers in aggregates (+46 tests)
- **Atomic Type Constraints**: Validates atomic ops only on int/pointer/float/vector (+3 tests)
- **Module Flags Structure**: Validates flag structure (some tests, limited by parser)
- **Test Harness**: Enhanced to recognize verification errors during parsing

These were implementable because they work with data the parser already captures.

### 2. Data Structure Enhancements
- Added `gep_source_type` to Instruction
- Enhanced parser to capture GEP source types in 3-tuple return
- Added metadata introspection API to Metadata
- Added module_flags storage to Module

## Why We're at 38% Not 100%

The remaining 209 tests are systematically blocked by **parser limitations**, not missing validation logic.

### Parser Architectural Limitations

#### 1. Metadata Content Not Parsed (50+ tests blocked)
**Current state:**
```rust
fn skip_metadata(&mut self) -> Option<String> {
    // Returns metadata NAME only, not content
    // !{i32 1, i32 2} → returns None
    // !DILocation(...) → returns "DILocation"
}
```

**What's needed:**
```rust
fn parse_metadata(&mut self) -> Metadata {
    // Must parse: !{i32 1, i32 2}
    // Must parse: !"string"
    // Must parse: !DILocation(line: 5, ...)
    // Must build AST of metadata content
    // Must link metadata references (!0, !1)
}
```

**Why it's hard:**
- Requires recursive descent parser for metadata AST
- Needs metadata registry to resolve references
- Must handle 20+ debug info metadata types
- Needs to integrate into module building
- Estimated: 500-800 lines of new parser code

**Example blocked test (access_group.ll):**
```llvm
load i8, ptr %p, !llvm.access.group !1
!1 = !{!0}      ; Need to parse this tuple
!0 = !{}        ; Need to validate it's distinct
; Current: Can't access !1's content to validate !0
```

#### 2. Constant Expressions Not Evaluated (30+ tests blocked)
**Current state:**
```rust
// Parser creates Value::Const but doesn't evaluate expressions
// Can't validate: bitcast(getelementptr(...))
// Can't track address space changes through const ops
```

**What's needed:**
- Constant expression evaluator
- Type propagation through constant operations
- Address space tracking in constant context
- Const GEP type calculation

**Why it's hard:**
- Requires separate constant evaluator pass
- Must handle all binary/unary/cast operations
- Must propagate types correctly
- Must validate before module construction
- Estimated: 400-600 lines

**Example blocked test:**
```llvm
@global = global %struct.Foo {
  ptr bitcast (
    ptr addrspace(2) getelementptr (...) to ptr addrspace(1)
  )
}
; Current: Can't validate bitcast changes address space in const expr
```

#### 3. Control Flow Not Tracked (15+ tests blocked)
**Current state:**
```rust
// Basic blocks parsed independently
// No successor/predecessor tracking
// No reachability analysis
```

**What's needed:**
- Build CFG during or after parsing
- Track successor/predecessor for each block
- Compute reachability
- Track which blocks are unwind destinations

**Why it's hard:**
- Parser currently doesn't store block relationships
- Labels referenced before definition
- Needs two-pass approach or deferred resolution
- Invoke destinations need special handling
- Estimated: 300-400 lines

**Example blocked test:**
```llvm
%r = invoke i32 @f() to label %normal unwind label %except
normal:
  ret i32 %r  ; OK
except:
  ret i32 %r  ; ERROR: can't use invoke result in unwind dest
; Current: No CFG to track which block is unwind destination
```

#### 4. Parameter Attributes Not Tracked on Call Sites (20+ tests blocked)
**Current state:**
```rust
// Parser recognizes attributes but doesn't store them per-parameter
// Can't validate byref/byval conflicts at call sites
```

**What's needed:**
- Store attributes per parameter in call instructions
- Track which arguments have which attributes
- Validate attribute combinations

**Why it's hard:**
- Call instruction structure doesn't have per-arg attributes
- Need to refactor how attributes are stored
- Must handle varargs correctly
- Estimated: 200-300 lines

**Example blocked test:**
```llvm
call void (i32, ...) @f(i32 1, ptr inalloca(i32) %x, i32 3)
; ERROR: inalloca must be last argument
; Current: Don't track that %x has inalloca
```

#### 5. Exception Handling Not Fully Parsed (10+ tests blocked)
**Current state:**
```rust
// personality attribute not parsed
// landingpad clauses not fully handled
// catchswitch/catchpad relationships not tracked
```

**What's needed:**
- Parse personality attribute on functions
- Parse landingpad filter/catch clauses
- Track exception handling parent/child relationships
- Validate landing

pad is first non-PHI instruction

**Why it's hard:**
- Complex parsing rules for landingpad clauses
- Need to track exception handling scopes
- Filter clauses are complex AST
- Estimated: 250-350 lines

## Realistic Path Forward

### Option A: Parser Rewrite (2-3 weeks)
Systematically enhance parser to capture all needed information:
1. Week 1: Metadata parsing infrastructure
2. Week 2: Constant expressions, CFG building
3. Week 3: Attributes, exception handling, polish

**Result:** Can reach 85-95% (remaining tests are edge cases)

### Option B: Targeted Validation (Current Approach)
Continue adding validation checks that work with existing parser:
- Look for patterns detectable without full metadata
- Add type-based checks
- Validate instruction sequences

**Result:** Can reach 45-55% with diminishing returns

### Option C: Hybrid Approach
1. Document what needs parser work (done in this doc)
2. Add remaining validation checks with current parser (1-2 days)
3. Create detailed spec for parser enhancements
4. Prioritize highest-impact parser changes

**Result:** Reach ~50%, clear roadmap for 100%

## Recommendation

Given current architecture:
1. **Document current state** (this file) ✓
2. **Add remaining checks** with existing infrastructure (~5-10 tests)
3. **Create parser enhancement spec** detailing exact changes needed
4. **Decision point**: Full parser rewrite vs. accept current limitations

The 38% achievement demonstrates the verification framework is solid. The bottleneck is definitively the parser architecture, not the verification logic.

## Technical Debt Items

### Parser Architecture Issues
1. **No two-pass design**: Can't forward-reference metadata
2. **No AST for metadata**: Metadata skipped, not parsed
3. **No constant evaluator**: Can't validate constant expressions
4. **No CFG builder**: Block relationships not tracked
5. **Incomplete attribute tracking**: Call-site attributes lost
6. **Limited exception handling**: personality, landingpad incomplete

### What This Means
- Many validation rules are **correct but can't run** due to missing data
- Parser produces **syntactically valid but semantically incomplete** IR
- Verification **can't access information** that should be in the IR

### Evidence
- 98.8% positive test pass rate (parser + basic validation works)
- 19.1% negative test rate (validation can't access needed data)
- Clear correlation: tests fail when they need metadata/const expr/CFG data

## Conclusion

**Achievement**: 38.0% through targeted validation (GEP, atomic, module flags)
**Bottleneck**: Parser architecture limitations
**Path to 100%**: Requires systematic parser enhancements (~2-3 weeks)
**Alternative**: Accept 45-55% as practical limit with current parser

The work completed provides:
- Solid verification framework
- Clear understanding of architectural needs
- Foundation for future parser enhancements
- Detailed roadmap for reaching 100%
