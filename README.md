# LLVM-Rust

A Rust implementation of LLVM's core functionality, providing memory-safe and performant compiler infrastructure.

## Overview

This project aims to port LLVM's Intermediate Representation (IR) and compiler infrastructure to Rust, leveraging Rust's memory safety guarantees while maintaining the performance and functionality of the original LLVM.

## Project Goals

- **Memory Safety**: Eliminate undefined behavior and memory vulnerabilities through Rust's type system
- **Performance**: Match or exceed C++ LLVM's performance
- **Maintainability**: Provide a cleaner, more maintainable codebase with modern language features
- **Compatibility**: Maintain API compatibility where practical for easier adoption
- **Incremental Migration**: Allow gradual migration and interoperability with existing LLVM

## Current Status

**Phase 1: Core Data Structures** ✅ In Progress

We have implemented:
- ✅ Type system (integers, floats, pointers, arrays, structs, functions)
- ✅ Value hierarchy (constants, instructions, arguments)
- ✅ Basic blocks and functions
- ✅ Module structure
- ✅ Context management
- ✅ Metadata system foundation

### Test Results

```
22 unit tests passing
All doctests passing
```

## Architecture

### Crate Structure

```
llvm-rust/
├── crates/
│   └── llvm-core/          # Core IR data structures (ACTIVE)
│       ├── context.rs       # LLVM context and type uniquing
│       ├── types.rs         # Type system implementation
│       ├── value.rs         # Value hierarchy and instructions
│       ├── module.rs        # Module, functions, basic blocks
│       └── metadata.rs      # Metadata and debug info
├── ROADMAP.md              # Detailed development roadmap
└── README.md               # This file
```

## Building

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build release version
cargo build --release
```

## Usage Example

```rust
use llvm_core::{Context, Module, Type};
use std::sync::Arc;

// Create a context
let ctx = Arc::new(Context::new());

// Create a module
let module = Module::new("my_module".to_string(), ctx.clone());

// Create types
let i32_ty = ctx.i32_type();
let ptr_ty = Type::pointer(Type::i32(), 0);

// Set module properties
module.set_target_triple("x86_64-unknown-linux-gnu".to_string());
```

## Key Features

### Type System

- Integer types with arbitrary bit widths (i1, i8, i16, i32, i64, i128, etc.)
- Floating-point types (half, float, double, fp128, x86_fp80)
- Pointer types with address spaces
- Array and vector types
- Struct types (named and anonymous)
- Function types with variadic support

### Values

- Constants (integers, floats, arrays, structs, null, undef, poison)
- Instructions (arithmetic, bitwise, memory, control flow, conversion)
- Comparison operations (integer and float predicates)
- Function calls and phi nodes

### Context Management

- Type uniquing and caching
- Arena-based allocation for efficient memory management
- Thread-safe operations using RwLock

### Module System

- Functions with linkage types
- Global variables
- Basic blocks
- Target triple and data layout
- Calling conventions

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the complete development plan.

**Upcoming Phases:**
1. ✅ Phase 1: Core Data Structures (Current)
2. Phase 2: IR Building and Parsing
3. Phase 3: Analysis Infrastructure
4. Phase 4: Optimization Passes
5. Phase 5: Code Generation Framework
6. Phase 6: Target Backend (x86-64)
7. Phase 7: Linker and Runtime

## Design Principles

1. **Safety First**: Leverage Rust's type system for compile-time guarantees
2. **Zero-Cost Abstractions**: No runtime overhead for safety
3. **Ergonomic APIs**: Idiomatic Rust interfaces
4. **Performance**: Use benchmarks to guide optimization
5. **Testing**: Comprehensive unit and integration tests
6. **Documentation**: Clear docs with examples

## Performance Considerations

- Arena allocators for IR nodes (using `bumpalo`)
- Efficient small collections with `smallvec`
- Fast hashing with `rustc-hash`
- Thread-safe primitives with `parking_lot`
- Type caching and uniquing to reduce allocations

## Contributing

This project is in active development. Contributions are welcome!

### Development Guidelines

- Follow Rust API guidelines
- Write comprehensive tests for new features
- Document public APIs with examples
- Run `cargo fmt` and `cargo clippy` before committing
- Ensure all tests pass with `cargo test`

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

This project is inspired by and aims to be compatible with LLVM, which is also licensed under the Apache License 2.0 with LLVM Exceptions.

## Resources

- [LLVM Project](https://llvm.org/)
- [LLVM IR Language Reference](https://llvm.org/docs/LangRef.html)
- [Rust Programming Language](https://www.rust-lang.org/)

## Status

**Version**: 0.1.0 (Early Development)
**Phase**: 1 - Core Data Structures
**Last Updated**: 2025-11-09

---

**Note**: This is an ambitious project in its early stages. The API is subject to change as we iterate on the design.
