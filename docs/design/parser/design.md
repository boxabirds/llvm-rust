# LLVM Rust Parser Enhancement Design

## Executive Summary

This document details the technical design for parser enhancements required to achieve 100% pass rate on the LLVM Verifier test suite. Currently at **194/338 tests passing (57.4%)**, the remaining **142 critical failures** are primarily blocked by parser limitations rather than validation logic gaps.

## Current State Analysis

### Parser Architecture
- **Location**: `src/parser.rs` (5,903 lines)
- **Core Structure**: `Parser` struct with token stream, symbol table, metadata registry
- **Key Components**:
  - `parse_module()` - Main entry point
  - `parse_global_variable()` - Global variables
  - `parse_alias()` - Alias declarations (exists but incomplete)
  - `parse_function_declaration()` / `parse_function_definition()` - Functions
  - `parse_global_initializer()` - Constant initializers

### Identified Gaps

#### 1. Alias Support (CRITICAL - affects 10+ tests)
**Status**: Parser method exists (`parse_alias` at line 766) but aliases are not properly integrated
- **Problem**: Aliases parsed but validation not triggered
- **Impact**: `alias.ll`, `bitcast-alias-address-space.ll`, and related tests

#### 2. Calling Conventions (HIGH - affects 5+ tests)
**Status**: Partial support, missing key variants
- **Problem**: `x86_intrcc` and other conventions not in enum
- **Impact**: `x86_intr.ll`, architecture-specific tests

#### 3. Array Initializers (HIGH - affects 5+ tests)
**Status**: Arrays parsed but represented incorrectly
- **Problem**: All arrays become `ZeroInitializer` instead of `ConstantArray`
- **Impact**: `llvm.used-invalid-init.ll`, `llvm.used-invalid-init2.ll`

#### 4. Operand Bundles (MEDIUM - affects 10+ tests)
**Status**: Partially parsed, not stored
- **Problem**: Bundles skipped, not available for validation
- **Impact**: `assume-bundles.ll`, `preallocated-invalid.ll`, `invalid-statepoint.ll`

#### 5. Metadata Structure (MEDIUM - affects 20+ tests)
**Status**: Metadata registry exists but structure not preserved
- **Problem**: Metadata parsed as generic nodes, losing type information
- **Impact**: Debug info tests, annotation tests

## Detailed Design

### 1. Alias Support Enhancement

#### Problem Statement
While `parse_alias()` exists (line 766), parsed aliases are not being added to the module's alias vector consistently, preventing validation code from executing.

#### Root Cause Analysis
```rust
// Current flow (src/parser.rs:204):
let alias = self.parse_alias()?;
module.add_alias(alias).map_err(|e| ParseError::InvalidSyntax { ... })?;
```
The issue is likely in the lookahead logic (lines 169-204) that determines whether something is an alias.

#### Solution Design

##### Data Structures
No changes needed - `Alias` struct in `src/module.rs` is complete:
```rust
pub struct Alias {
    pub name: String,
    pub ty: Type,
    pub aliasee: Value,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub dll_storage_class: DLLStorageClass,
    pub thread_local_mode: ThreadLocalMode,
    pub unnamed_addr: UnnamedAddr,
}
```

##### Method Changes

**File**: `src/parser.rs`

**Method**: `parse_module()` (around lines 165-210)
```rust
// BEFORE: Lookahead logic is complex and may miss aliases
if self.peek() == Some(&Token::GlobalIdent(_)) {
    // Complex lookahead...
    if /* conditions */ {
        let alias = self.parse_alias()?;
        module.add_alias(alias)?;
    }
}

// AFTER: Simplified approach
if self.peek() == Some(&Token::GlobalIdent(_)) {
    let checkpoint = self.current;
    let name = self.expect_global_ident()?;

    if self.match_token(&Token::Equal) {
        // Save position after '='
        let after_eq = self.current;

        // Skip linkage/visibility/etc.
        self.skip_global_attributes();

        // Check for 'alias' keyword
        if self.match_token(&Token::Alias) {
            // Reset to after '=' and parse full alias
            self.current = after_eq;
            let alias = self.parse_alias_after_name(name)?;
            module.add_alias(alias)?;
            continue;
        } else {
            // Not an alias, reset and parse as global variable
            self.current = checkpoint;
            let global = self.parse_global_variable()?;
            module.add_global(global);
            continue;
        }
    }
}
```

**New Method**: `skip_global_attributes()` at line ~850
```rust
fn skip_global_attributes(&mut self) {
    // Skip common global attributes to reach 'alias' keyword
    loop {
        match self.peek() {
            Some(Token::Private) | Some(Token::Internal) |
            Some(Token::External) | Some(Token::Weak) |
            Some(Token::Linkonce) | Some(Token::Linkonce_odr) |
            Some(Token::Weak_odr) | Some(Token::Available_externally) |
            Some(Token::Extern_weak) | Some(Token::Common) |
            Some(Token::Appending) | Some(Token::Hidden) |
            Some(Token::Protected) | Some(Token::Default) |
            Some(Token::Dllimport) | Some(Token::Dllexport) |
            Some(Token::Thread_local) | Some(Token::Localdynamic) |
            Some(Token::Initialexec) | Some(Token::Localexec) |
            Some(Token::Unnamed_addr) | Some(Token::Local_unnamed_addr) => {
                self.advance();
            }
            _ => break,
        }
    }
}
```

**Modified Method**: `parse_alias()` becomes `parse_alias_after_name()` at line 766
```rust
// Add parameter for already-parsed name
fn parse_alias_after_name(&mut self, name: String) -> ParseResult<crate::module::Alias> {
    // Current implementation starts at line 770 expecting name already consumed
    // No changes to internal logic, just rename and add name parameter
    // Remove: let name = self.expect_global_ident()?;
    // Remove: self.consume(&Token::Equal)?;
    // ... rest of method unchanged
}
```

#### Testing Strategy
- Run `alias.ll` - should now reject cycles and invalid linkage
- Run `bitcast-alias-address-space.ll` - should validate address space consistency
- Verify `module.aliases()` returns non-empty vector

---

### 2. Calling Convention Support

#### Problem Statement
Missing calling conventions prevent proper validation of architecture-specific code. `x86_intrcc` is used in tests but not in the parser's calling convention mapping.

#### Root Cause Analysis
```rust
// src/function.rs - CallingConvention enum
pub enum CallingConvention {
    // ... X86_StdCall, X86_FastCall, etc. exist
    // Missing: X86_INTR, RISCV_VLS_CC, and others
}

// src/parser.rs:4325-4340 - calling convention parsing
"x86_stdcallcc" => Some(CallingConvention::X86_StdCall),
// Missing: "x86_intrcc" mapping
```

#### Solution Design

##### Data Structure Changes

**File**: `src/function.rs` (line ~200)

Add to `CallingConvention` enum:
```rust
pub enum CallingConvention {
    // ... existing variants ...
    X86_RegCall,               // x86_regcallcc (exists)

    // ADD THESE:
    X86_INTR,                  // x86_intrcc - interrupt handler
    RISCV_VLS_CC,              // riscv_vls_cc - vector length specific
    M68k_INTR,                 // m68k_intrcc
    M68k_RTD,                  // m68k_rtdcc
    AVR_INTR,                  // avr_intrcc
    AVR_SIGNAL,                // avr_signalcc
    MSP430_INTR,               // msp430_intrcc
    AARCH64_SVE_Vector_PCS_Preserve, // aarch64_sve_vector_pcs_preserve

    // ... rest of enum
}
```

**Rationale**: Each calling convention has specific ABI rules that require validation. Missing conventions cause tests to be skipped or incorrectly parsed.

##### Method Changes

**File**: `src/parser.rs` (around line 4340)

Extend calling convention parsing in `parse_calling_convention()`:
```rust
fn parse_calling_convention(&mut self) -> ParseResult<CallingConvention> {
    // ... existing code ...

    let cc_map = match cc_name {
        // ... existing mappings ...
        "x86_regcallcc" => Some(CallingConvention::X86_RegCall),

        // ADD THESE:
        "x86_intrcc" => Some(CallingConvention::X86_INTR),
        "riscv_vls_cc" => Some(CallingConvention::RISCV_VLS_CC),
        "m68k_intrcc" => Some(CallingConvention::M68k_INTR),
        "m68k_rtdcc" => Some(CallingConvention::M68k_RTD),
        "avr_intrcc" => Some(CallingConvention::AVR_INTR),
        "avr_signalcc" => Some(CallingConvention::AVR_SIGNAL),
        "msp430_intrcc" => Some(CallingConvention::MSP430_INTR),
        "aarch64_sve_vector_pcs_preserve" => Some(CallingConvention::AARCH64_SVE_Vector_PCS_Preserve),

        _ => None,
    };

    // ... rest of method
}
```

##### Validation Integration

**File**: `src/verification.rs` (around line 865)

Add x86_intrcc validation after varargs check:
```rust
// After existing varargs validation (line 865)

// x86_intrcc requires all parameters to be pointers with byval attribute
if cc == CallingConvention::X86_INTR {
    let attrs = function.attributes();
    for (i, param) in function.arguments().iter().enumerate() {
        let param_type = param.get_type();

        // Parameter must be a pointer
        if !param_type.is_pointer() {
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Calling convention parameter requires byval".to_string(),
                location: format!("ptr @{}", fn_name),
            });
            continue;
        }

        // Parameter must have byval attribute
        if i < attrs.parameter_attributes.len() {
            let param_attrs = &attrs.parameter_attributes[i];
            if param_attrs.byval.is_none() {
                self.errors.push(VerificationError::InvalidInstruction {
                    reason: "Calling convention parameter requires byval".to_string(),
                    location: format!("ptr @{}", fn_name),
                });
            }
        } else {
            // No attributes for this parameter - missing byval
            self.errors.push(VerificationError::InvalidInstruction {
                reason: "Calling convention parameter requires byval".to_string(),
                location: format!("ptr @{}", fn_name),
            });
        }
    }
}
```

#### Testing Strategy
- Run `x86_intr.ll` - should reject non-byval parameters
- Verify calling convention preserved through parse/verify cycle
- Check that validation catches incorrect usage

---

### 3. Array Initializer Representation

#### Problem Statement
Array initializers are incorrectly represented as `ZeroInitializer` instead of `ConstantArray`, preventing validation of individual array elements (e.g., detecting null pointers in `llvm.used`).

#### Root Cause Analysis
```rust
// src/parser.rs:866 - parse_global_initializer
match self.peek() {
    Some(Token::Zeroinitializer) => {
        // Correctly returns ZeroInitializer
        Ok(Value::zero_initializer(ty.clone()))
    },
    Some(Token::LBracket) => {
        // Array constant parsing
        // Problem: Parsed values discarded, returns ZeroInitializer
        let _ty = self.parse_type()?;
        let _val = self.parse_value()?;
        // Elements never collected!
    }
}
```

The array parsing code (line ~930) parses elements but doesn't collect them into a `ConstantArray`.

#### Solution Design

##### Method Changes

**File**: `src/parser.rs`

**Method**: `parse_global_initializer()` at line 866

Replace array parsing section:
```rust
// BEFORE (line ~930):
Token::LBracket => {
    self.advance(); // consume '['
    while !self.check(&Token::RBracket) && !self.is_at_end() {
        let _ty = self.parse_type()?;
        let _val = self.parse_value()?;
        if !self.match_token(&Token::Comma) {
            break;
        }
    }
    self.consume(&Token::RBracket)?;
    // Returns zeroinitializer - WRONG!
    Ok(Value::zero_initializer(ty.clone()))
}

// AFTER:
Token::LBracket => {
    self.advance(); // consume '['

    let mut elements = Vec::new();

    // Parse array elements
    while !self.check(&Token::RBracket) && !self.is_at_end() {
        // Parse element type
        let elem_ty = self.parse_type()?;

        // Parse element value
        let elem_val = self.parse_constant_value(&elem_ty)?;
        elements.push(elem_val);

        // Check for comma separator
        if !self.match_token(&Token::Comma) {
            break;
        }
    }

    self.consume(&Token::RBracket)?;

    // Return ConstantArray with actual elements
    Ok(Value::const_array(ty.clone(), elements))
}
```

**New Method**: `parse_constant_value()` at line ~1000
```rust
fn parse_constant_value(&mut self, ty: &Type) -> ParseResult<Value> {
    // Parse a constant value of a given type
    // Similar to parse_value but restricted to constants
    match self.peek() {
        Some(Token::Integer(n)) => {
            let val = *n;
            self.advance();
            Ok(Value::const_int(ty.clone(), val as i64, None))
        },
        Some(Token::Float(f)) => {
            let val = *f;
            self.advance();
            Ok(Value::const_float(ty.clone(), val, None))
        },
        Some(Token::Null) => {
            self.advance();
            Ok(Value::const_null(ty.clone()))
        },
        Some(Token::Zeroinitializer) => {
            self.advance();
            Ok(Value::zero_initializer(ty.clone()))
        },
        Some(Token::Undef) => {
            self.advance();
            Ok(Value::undef(ty.clone()))
        },
        Some(Token::True) => {
            self.advance();
            Ok(Value::const_int(ty.clone(), 1, None))
        },
        Some(Token::False) => {
            self.advance();
            Ok(Value::const_int(ty.clone(), 0, None))
        },
        Some(Token::GlobalIdent(_)) => {
            // Global reference
            self.parse_value()
        },
        Some(Token::LBracket) => {
            // Nested array
            self.parse_global_initializer(ty)
        },
        Some(Token::LBrace) => {
            // Struct constant
            self.parse_struct_constant(ty)
        },
        Some(Token::LAngle) => {
            // Vector constant
            self.parse_vector_constant(ty)
        },
        _ => {
            // Try constant expression
            self.parse_constant_expression()
        }
    }
}
```

##### Data Structure Verification

**File**: `src/value.rs` (line 34)

Verify `ConstantArray` variant exists:
```rust
pub enum ValueKind {
    // ...
    ConstantArray { elements: Vec<Value> },  // Already exists!
    // ...
}
```

**File**: `src/value.rs` (line 144)

Verify constructor exists:
```rust
pub fn const_array(ty: Type, elements: Vec<Value>) -> Self {
    Self::new(ty, ValueKind::ConstantArray { elements }, None)
}
```
If missing, add this method.

#### Testing Strategy
- Run `llvm.used-invalid-init.ll` - should detect zeroinitializer
- Run `llvm.used-invalid-init2.ll` - should detect null elements
- Verify array globals have proper `ConstantArray` representation
- Test nested arrays: `[[2 x i32] [i32 1, i32 2], [2 x i32] [i32 3, i32 4]]`

---

### 4. Operand Bundle Support

#### Problem Statement
Operand bundles (e.g., `["deopt"(...)]`, `["gc-live"(...)]`) are parsed but not stored in the instruction representation, preventing validation of bundle-specific rules.

#### Root Cause Analysis
```rust
// src/parser.rs:1653 - operand bundles mentioned in comment
// Handle operand bundles: ["bundle"(args...)]

// src/parser.rs:2306 - bundles skipped
// Handle optional operand bundles
```

Bundles are recognized but discarded during parsing.

#### Solution Design

##### Data Structure Changes

**File**: `src/instruction.rs` (estimate line ~50)

Add operand bundle field to `Instruction`:
```rust
pub struct Instruction {
    // ... existing fields ...
    metadata: HashMap<String, Metadata>,

    // ADD THIS:
    operand_bundles: Vec<OperandBundle>,
}

// ADD THIS struct:
#[derive(Debug, Clone)]
pub struct OperandBundle {
    pub tag: String,          // e.g., "deopt", "gc-live", "funclet"
    pub inputs: Vec<Value>,   // Bundle arguments
}

impl Instruction {
    // ADD THIS method:
    pub fn operand_bundles(&self) -> &[OperandBundle] {
        &self.operand_bundles
    }

    pub fn add_operand_bundle(&mut self, bundle: OperandBundle) {
        self.operand_bundles.push(bundle);
    }
}
```

##### Method Changes

**File**: `src/parser.rs`

**Method**: `parse_call_instruction()` at line ~1650

Add bundle parsing after operands:
```rust
fn parse_call_instruction(&mut self) -> ParseResult<Instruction> {
    // ... existing operand parsing ...

    // Parse optional operand bundles: [ "tag"(args...), "tag2"(args...) ]
    let mut bundles = Vec::new();

    if self.match_token(&Token::LBracket) {
        loop {
            // Expect bundle tag (string literal)
            let tag = match self.peek() {
                Some(Token::StringLiteral(s)) => {
                    let tag = s.clone();
                    self.advance();
                    tag
                },
                _ => return Err(ParseError::InvalidSyntax {
                    message: "Expected operand bundle tag (string)".to_string(),
                    position: self.current,
                }),
            };

            // Expect '('
            self.consume(&Token::LParen)?;

            // Parse bundle inputs
            let mut inputs = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                // Bundle inputs can be typed values
                if self.is_type_token() {
                    let ty = self.parse_type()?;
                    let val = self.parse_value_of_type(&ty)?;
                    inputs.push(val);
                } else {
                    // Untyped value - try to infer
                    inputs.push(self.parse_value()?);
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RParen)?;

            bundles.push(OperandBundle { tag, inputs });

            // Check for more bundles
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RBracket)?;
    }

    // Create instruction with bundles
    let mut inst = Instruction::new(Opcode::Call, operands, result);
    for bundle in bundles {
        inst.add_operand_bundle(bundle);
    }

    Ok(inst)
}
```

**Method**: `parse_invoke_instruction()` at line ~2300

Add similar bundle parsing:
```rust
fn parse_invoke_instruction(&mut self) -> ParseResult<Instruction> {
    // ... existing parsing ...

    // Parse optional operand bundles (same logic as call)
    let mut bundles = Vec::new();
    if self.match_token(&Token::LBracket) {
        // Same bundle parsing logic as in parse_call_instruction
        // (extract to parse_operand_bundles() helper method)
    }

    let mut inst = Instruction::new(Opcode::Invoke, operands, result);
    for bundle in bundles {
        inst.add_operand_bundle(bundle);
    }

    Ok(inst)
}
```

**New Method**: `parse_operand_bundles()` at line ~2400
```rust
fn parse_operand_bundles(&mut self) -> ParseResult<Vec<OperandBundle>> {
    // Extract common bundle parsing logic
    let mut bundles = Vec::new();

    // Expect already consumed '['
    loop {
        let tag = match self.peek() {
            Some(Token::StringLiteral(s)) => {
                let tag = s.clone();
                self.advance();
                tag
            },
            _ => return Err(ParseError::InvalidSyntax {
                message: "Expected operand bundle tag".to_string(),
                position: self.current,
            }),
        };

        self.consume(&Token::LParen)?;

        let mut inputs = Vec::new();
        while !self.check(&Token::RParen) && !self.is_at_end() {
            if self.is_type_token() {
                let ty = self.parse_type()?;
                let val = self.parse_value_of_type(&ty)?;
                inputs.push(val);
            } else {
                inputs.push(self.parse_value()?);
            }

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RParen)?;
        bundles.push(OperandBundle { tag, inputs });

        if !self.match_token(&Token::Comma) {
            break;
        }
    }

    Ok(bundles)
}
```

##### Validation Integration

**File**: `src/verification.rs` (around line 2300)

Add bundle validation for deoptimize intrinsic:
```rust
// In verify_intrinsic_call method
if intrinsic_name == "llvm.experimental.deoptimize" {
    let bundles = inst.operand_bundles();

    // Must have exactly one "deopt" bundle
    let deopt_bundles: Vec<_> = bundles.iter()
        .filter(|b| b.tag == "deopt")
        .collect();

    if deopt_bundles.len() != 1 {
        self.errors.push(VerificationError::InvalidInstruction {
            reason: "experimental_deoptimize must have exactly one \"deopt\" operand bundle".to_string(),
            location: "call instruction".to_string(),
        });
    }

    // experimental_deoptimize cannot be invoked
    if inst.opcode() == Opcode::Invoke {
        self.errors.push(VerificationError::InvalidInstruction {
            reason: "experimental_deoptimize cannot be invoked".to_string(),
            location: "invoke instruction".to_string(),
        });
    }
}
```

#### Testing Strategy
- Run `assume-bundles.ll` - validate assume bundle constraints
- Run `deoptimize-intrinsic.ll` - validate deopt bundle rules
- Run `preallocated-invalid.ll` - validate preallocated bundles
- Run `invalid-statepoint.ll` - validate gc-live bundles

---

### 5. Metadata Structure Preservation

#### Problem Statement
Metadata is parsed into a generic registry but type-specific structure (DILocation, DISubprogram, etc.) is lost, preventing validation of debug info constraints.

#### Root Cause Analysis
```rust
// src/parser.rs:48 - metadata registry
metadata_registry: std::collections::HashMap<String, crate::metadata::Metadata>,

// src/metadata.rs - Metadata is too generic
pub struct Metadata {
    // Loses specific debug info node types
}
```

#### Solution Design

##### Data Structure Changes

**File**: `src/metadata.rs` (estimate line ~20)

Add typed metadata variants:
```rust
#[derive(Debug, Clone)]
pub enum Metadata {
    // Generic metadata
    Node {
        operands: Vec<MetadataValue>,
    },
    String(String),
    Value(Value),

    // ADD THESE typed variants:
    DILocation {
        line: u32,
        column: u32,
        scope: Box<Metadata>,
        inlined_at: Option<Box<Metadata>>,
    },
    DISubprogram {
        name: String,
        linkage_name: Option<String>,
        scope: Option<Box<Metadata>>,
        file: Option<Box<Metadata>>,
        line: u32,
        ty: Option<Box<Metadata>>,
        is_local: bool,
        is_definition: bool,
        scope_line: u32,
        retained_nodes: Vec<Metadata>,
        // ... other fields
    },
    DICompositeType {
        tag: u32,
        name: String,
        file: Option<Box<Metadata>>,
        line: u32,
        base_type: Option<Box<Metadata>>,
        size: u64,
        align: u64,
        elements: Vec<Metadata>,
        flags: u32,
        // ... other fields
    },
    DISubrange {
        count: Option<MetadataValue>,
        lower_bound: Option<MetadataValue>,
        upper_bound: Option<MetadataValue>,
        stride: Option<MetadataValue>,
    },
    DIBasicType {
        name: String,
        size: u64,
        encoding: u32,
    },
    // ... other DI types
}

#[derive(Debug, Clone)]
pub enum MetadataValue {
    Int(i64),
    Metadata(Box<Metadata>),
    Null,
}
```

##### Method Changes

**File**: `src/parser.rs`

**Method**: `parse_metadata_node()` at line ~3500 (estimated)

Add dispatch based on node name:
```rust
fn parse_metadata_node(&mut self) -> ParseResult<Metadata> {
    // Expect '!'
    self.consume(&Token::Exclaim)?;

    // Check for named metadata node
    if let Some(Token::Identifier(name)) = self.peek() {
        let name_str = name.clone();
        self.advance();

        // Dispatch based on metadata type
        match name_str.as_str() {
            "DILocation" => self.parse_dilocation(),
            "DISubprogram" => self.parse_disubprogram(),
            "DICompositeType" => self.parse_dicomposite_type(),
            "DISubrange" => self.parse_disubrange(),
            "DIBasicType" => self.parse_dibasic_type(),
            "DIFile" => self.parse_difile(),
            "DICompileUnit" => self.parse_dicompile_unit(),
            "DILocalVariable" => self.parse_dilocal_variable(),
            "DIExpression" => self.parse_diexpression(),
            // ... other DI types
            _ => {
                // Unknown metadata type - parse as generic node
                self.parse_generic_metadata_node()
            }
        }
    } else {
        // Numbered metadata reference: !0, !1, etc.
        self.parse_metadata_reference()
    }
}
```

**New Methods**: `parse_dilocation()`, `parse_disubrange()`, etc. at line ~3600

```rust
fn parse_dilocation(&mut self) -> ParseResult<Metadata> {
    // Expect '('
    self.consume(&Token::LParen)?;

    let mut line = 0;
    let mut column = 0;
    let mut scope = None;
    let mut inlined_at = None;

    // Parse fields
    while !self.check(&Token::RParen) && !self.is_at_end() {
        // Expect field name
        let field_name = self.expect_identifier()?;
        self.consume(&Token::Colon)?;

        match field_name.as_str() {
            "line" => {
                line = self.parse_integer()? as u32;
            },
            "column" => {
                column = self.parse_integer()? as u32;
            },
            "scope" => {
                scope = Some(Box::new(self.parse_metadata_value()?));
            },
            "inlinedAt" => {
                inlined_at = Some(Box::new(self.parse_metadata_value()?));
            },
            _ => {
                // Skip unknown field
                self.skip_metadata_value()?;
            }
        }

        if !self.match_token(&Token::Comma) {
            break;
        }
    }

    self.consume(&Token::RParen)?;

    Ok(Metadata::DILocation {
        line,
        column,
        scope: scope.ok_or_else(|| ParseError::InvalidSyntax {
            message: "DILocation requires scope field".to_string(),
            position: self.current,
        })?,
        inlined_at,
    })
}

fn parse_disubrange(&mut self) -> ParseResult<Metadata> {
    self.consume(&Token::LParen)?;

    let mut count = None;
    let mut lower_bound = None;
    let mut upper_bound = None;
    let mut stride = None;

    while !self.check(&Token::RParen) && !self.is_at_end() {
        let field_name = self.expect_identifier()?;
        self.consume(&Token::Colon)?;

        match field_name.as_str() {
            "count" => {
                count = Some(self.parse_metadata_value_raw()?);
            },
            "lowerBound" => {
                lower_bound = Some(self.parse_metadata_value_raw()?);
            },
            "upperBound" => {
                upper_bound = Some(self.parse_metadata_value_raw()?);
            },
            "stride" => {
                stride = Some(self.parse_metadata_value_raw()?);
            },
            _ => {
                self.skip_metadata_value()?;
            }
        }

        if !self.match_token(&Token::Comma) {
            break;
        }
    }

    self.consume(&Token::RParen)?;

    Ok(Metadata::DISubrange {
        count,
        lower_bound,
        upper_bound,
        stride,
    })
}
```

##### Validation Integration

**File**: `src/verification.rs` (around line 2400)

Update `verify_disubrange()` to use typed metadata:
```rust
fn verify_disubrange(&mut self, metadata: &Metadata) {
    // Now we have typed metadata!
    if let Metadata::DISubrange { count, lower_bound, upper_bound, stride } = metadata {
        // Check count OR upperBound, not both
        if count.is_some() && upper_bound.is_some() {
            self.errors.push(VerificationError::InvalidDebugInfo {
                reason: "Subrange can have any one of count or upperBound".to_string(),
                location: "DISubrange".to_string(),
            });
        }

        // Validate count if present
        if let Some(MetadataValue::Int(_)) = count {
            // Valid: signed constant
        } else if let Some(MetadataValue::Metadata(m)) = count {
            // Check if it's DIVariable or DIExpression
            if !matches!(**m, Metadata::DILocalVariable { .. } | Metadata::DIExpression { .. }) {
                self.errors.push(VerificationError::InvalidDebugInfo {
                    reason: "Count must be signed constant or DIVariable or DIExpression".to_string(),
                    location: "DISubrange".to_string(),
                });
            }
        }

        // Similar validation for lowerBound, upperBound, stride
    }
}
```

#### Testing Strategy
- Run `dbg-invalid-vector.ll` - validate DICompositeType elements
- Run `invalid-disubrange-count-node.ll` - validate count field types
- Run `dbg-invalid-retaintypes.ll` - validate retainedTypes field
- Run `cc-flags.ll` - validate DICompositeType flags

---

## Implementation Strategy

### Phase 1: Foundation (Week 1)
1. Alias support enhancement
2. Calling convention additions
3. Array initializer fix

**Target**: +15 tests passing (194 → 209)

### Phase 2: Advanced Features (Week 2)
4. Operand bundle support
5. Basic metadata structure (DISubrange, DIBasicType)

**Target**: +20 tests passing (209 → 229)

### Phase 3: Complete Metadata (Week 3)
6. Full debug info metadata types
7. Metadata validation integration

**Target**: +30 tests passing (229 → 259)

### Phase 4: Edge Cases (Week 4)
8. Remaining parser gaps
9. Complex validation scenarios

**Target**: +40+ tests passing (259 → 300+)

## Risk Mitigation

### Backward Compatibility
- All changes are additive (new enum variants, new fields)
- Existing code continues to work
- Gradual rollout per phase

### Testing Strategy
- Unit tests for each parser method
- Integration tests with LLVM test suite
- Regression testing after each phase

### Performance Considerations
- Parser complexity increases slightly
- Metadata structure overhead acceptable (debug builds only)
- No impact on non-debug code

## Success Metrics

- **Primary**: Test pass rate 194/338 (57.4%) → 338/338 (100%)
- **Secondary**: Critical failures 142 → 0
- **Code Quality**: All changes with unit tests
- **Documentation**: Updated design docs

## Appendix

### File Change Summary
- `src/function.rs`: +10 CallingConvention variants
- `src/parser.rs`: ~500 lines added/modified
- `src/metadata.rs`: +15 Metadata variants, +200 lines
- `src/instruction.rs`: +OperandBundle struct
- `src/verification.rs`: Enhanced validation logic
- `src/value.rs`: Minor enhancements

### Dependencies
- No new external dependencies
- Existing lexer/token system sufficient
- Type system complete

### Open Questions
1. Should metadata preserve all fields or just validation-critical ones?
   - **Decision**: Preserve all for completeness
2. Performance impact of typed metadata?
   - **Mitigation**: Profile after Phase 3
3. Handling of unknown metadata types?
   - **Strategy**: Fall back to generic node, log warning
