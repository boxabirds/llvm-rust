# Visual Parsing Flow Diagram - Token 2171 Error

## Token Stream Around Error

```
Token Index:  2167      2168    2169   2170    2171              2172     2173
Token Type:   Ident     Colon   Ret    Void    Identifier        Colon    Switch
Token Value:  "iftrue"  :       ret    void    "iffalse"         :        switch
Source Line:  674       674     675    675     677               677      679
              └────┬────┘       └─┬──┘        └──────┬──────────┘
              Label Definition   Return Inst  Label Definition
              iftrue:            ret void      iffalse:
```

---

## Parsing Timeline

```
Step 1: Parser encounters label "iftrue:" (tokens 2167-2168)
   [2167] Identifier("iftrue")
   [2168] Colon
   → Recognized as label, basic block created
   → Current position: token 2169

Step 2: Parser starts parsing instruction
   [2169] Ret  ← Current token
   → parse_instruction() called
   → parse_opcode() matches Token::Ret at line 656
   → Advances to token 2170
   → Returns Opcode::Ret

Step 3: Parser parses Ret instruction operands
   Current position: token 2170 (Void)

   parse_instruction_operands(Opcode::Ret) called:

   Line 735-736 check:
   ✓ !self.is_at_end()                     = true
   ✓ !self.check(&Token::RBrace)          = true
   ✓ self.peek_ahead(1) != Some(&Token::Colon)
     peek() = Token::Void (2170)
     peek_ahead(1) = Token::Identifier("iffalse") (2171)  ← Not a colon!
     = true

   → Condition passes, enters block

Step 4: Parse return type
   Line 737: let _ty = self.parse_type()?

   Current position: [2170] Void
   → parse_type() consumes "void"
   → Advances to token 2171
   → Returns Type::Void

   Current position: [2171] Identifier("iffalse")  ← ADVANCED PAST "void"

Step 5: Check if we should parse a value
   Line 740-758: matches!(self.peek(), ...)

   Current token: [2171] Identifier("iffalse")

   Checking pattern:
   ✗ Some(Token::LocalIdent(_))     - No
   ✗ Some(Token::GlobalIdent(_))    - No
   ✗ Some(Token::Integer(_))        - No
   ✗ Some(Token::Float64(_))        - No
   ✗ Some(Token::Null)              - No
   ✗ Some(Token::Undef)             - No
   ...
   ✓ Some(Token::Identifier(_))     - YES! MATCHES at line 745

   → Condition is TRUE
   → Will call parse_value()

Step 6: Call parse_value() - ERROR!
   Line 759: let _val = self.parse_value()?

   Current token: [2171] Identifier("iffalse")

   parse_value() checks (line 1634+):
   ✗ if id == "asm"                 - No
   ✗ if id == "splat"              - No
   ✗ if id == "dso_local_equivalent" - No
   ✗ if id == "no_cfi"             - No
   ✗ if id == "blockaddress"       - No

   Falls through to default case (line 1831):
   → ERROR: "Expected value, found Identifier("iffalse")"
```

---

## The Problem Visualized

```
Source Code:
   674: iftrue:
   675:   ret void      ← Parsing THIS instruction
   676:   ; CHECK: ret void
   677: iffalse:        ← Parser mistakenly tries to parse this as a value!

Token Stream:
   [2169] Ret  [2170] Void  [2171] Identifier("iffalse")  [2172] Colon
    ^           ^             ^                            ^
    |           |             |                            |
  Opcode    Return Type    LOOKS LIKE           Actually part of
  parsed    parsed         a value?             label definition!
            Advances       But it's not!
            to here
```

---

## Why the Bug Occurs

### The Lookahead Check is in the Wrong Place

```rust
// LINE 735-736: Initial check BEFORE parsing type
if !self.is_at_end() && !self.check(&Token::RBrace) &&
   self.peek_ahead(1) != Some(&Token::Colon) {  ← CHECK IS HERE
       ^
       |
   Current position: [2170] Void
   peek_ahead(1) looks at: [2171] Identifier("iffalse")
   → Not a colon, so check passes ✓

   // LINE 737: Parse type - ADVANCES POSITION!
   let _ty = self.parse_type()?;
       ^
       |
   Consumes [2170] Void
   Advances to: [2171] Identifier("iffalse")

   // LINE 740-758: Check if we should parse value
   if matches!(self.peek(), ... | Some(Token::Identifier(_)) | ...) {
       ^
       |
   Current position: [2171] Identifier("iffalse")
   → Matches! But we SHOULD check if it's followed by colon here!
   → Missing check: peek_ahead(1) != Some(&Token::Colon)

       // LINE 759: Try to parse value - ERROR!
       let _val = self.parse_value()?;  ← ERROR HERE
   }
}
```

---

## The Fix Strategy

**Option 1**: Add a lookahead check after parsing the type

```rust
Opcode::Ret => {
    if !self.is_at_end() && !self.check(&Token::RBrace) &&
       self.peek_ahead(1) != Some(&Token::Colon) {
        let _ty = self.parse_type()?;

        // ADD THIS CHECK:
        if self.peek_ahead(1) == Some(&Token::Colon) {
            // Next token is a label, don't try to parse value
            return Ok(operands);
        }

        if matches!(self.peek(), ...) {
            let _val = self.parse_value()?;
        }
    }
}
```

**Option 2**: Remove `Token::Identifier(_)` from the match pattern

```rust
// Remove line 745: Some(Token::Identifier(_))
// Only allow specific identifier types like LocalIdent and GlobalIdent
if matches!(self.peek(),
    Some(Token::LocalIdent(_)) |    ← Keep
    Some(Token::GlobalIdent(_)) |   ← Keep
    // Remove: Some(Token::Identifier(_))  ← Remove this!
    Some(Token::Integer(_)) |
    ...) {
    let _val = self.parse_value()?;
}
```

**Option 3**: Check for label in the match condition

```rust
if matches!(self.peek(), ...) &&
   self.peek_ahead(1) != Some(&Token::Colon) {  ← Add this check
    let _val = self.parse_value()?;
}
```

---

## Expected Correct Behavior

```
Step 1-4: Same as above (parse opcode, parse type "void")

Step 5: Check if we should parse a value
   Current token: [2171] Identifier("iffalse")

   NEW CHECK: Is next token a colon?
   peek_ahead(1) = [2172] Colon

   → YES, this is a label definition!
   → Do NOT call parse_value()
   → Return from parse_instruction_operands()

Step 6: Continue parsing next instruction
   Current position: [2171] Identifier("iffalse")

   → parse_instruction() returns
   → Parser moves to next basic block
   → Recognizes "iffalse:" as label
   → Success!
```
