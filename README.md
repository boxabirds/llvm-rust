# LLVM-Rust

A Rust implementation of LLVM's core components, providing a type-safe and idiomatic Rust API for constructing and manipulating LLVM Intermediate Representation (IR).

## Overview

This project aims to port LLVM's fundamental architecture to Rust, starting with the core IR construction and manipulation capabilities. LLVM-Rust provides:

- **Type System**: Integer types, floating-point types, pointers, arrays, structs, and function types
- **Values**: Constants, instructions, function arguments, and basic blocks
- **Instructions**: Comprehensive instruction set including arithmetic, logical, memory, and control flow operations
- **Basic Blocks**: Single-entry, single-exit code sequences
- **Functions**: Callable entities with signatures and bodies
- **Modules**: Top-level containers for organizing code
- **IR Builder**: Convenient API for programmatically constructing LLVM IR

## Features

### Implemented Components

- **Context**: Top-level container managing type uniqueness and ownership
- **Type System**:
  - Void type
  - Integer types (i1, i8, i16, i32, i64, arbitrary bit widths)
  - Floating-point types (half, float, double)
  - Pointer types
  - Array types
  - Struct types (named and anonymous)
  - Function types

- **Values**:
  - Constant integers and floats
  - Null pointers and undefined values
  - Function arguments
  - Instruction results

- **Instructions**:
  - Terminator instructions (ret, br, condbr, unreachable)
  - Binary operations (add, sub, mul, div, rem)
  - Bitwise operations (and, or, xor, shl, lshr, ashr)
  - Floating-point operations (fadd, fsub, fmul, fdiv)
  - Memory operations (alloca, load, store)
  - Comparison operations (icmp, fcmp)
  - Conversion operations (trunc, zext, sext, bitcast)
  - Other operations (call, select, phi)

- **Control Flow**:
  - Basic blocks with instruction sequences
  - Functions with multiple basic blocks
  - Module-level organization

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
llvm-rust = "0.1.0"
```

## Quick Start

Here's a simple example that creates a function to add two integers:

```rust
use llvm_rust::{Context, Module, Function, BasicBlock, Builder, Value};

fn main() {
    // Create a context
    let ctx = Context::new();

    // Create a module
    let module = Module::new("example".to_string(), ctx.clone());

    // Create function signature: i32 add(i32, i32)
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);

    // Create the function
    let function = Function::new("add".to_string(), fn_type);

    // Create arguments
    let arg_a = Value::argument(i32_type.clone(), 0, Some("a".to_string()));
    let arg_b = Value::argument(i32_type.clone(), 1, Some("b".to_string()));
    function.set_arguments(vec![arg_a.clone(), arg_b.clone()]);

    // Create basic block
    let entry_bb = BasicBlock::new(Some("entry".to_string()));
    function.add_basic_block(entry_bb.clone());

    // Build instructions
    let mut builder = Builder::new(ctx.clone());
    builder.position_at_end(entry_bb);

    let sum = builder.build_add(arg_a, arg_b, Some("sum".to_string()));
    builder.build_ret(sum);

    // Add to module and print
    module.add_function(function);
    println!("{}", module);
}
```

This generates the following LLVM IR:

```llvm
; ModuleID = 'example'

define i32 (i32, i32) @add(i32 %a, i32 %b) {
entry:
  %sum = Add
  Ret
}
```

## Examples

Run the included example:

```bash
cargo run --example simple_function
```

## Architecture

### Design Principles

1. **Type Safety**: Leverages Rust's type system to prevent common IR construction errors
2. **Memory Safety**: Uses Rust's ownership and borrowing to manage IR entities safely
3. **Ergonomics**: Provides a clean, idiomatic Rust API
4. **Interning**: Type interning ensures type uniqueness and efficient comparison
5. **Thread Safety**: Uses `Arc` and `RwLock` for safe concurrent access

### Module Structure

- `context.rs`: Context and type interning
- `types.rs`: Type system implementation
- `value.rs`: Value representation
- `instruction.rs`: Instruction definitions and opcodes
- `basic_block.rs`: Basic block implementation
- `function.rs`: Function implementation
- `module.rs`: Module and global variable management
- `builder.rs`: IR builder for convenient construction

## Development Status

This is an initial implementation focusing on core LLVM IR concepts. Current status:

- ✅ Type system (basic types, pointers, arrays, functions)
- ✅ Value system (constants, arguments, instructions)
- ✅ Instruction set (arithmetic, logic, memory, control flow)
- ✅ Basic blocks and functions
- ✅ Module structure
- ✅ IR builder
- ⚠️  IR printing (basic implementation)
- ❌ IR parsing
- ❌ IR verification
- ❌ Optimization passes
- ❌ Code generation
- ❌ JIT compilation

## Future Work

### Short Term
- Improve IR printing to match LLVM's format exactly
- Add IR verification to catch malformed IR
- Implement PHI nodes properly
- Add more comprehensive tests
- Support for metadata and debug information

### Medium Term
- IR parsing from text format
- Basic optimization passes (constant folding, dead code elimination)
- Support for more instruction types
- Attribute system for functions and arguments

### Long Term
- Complete optimization pass infrastructure
- Code generation backends (x86, ARM, etc.)
- JIT compilation support
- Full compatibility with LLVM IR

## Comparison with Original LLVM

LLVM is a massive project with millions of lines of C++ code accumulated over decades. This Rust port:

- **Scope**: Focuses on core IR manipulation, not the full LLVM infrastructure
- **Language**: Written in safe Rust vs. C++
- **API**: Idiomatic Rust API vs. C++ API
- **Size**: Thousands of lines vs. millions
- **Goal**: Educational and experimental vs. production compiler infrastructure

## Contributing

Contributions are welcome! Areas that need work:

- Additional instruction types and intrinsics
- Better IR printing and formatting
- IR verification
- Optimization passes
- Documentation and examples
- Testing

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

This project is inspired by and based on the design of [LLVM](https://llvm.org/), the production-grade compiler infrastructure project.

## Resources

- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [LLVM Programmer's Manual](https://llvm.org/docs/ProgrammersManual.html)
- [LLVM Tutorial](https://llvm.org/docs/tutorial/)
