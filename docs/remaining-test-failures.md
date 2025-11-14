# Remaining Test Failures Analysis

## Current Status
- **Total tests**: 338
- **Passing**: 194 (57.4%)
- **Critical failures**: 142 (incorrectly accepting invalid IR)
- **Other failures**: 2

## Test Fixes This Session
1. **SelfReferential.ll** - Added validation: only PHI nodes can reference their own value

## Categories of Remaining Failures

### 1. Parser Limitations (Estimated 50+ tests)

#### Alias Validation
- `alias.ll` - Parser doesn't create Alias objects
- Comprehensive alias validation code added but not triggered

#### Calling Conventions
- `x86_intr.ll` - x86_intrcc calling convention not in parser enum
- Several architecture-specific calling conventions missing

#### Metadata
- Many tests require metadata parsing/validation
- Debug info tests (DILocation, DISubprogram, etc.)
- Annotation metadata
- Access group metadata

#### Operand Bundles
- `assume-bundles.ll` - Operand bundle parsing incomplete
- Preallocated bundles
- GC bundles

#### Global Array Initializers
- `llvm.used-invalid-init.ll` - All arrays parsed as ZeroInitializer
- `llvm.used-invalid-init2.ll` - Cannot validate specific array elements

### 2. Complex Validation Logic (Estimated 30+ tests)

#### CFG/Dominance Analysis Required
- `2009-05-29-InvokeResult1.ll` - Invoke results in unwind block
- `2009-05-29-InvokeResult2.ll` - Invoke results via PHI from unwind
- `2009-05-29-InvokeResult3.ll` - Invoke results reachability analysis
- Exception handling validation

#### Constant Expression Validation  
- `x86_amx9.ll` - Const x86_amx not allowed (requires constant expr checking)
- Address space validation through constant expressions
- Bitcast cycles in global initializers

#### Intrinsic-Specific Validation
- Architecture-specific intrinsics (aarch64, arm, etc.)
- GC statepoint validation
- Complex intrinsic parameter rules

### 3. Old LLVM IR Syntax (Estimated 10+ tests)
- `2004-05-21-SwitchConstantMismatch.ll` - Uses "int" vs "uint" (no longer distinct)
- Tests using deprecated syntax that modern parser doesn't support

### 4. Achievable Improvements (Estimated 50+ tests)

These may be fixable with targeted validation additions:
- Function attribute value validation (warn-stack-size, patchable-function-*)
- Call site attribute validation (speculatable)
- Some metadata validation that doesn't require full parser support
- Type system enhancements (target types in vectors)
- Switch case type checking (already implemented, may need parser support)

## Recommendations

### Short Term
1. Continue adding validation for achievable cases
2. Document parser limitations clearly
3. Focus on validation that doesn't require parser changes

### Medium Term
1. Enhance parser to create Alias objects
2. Add missing calling conventions
3. Improve metadata preservation

### Long Term
1. Implement CFG/dominance analysis for complex validation
2. Full operand bundle support
3. Comprehensive constant expression validation

## Parser Enhancement Priorities

Based on test impact:
1. **Alias support** - Would unlock alias cycle detection, linkage validation
2. **Calling conventions** - x86_intrcc and others
3. **Metadata structure** - Debug info, annotations, etc.
4. **Operand bundles** - Preallocated, GC, deopt
5. **Array initializers** - Proper element representation

