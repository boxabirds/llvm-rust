# Parser Timeout/Infinite Loop Fix

## Problem

The `test_parse_llvm_assembler_tests` test would hang indefinitely ("kill the session") when run, making it impossible to stop without terminating the entire session.

## Root Cause

The parser had several unbounded loops that could enter infinite iteration when parsing malformed or complex LLVM IR:

1. **Function body parsing loop** in `parse_function_definition`: Would repeatedly call `parse_basic_block()` which was returning `Some` without consuming tokens, creating an infinite loop.

2. **Module-level parsing loop** in `parse_module`: No iteration limit, could loop forever on certain token sequences.

3. **Basic block instruction parsing loop** in `parse_basic_block`: No iteration limit.

4. **Skip functions**: `skip_parameter_attributes` and `skip_until_newline_or_semicolon` had unbounded loops.

5. **Instruction operand parsing**: The default case in `parse_instruction_operands` did nothing, leaving tokens unconsumed.

## Solution

Added comprehensive safeguards to prevent infinite loops:

### 1. Module Parsing Safety (`parse_module`)
- Added `MAX_MODULE_ITERATIONS` limit (100,000 iterations)
- Returns error if limit exceeded

### 2. Basic Block Parsing Safety (`parse_basic_block`)
- Added `MAX_INSTRUCTIONS_PER_BLOCK` limit (10,000 instructions per block)
- Returns error if limit exceeded

### 3. Function Body Parsing Safety (`parse_function_definition`)
- Added `MAX_BASIC_BLOCKS` limit (10,000 basic blocks per function)
- Returns error if limit exceeded

### 4. Skip Function Safety
- `skip_parameter_attributes`: Max 50 tokens
- `skip_until_newline_or_semicolon`: Max 500 tokens
- Both have iteration counters to prevent infinite loops

### 5. Instruction Operand Parsing (`parse_instruction_operands`)
- Default case now actively skips to next recognizable statement
- Added `MAX_SKIP_TOKENS` limit (100 tokens)
- Looks for instruction boundaries to stop skipping

### 6. Test Improvements
- Added per-file timing to detect slow parsing
- Added detection for "exceeded maximum iterations" errors
- Displays warnings for files that take >5 seconds

## Testing

Created `test_parser_safety.ll` and example program to verify the fix works correctly. The parser now:
- Completes in <1ms for simple valid LLVM IR
- Returns meaningful error messages when hitting iteration limits
- Never hangs indefinitely

## Files Modified

- `src/parser.rs`: Added iteration limits and safety checks
- `tests/parse_llvm_tests.rs`: Added timing and better error reporting
- `TIMEOUT_FIX.md`: This documentation

## Impact

- Parser will never hang indefinitely
- Malformed LLVM IR will produce error messages instead of freezing
- Performance impact is negligible for valid IR (limits are very high)
- Error messages clearly indicate when iteration limits are hit
