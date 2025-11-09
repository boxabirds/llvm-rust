# Quick Reference: Token 2171 Error Debug Summary

## Error
**"Expected value, found Identifier("iffalse")"** at token position 2171

---

## 1. Tokens 2165-2175 (10 tokens centered on 2171)

```
[2165] Label
[2166] LocalIdent("iftrue")
[2167] Identifier("iftrue")    ← Label start
[2168] Colon                    ← Label end
[2169] Ret                      ← Return instruction
[2170] Void                     ← Return type
[2171] Identifier("iffalse")    ← *** ERROR TOKEN ***
[2172] Colon                    ← This proves 2171 is a label!
[2173] Switch
[2174] IntType(8)
[2175] LocalIdent("val")
```

---

## 2. Source Code Lines (with line numbers)

```llvm
674: iftrue:              ← Tokens 2167-2168
675:   ret void           ← Tokens 2169-2170 (instruction being parsed)
676:   ; CHECK: ret void  ← Comment (not tokenized)
677: iffalse:             ← Tokens 2171-2172 (ERROR: parsed as value)
678:
679:   switch i8 %val, label %defaultdest [
```

---

## 3. Instruction Being Parsed

**`ret void`** (line 675) is the instruction being parsed when token 2171 error occurs.

---

## 4. Which parse_value() Call?

**Location**: `/home/user/llvm-rust/src/parser.rs` line 759

**Call stack**:
```
parse_instruction() [line 518]
  → parse_opcode() [line 652]
      → Recognizes Token::Ret, returns Opcode::Ret
  → parse_instruction_operands(Opcode::Ret) [line 628]
      → Lines 732-762: Handle Ret opcode
      → Line 737: parse_type() consumes "void", advances to token 2171
      → Line 745: matches! includes Token::Identifier(_)
                  → Matches token 2171: Identifier("iffalse")
      → Line 759: parse_value() ← *** ERROR OCCURS HERE ***
          → No match for "iffalse"
          → Falls to default case (line 1831)
          → Returns error (line 1832-1833)
```

---

## 5. Root Cause

**File**: `/home/user/llvm-rust/src/parser.rs`
**Function**: `parse_instruction_operands()`
**Lines**: 732-762 (Ret opcode handling)

### The Bug

After parsing the return type at line 737:
```rust
let _ty = self.parse_type()?;  // Consumes token 2170, advances to 2171
```

The code checks if it should parse a value (lines 740-758):
```rust
if matches!(self.peek(),
    Some(Token::LocalIdent(_)) |
    Some(Token::GlobalIdent(_)) |
    ...
    Some(Token::Identifier(_)) |  ← LINE 745: BUG IS HERE
    ...) {
    let _val = self.parse_value()?;  // Line 759: Error triggered
}
```

**Problem**: Token 2171 is `Identifier("iffalse")`, which matches `Some(Token::Identifier(_))` at line 745. However, this identifier is followed by a colon (token 2172), meaning it's a label definition, NOT a value!

### Missing Check

There's an initial check at line 736:
```rust
self.peek_ahead(1) != Some(&Token::Colon)
```

BUT this check happens BEFORE parsing the type. After `parse_type()` advances the token position, there's no check to see if the NEW current token is an identifier followed by a colon.

---

## 6. Why parse_value() Fails

**File**: `/home/user/llvm-rust/src/parser.rs`
**Function**: `parse_value()`
**Line**: 1634

When called with token 2171 `Identifier("iffalse")`, it checks for special identifiers:
- Line 1638: `if id == "asm"` → No
- Line 1667: `if id == "splat"` → No
- Line 1677: `if id == "dso_local_equivalent"` → No
- Line 1684: `if id == "no_cfi"` → No
- Line 1691: `if id == "blockaddress"` → No

Falls through to default case:
```rust
_ => {
    Err(ParseError::InvalidSyntax {
        message: format!("Expected value, found {:?}", token),
    })
}
```

Generates: **"Expected value, found Identifier("iffalse")"**

---

## 7. Fix Options

### Option A: Add lookahead check after parse_type()
```rust
let _ty = self.parse_type()?;

// ADD THIS:
if self.peek_ahead(1) == Some(&Token::Colon) {
    return Ok(operands);  // Don't parse label as value
}

if matches!(self.peek(), ...) {
    let _val = self.parse_value()?;
}
```

### Option B: Remove Identifier from match pattern
```rust
if matches!(self.peek(),
    Some(Token::LocalIdent(_)) |
    Some(Token::GlobalIdent(_)) |
    // Remove: Some(Token::Identifier(_))  ← DELETE LINE 745
    Some(Token::Integer(_)) |
    ...) {
```

### Option C: Add colon check to condition
```rust
if matches!(self.peek(), ... | Some(Token::Identifier(_)) | ...) &&
   self.peek_ahead(1) != Some(&Token::Colon) {  // ADD THIS
    let _val = self.parse_value()?;
}
```

---

## File Paths

- **Source file with error**: `/home/user/llvm-rust/llvm-tests/llvm-project/test/Bitcode/compatibility-3.6.ll`
- **Parser code**: `/home/user/llvm-rust/src/parser.rs`
- **Debug script**: `/home/user/llvm-rust/debug_compat36_2171.rs`
- **Detailed analysis**: `/home/user/llvm-rust/debug_analysis_2171.md`
- **Flow diagram**: `/home/user/llvm-rust/debug_flow_diagram_2171.md`
