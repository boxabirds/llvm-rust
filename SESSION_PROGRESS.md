# Test Suite Progress - Session Update

## Current Status
- **Negative Tests**: 113/255 passing (44.3%)
- **Positive Tests**: 239/240 passing (99.6%)
- **Total**: 352/495 passing (71.1%)

## Tests Fixed This Session
1. alias-redefinition.ll - Duplicate alias detection
2. byref-parse-error-8.ll - byref on function validation
3. byref-parse-error-9.ll - byref on function validation
4. byref-parse-error-10.ll - byref on function validation
5. 2007-08-06-AliasInvalid.ll - Bare alias syntax detection
6. invalid-attrgrp.ll - Attribute group ID validation
7. invalid-comdat2.ll - Duplicate comdat detection
8. invalid-comdat.ll - Undefined comdat reference validation

## Key Improvements
- Fixed parser main loop to properly continue after parsing aliases (prevents token skipping)
- Added duplicate detection for aliases and globals
- Added parameter-only attribute validation (byref, byval, etc. cannot be on functions)
- Added attribute group ID requirement validation
- Added comdat definition tracking and duplicate/undefined reference detection

## Remaining Work
- 142 negative tests still failing
- Most remaining tests require:
  - Complex semantic validation (SSA, type checking)
  - Metadata/debug info validation (~70 tests)
  - Instruction-specific validation (getelementptr, landingpad, etc.)
  - Use-list ordering validation (~16 tests)
