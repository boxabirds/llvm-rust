# LLVM-Rust: Comprehensive Architecture Plan

## Executive Summary

This document outlines a complete architecture for porting LLVM to Rust. Unlike a toy implementation, this targets **full functional equivalence** with LLVM's core capabilities: parsing any valid IR, verifying correctness, running optimization passes, and generating executable code.

**Completion Criteria:**
- Parse and verify any valid LLVM IR (.ll or .bc format)
- Run standard optimization pipeline (-O0, -O1, -O2, -O3)
- Generate machine code for at least x86-64
- Pass 100% of LLVM's IR verification tests
- JIT compile and execute code
- Performance within 2x of C++ LLVM

---

## 1. Core IR Architecture

### 1.1 Critical Missing Component: Use-Def Chains

**Problem:** Current implementation lacks use-def chains - the foundation of LLVM's IR.

**Required Implementation:**
```rust
pub struct Use {
    value: Weak<ValueData>,      // What value is being used
    user: Weak<InstructionData>,  // Who is using it
    operand_no: u32,              // Which operand position
    next: Option<Box<Use>>,       // Linked list of uses
    prev: Option<Weak<Use>>,
}

pub struct Value {
    uses: Option<Box<Use>>,       // Head of use list
    // ... existing fields
}

pub struct Instruction {
    operands: Vec<Use>,           // Uses, not just Values
    // ... existing fields
}
```

**Why Critical:**
- Every optimization pass needs to traverse uses
- Dead code elimination: check if value has uses
- Constant propagation: update all uses
- SSA construction: track definitions and uses
- Without this, we have a data structure, not a compiler IR

**Test Coverage:**
- Add value as operand → use recorded
- Remove instruction → uses cleared
- Replace value → uses updated
- Query all uses of a value

---

### 1.2 Type System Enhancement

**Current State:** Basic types exist but incomplete

**Required Additions:**

1. **Type Layout Calculation**
```rust
pub struct DataLayout {
    pointer_size: u32,
    pointer_alignment: u32,
    // ... endianness, alignments, etc.
}

impl Type {
    fn size_in_bits(&self, layout: &DataLayout) -> u64;
    fn alignment(&self, layout: &DataLayout) -> u32;
    fn store_size(&self, layout: &DataLayout) -> u64;
}
```

2. **Opaque Struct Types**
```rust
pub struct StructType {
    name: Option<String>,
    body: Option<Vec<Type>>,  // None = opaque
    packed: bool,
}
```

3. **Type Equality and Equivalence**
```rust
impl Type {
    fn is_structurally_equivalent(&self, other: &Type) -> bool;
    fn is_sized(&self) -> bool;
    fn contains_pointer(&self) -> bool;
}
```

**Test Coverage:**
- Layout calculations match LLVM exactly
- Type equivalence (nominal vs structural)
- All 50+ LLVM type queries

---

### 1.3 Constant System

**Current State:** Basic constants, but missing critical features

**Required:**

1. **Constant Folding Engine**
```rust
pub fn constant_fold_binary_op(
    op: Opcode,
    lhs: &Constant,
    rhs: &Constant
) -> Option<Constant>;

pub fn constant_fold_cast(
    op: Opcode,
    value: &Constant,
    dest_ty: Type
) -> Option<Constant>;
```

2. **Constant Expressions**
- GEP constant expressions
- Cast constant expressions
- Binary operation constant expressions
- Compare constant expressions

3. **Global Values**
```rust
pub struct GlobalVariable {
    initializer: Option<Constant>,
    is_constant: bool,
    linkage: Linkage,
    visibility: Visibility,
    dll_storage_class: DLLStorageClass,
    thread_local_mode: ThreadLocalMode,
    // ... attributes
}
```

**Test Coverage:**
- All constant folding operations match LLVM
- Parse/print all constant expressions
- Global variable semantics

---

## 2. IR Input/Output

### 2.1 Text IR Parser (Complete Rewrite Needed)

**Current State:** Stub implementation, doesn't work

**Architecture:**
```
Input .ll file
    ↓
Lexer → Token stream
    ↓
Parser → AST
    ↓
Resolver → Type resolution, symbol resolution
    ↓
IR Constructor → LLVM IR in memory
```

**Components:**

1. **Lexer (5 states, 50+ token types)**
```rust
pub enum Token {
    // Keywords
    Define, Declare, Global, Constant,
    // Types
    Void, I(u32), Float, Double, Label,
    // Instructions
    Add, Sub, Mul, Load, Store, Br, Ret,
    // ... 100+ more
}
```

2. **Parser (Recursive descent, 200+ productions)**
```rust
impl Parser {
    fn parse_module(&mut self) -> Result<Module>;
    fn parse_function(&mut self) -> Result<Function>;
    fn parse_basic_block(&mut self) -> Result<BasicBlock>;
    fn parse_instruction(&mut self) -> Result<Instruction>;
    fn parse_type(&mut self) -> Result<Type>;
    fn parse_value(&mut self) -> Result<Value>;
    // ... 50+ parse methods
}
```

3. **Error Recovery**
- Don't bail on first error
- Collect multiple errors
- Provide line/column information
- Suggest fixes where possible

**Test Coverage:**
- Parse every .ll file in LLVM test suite (10,000+ files)
- Error cases: detect and report malformed IR
- Round-trip: parse → print → parse yields identical IR

---

### 2.2 Bitcode Format

**Required:**
1. **Bitstream Reader/Writer**
   - VBR (variable bit-rate) encoding
   - Block structure
   - Abbreviations

2. **Bitcode Format**
   - Module block
   - Type table
   - Value symbol table
   - Function blocks
   - Metadata blocks

3. **Versioning**
   - Support LLVM 14-19 bitcode
   - Auto-upgrade old bitcode

**Test Coverage:**
- Parse all .bc files in LLVM test suite
- Round-trip: IR → bitcode → IR is lossless
- Compatibility with real LLVM tools

---

### 2.3 IR Printer (Fix Current Implementation)

**Current Issues:**
- Incomplete instruction formatting
- Missing operand details
- No attribute printing
- No metadata printing

**Required:**
```rust
impl Module {
    fn print(&self, output: &mut dyn Write) -> Result<()>;
}

impl Function {
    fn print(&self, output: &mut dyn Write) -> Result<()>;
}

impl Instruction {
    fn print(&self, output: &mut dyn Write) -> Result<()>;
}
```

**Must Print:**
- Target triple and data layout
- Global variables with linkage/attributes
- Function signatures with attributes
- Instructions with types and metadata
- Metadata definitions
- Named types

**Test Coverage:**
- Output parses with llvm-as (LLVM's assembler)
- Diff with LLVM's output is minimal/acceptable

---

## 3. Verification & Analysis

### 3.1 IR Verifier (Critical Path)

**Current State:** Basic checks, misses 90% of issues

**Required Verification:**

1. **Module-level**
   - Valid target triple
   - Consistent data layout
   - No duplicate global names

2. **Function-level**
   - Entry block exists
   - All blocks reachable or marked unreachable
   - Single entry, properly terminated blocks

3. **Instruction-level**
   - Type correctness (e.g., add requires same-type integer/float/vector operands)
   - Operand count matches instruction
   - Operands dominate uses (SSA property)
   - Phi nodes valid (one entry per predecessor)
   - Call targets are functions
   - GEP indices are correct types

4. **SSA Form**
   - Every value defined once
   - Every use dominated by definition
   - No uses before definition in same block

5. **Control Flow**
   - All predecessors listed in phi nodes
   - Branch targets exist
   - No critical edges (or properly split)

**Implementation:**
```rust
pub struct Verifier {
    module: &Module,
    dominators: HashMap<Function, DominatorTree>,
    errors: Vec<VerifierError>,
}

impl Verifier {
    pub fn verify_module(&mut self) -> Result<(), Vec<VerifierError>>;
    fn verify_function(&mut self, func: &Function);
    fn verify_instruction(&mut self, inst: &Instruction);
    fn verify_ssa_form(&mut self, func: &Function);
}
```

**Test Coverage:**
- All valid LLVM IR passes verification
- All invalid IR in test suite caught
- Specific error messages guide fixing

---

### 3.2 Pass Manager (Complete Rewrite)

**Current State:** Toy implementation

**Required Architecture:**

```rust
pub trait Pass {
    fn name(&self) -> &str;
    fn get_required_analyses(&self) -> Vec<AnalysisID>;
    fn get_preserved_analyses(&self) -> Vec<AnalysisID>;
}

pub trait ModulePass: Pass {
    fn run(&mut self, module: &mut Module, am: &AnalysisManager) -> PreservedAnalyses;
}

pub trait FunctionPass: Pass {
    fn run(&mut self, func: &mut Function, am: &AnalysisManager) -> PreservedAnalyses;
}

pub trait LoopPass: Pass {
    fn run(&mut self, lp: &Loop, am: &AnalysisManager) -> PreservedAnalyses;
}

pub struct PassManager {
    passes: Vec<Box<dyn ModulePass>>,
    analysis_manager: AnalysisManager,
}

pub struct AnalysisManager {
    results: HashMap<(FunctionID, AnalysisID), Box<dyn Any>>,
}
```

**Key Features:**
- Analysis result caching
- Invalidation tracking
- Pass pipeline construction
- Pass timing/statistics
- Parallel pass execution (where safe)

**Test Coverage:**
- Analysis results cached correctly
- Invalidation works (changed function → invalidate analyses)
- Pass dependencies satisfied
- Pipeline matches LLVM's behavior

---

### 3.3 Core Analyses

**Required Implementations:**

1. **Dominator Tree** (COMPLETE)
```rust
pub struct DominatorTree {
    roots: Vec<BasicBlock>,
    idom: HashMap<BasicBlock, BasicBlock>,  // immediate dominator
    dom_tree_nodes: HashMap<BasicBlock, DomTreeNode>,
}

impl DominatorTree {
    pub fn dominates(&self, a: &BasicBlock, b: &BasicBlock) -> bool;
    pub fn properly_dominates(&self, a: &BasicBlock, b: &BasicBlock) -> bool;
    pub fn get_idom(&self, bb: &BasicBlock) -> Option<&BasicBlock>;
    pub fn get_children(&self, bb: &BasicBlock) -> &[BasicBlock];
}
```

2. **Post-Dominator Tree**
   - Same structure as dominator tree
   - Computed on reverse CFG

3. **Loop Analysis** (COMPLETE)
```rust
pub struct Loop {
    header: BasicBlock,
    blocks: Vec<BasicBlock>,
    backedges: Vec<(BasicBlock, BasicBlock)>,
    parent: Option<Box<Loop>>,
    sub_loops: Vec<Loop>,
}

pub struct LoopInfo {
    loops: Vec<Loop>,
    block_to_loop: HashMap<BasicBlock, Loop>,
}

impl LoopInfo {
    pub fn get_loop_for(&self, bb: &BasicBlock) -> Option<&Loop>;
    pub fn get_loop_depth(&self, bb: &BasicBlock) -> usize;
}
```

4. **Call Graph**
```rust
pub struct CallGraph {
    nodes: HashMap<Function, CallGraphNode>,
    root: CallGraphNode,  // external calling node
}

pub struct CallGraphNode {
    function: Function,
    callees: Vec<(CallSite, Function)>,
    callers: Vec<(CallSite, Function)>,
}
```

5. **Alias Analysis** (CRITICAL)
```rust
pub trait AliasAnalysis {
    fn alias(&self, loc1: MemoryLocation, loc2: MemoryLocation) -> AliasResult;
}

pub enum AliasResult {
    NoAlias,      // Definitely don't alias
    MayAlias,     // Might alias
    PartialAlias, // Definitely overlap, but not complete
    MustAlias,    // Definitely alias completely
}

// Implementations:
// - BasicAA (basic alias analysis)
// - TBAA (type-based AA)
// - ScopedAA (noalias metadata)
```

6. **Memory Dependence Analysis**
```rust
pub struct MemoryDependenceAnalysis {
    // For each load/store, what is the nearest memory operation it depends on?
    deps: HashMap<Instruction, MemDepResult>,
}

pub enum MemDepResult {
    Def(Instruction),         // Depends on this instruction
    Clobber(Instruction),     // Clobbered by this instruction
    NonLocal,                 // Depends on operations in other blocks
    Unknown,
}
```

7. **Scalar Evolution**
```rust
pub struct ScalarEvolution {
    // Analyze induction variables and trip counts
    expressions: HashMap<Value, SCEV>,
}

pub enum SCEV {
    Constant(i64),
    AddRecExpr { start: Box<SCEV>, step: Box<SCEV>, loop_: Loop },
    AddExpr(Vec<SCEV>),
    MulExpr(Vec<SCEV>),
    // ... more
}

impl ScalarEvolution {
    pub fn get_trip_count(&self, loop_: &Loop) -> Option<u64>;
}
```

**Test Coverage:**
- Each analysis has 50+ tests
- Test against LLVM's analysis results
- Performance benchmarks

---

## 4. Transformations

### 4.1 SSA Construction (Mem2Reg)

**Current State:** Stub

**Required:**
- Implement Cytron et al. algorithm
- Compute dominance frontiers
- Insert phi nodes at join points
- Rename variables to SSA form
- Promote allocas to registers

**Algorithm:**
```
1. Find promotable allocas (single-typed, not address-taken except load/store)
2. Compute dominance frontiers
3. For each alloca:
   a. Find all stores (definitions)
   b. Insert phi nodes at dominance frontier of definitions
   c. Rename: traverse dominator tree, rename uses to reaching definition
4. Remove dead allocas, loads, stores
```

**Test Coverage:**
- All promotable patterns recognized
- Output matches LLVM's mem2reg
- Handle edge cases (unreachable blocks, critical edges)

---

### 4.2 Core Optimization Passes

**Instruction Combining (InstCombine)**
- 1000+ peephole rules
- Simplify: `(x + 0) → x`, `(x * 1) → x`, `(x & x) → x`
- Strength reduction: `(x * 8) → (x << 3)`
- Canonicalization: put constants on RHS
- Fold chains: `(x + 1) + 2 → x + 3`

**Global Value Numbering (GVN)**
- Find redundant computations
- Propagate available values
- Load PRE (partial redundancy elimination)
- Use SSA and dominance

**Dead Code Elimination (DCE)**
- Aggressive: remove any instruction without uses
- Simple: only remove obviously dead code
- Handle side effects correctly

**Constant Propagation & Folding**
- Propagate constants through SSA
- Fold constant expressions
- Simplify conditionals with constant conditions

**SROA (Scalar Replacement of Aggregates)**
- Break apart structs/arrays into scalars
- Enable more optimization opportunities
- Use type-based slicing

**Loop Optimizations:**
- LICM (Loop Invariant Code Motion): hoist invariant computations
- Loop Unrolling: unroll small loops
- Loop Vectorization: SIMD-ize loops
- Loop Fusion/Distribution

**Inlining**
- Cost model (instruction count, call overhead)
- Inline small functions
- Respect noinline attribute
- Update call graph

**CFG Simplification**
- Merge basic blocks
- Remove unreachable blocks
- Thread jumps
- Simplify conditionals

**Test Coverage:**
- Each pass: 100+ test cases
- Test combinations of passes
- Performance improvements measured

---

## 5. Target Infrastructure & Code Generation

### 5.1 Target Machine Abstraction

```rust
pub trait TargetMachine {
    fn get_target_triple(&self) -> &TargetTriple;
    fn get_data_layout(&self) -> &DataLayout;
    fn get_target_features(&self) -> &[String];

    // Code generation
    fn emit_obj(&self, module: &Module, output: &mut dyn Write) -> Result<()>;
    fn emit_asm(&self, module: &Module, output: &mut dyn Write) -> Result<()>;
}

pub struct TargetTriple {
    arch: Architecture,      // x86_64, aarch64, riscv64, etc.
    vendor: Vendor,          // unknown, apple, pc
    os: OS,                  // linux, windows, darwin
    environment: Environment, // gnu, musl, msvc
}
```

---

### 5.2 Backend (At Least One: x86-64)

**Required Pipeline:**
```
LLVM IR
    ↓
Instruction Selection (ISEL)
    ↓
Machine IR (MIR)
    ↓
Register Allocation
    ↓
Instruction Scheduling
    ↓
Peephole Optimization
    ↓
Assembly/Object Emission
```

**Key Components:**

1. **Instruction Selection**
   - Pattern matching: IR → machine instructions
   - Handle calling conventions
   - Select addressing modes
   - Legalize types

2. **Register Allocation**
   - Linear scan or graph coloring
   - Spilling to stack
   - Coalescing
   - Pre-colored registers (calling convention)

3. **Machine Code Emission**
   - Encode instructions
   - Emit relocations
   - Generate ELF/Mach-O/COFF
   - Debug info (DWARF)

**x86-64 Specifics:**
- Calling conventions: System V AMD64, Microsoft x64
- Instruction encoding (ModR/M, SIB bytes, REX prefix, VEX)
- Register classes (GPR, XMM, YMM, ZMM)
- Addressing modes (base, base+offset, base+index*scale+offset)

**Test Coverage:**
- Generate code for basic programs
- Validate with native assembler
- Execute and verify results
- Performance: comparable to Clang -O0

---

## 6. JIT Compilation

### 6.1 ORC JIT v2 Architecture

```rust
pub struct ORCJIT {
    execution_session: ExecutionSession,
    object_layer: ObjectLinkingLayer,
    compile_layer: IRCompileLayer,
    optimizer_layer: OptimizeLayer,
}

pub struct ExecutionSession {
    symbol_table: HashMap<String, JITEvaluatedSymbol>,
    dylibs: Vec<JITDylib>,
}

impl ORCJIT {
    pub fn add_module(&mut self, module: Module) -> Result<JITDylib>;
    pub fn lookup(&self, name: &str) -> Result<*const u8>;
    pub fn remove_module(&mut self, dylib: JITDylib) -> Result<()>;
}
```

**Features:**
- Lazy compilation (compile on first call)
- Symbol resolution (link against process symbols)
- Memory management (allocate executable memory)
- Cleanup (free JIT'd code)

**Test Coverage:**
- JIT compile simple functions
- Call JIT'd code from Rust
- Pass arguments, return values
- Multiple modules

---

## 7. Testing Strategy

### 7.1 Test Levels

1. **Unit Tests** (5,000+ tests)
   - Every public API function
   - Edge cases and error paths
   - Mock complex dependencies

2. **Integration Tests** (1,000+ tests)
   - Parse → verify → optimize → codegen
   - Real LLVM IR files
   - End-to-end workflows

3. **Fuzz Testing**
   - Generate random valid IR
   - Generate random bitcode
   - Mutate existing IR
   - Run for days, catch crashes

4. **Differential Testing**
   - Compare with real LLVM
   - IR → optimize → compare output
   - Verify optimization soundness

5. **Performance Benchmarks**
   - Compile time: parse, optimize, codegen
   - Run time: JIT overhead
   - Memory usage
   - Compare to LLVM

6. **Regression Tests**
   - Every bug gets a test
   - Prevent regressions
   - Growing test suite

---

### 7.2 Test Data Sources

1. **LLVM Test Suite**
   - test/Assembler: parser tests (1000+ files)
   - test/Transforms: optimization tests (5000+ files)
   - test/CodeGen: backend tests (10000+ files)
   - test/Verifier: verification tests (500+ files)

2. **Real-World Projects**
   - Compile Rust programs to LLVM IR (rustc --emit=llvm-ir)
   - Compile C programs to LLVM IR (clang -S -emit-llvm)
   - Test on large codebases (LLVM itself, Chromium, etc.)

3. **Synthetic Tests**
   - Stress tests (huge functions, deep nesting)
   - Pathological cases (many predecessors, complex CFG)
   - Boundary tests (max int sizes, huge arrays)

---

## 8. Security Considerations

### 8.1 Input Validation

**Threats:**
- Maliciously crafted IR causing crashes
- Infinite loops in parser/optimizer
- Stack overflow from deep recursion
- Memory exhaustion

**Mitigations:**
- Recursion limits (parser, type resolution)
- Iteration limits (optimizer passes)
- Memory limits (bounded allocations)
- Timeouts (compilation phases)
- Input sanitization (reject malformed IR early)

---

### 8.2 Memory Safety

**Rust Advantages:**
- No use-after-free
- No buffer overflows
- No null pointer dereferences

**Careful Areas:**
- Unsafe code (JIT, raw pointers): minimize and audit
- Cyclic structures: use Weak pointers, break cycles
- FFI boundaries: validate all data crossing
- Transmute: never use without extreme justification

---

### 8.3 Fuzzing

**Tools:**
- libFuzzer integration
- AFL++ support
- Continuous fuzzing (OSS-Fuzz)

**Targets:**
- Parser (text and bitcode)
- Verifier
- Each optimization pass
- Backend

**Goals:**
- No crashes in 1 billion inputs
- No hangs (timeouts)
- No memory leaks (ASAN)

---

## 9. Performance Considerations

### 9.1 Memory Management

**Strategy:**
- Arena allocation for IR nodes (bump allocator)
- Interning (types, strings)
- Flyweight pattern (share immutable data)
- Pool allocators for fixed-size objects

**Implementation:**
```rust
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    current: Vec<T>,
}

impl<T> Arena<T> {
    pub fn alloc(&mut self, value: T) -> &mut T;
}

pub struct Module {
    arena: Arena<InstructionData>,
    // All instructions allocated from arena
}
```

---

### 9.2 Data Structure Optimization

1. **SmallVector** (common case: few elements)
```rust
pub enum SmallVec<T, const N: usize> {
    Inline([MaybeUninit<T>; N], usize),
    Heap(Vec<T>),
}
```

2. **DenseMap** (hash map for pointer keys)
```rust
pub struct DenseMap<K, V> {
    // Open addressing, linear probing
    // Optimized for pointer keys
}
```

3. **StringRef** (non-owning string view)
```rust
pub struct StringRef<'a> {
    ptr: &'a str,
}
```

---

### 9.3 Pass Performance

**Strategies:**
- Cache analysis results
- Incremental updates (don't recompute from scratch)
- Parallel analysis (where safe)
- Lazy evaluation
- Early exit (stop when no changes)

**Profiling:**
- Measure each pass
- Identify hotspots
- Optimize critical paths
- Compare with LLVM

---

## 10. Implementation Phases

### Phase 1: Foundation (Weeks 1-4)
**Goal:** Working IR with use-def chains

- [ ] Implement use-def chains
- [ ] Complete type system with layout
- [ ] Full constant system
- [ ] IR builder with all instructions
- [ ] 1000+ unit tests

**Success Criteria:**
- Can construct any valid LLVM IR programmatically
- All unit tests pass
- Memory leak free (valgrind/miri)

---

### Phase 2: I/O (Weeks 5-8)
**Goal:** Parse and print real LLVM IR

- [ ] Complete text parser
- [ ] Complete text printer
- [ ] Bitcode reader
- [ ] Bitcode writer
- [ ] 2000+ parser tests

**Success Criteria:**
- Parse 100% of LLVM test suite .ll files
- Round-trip lossless
- llvm-as accepts our output

---

### Phase 3: Verification & Analysis (Weeks 9-12)
**Goal:** Verify IR and run analyses

- [ ] Complete IR verifier
- [ ] Pass manager infrastructure
- [ ] Dominator tree
- [ ] Post-dominator tree
- [ ] Loop analysis
- [ ] Call graph
- [ ] Basic alias analysis
- [ ] Memory dependence analysis
- [ ] Scalar evolution
- [ ] 3000+ analysis tests

**Success Criteria:**
- Catches all invalid IR in LLVM test suite
- Analysis results match LLVM
- Pass manager works correctly

---

### Phase 4: Optimization (Weeks 13-20)
**Goal:** Core optimization passes work

- [ ] Mem2Reg (SSA construction)
- [ ] SROA
- [ ] InstCombine (1000+ rules)
- [ ] GVN
- [ ] DCE
- [ ] Constant propagation
- [ ] CFG simplification
- [ ] Loop optimizations (LICM, unroll)
- [ ] Inlining
- [ ] 5000+ transformation tests

**Success Criteria:**
- Optimizations produce correct code
- Performance gains measurable
- Results similar to LLVM -O2

---

### Phase 5: Code Generation (Weeks 21-28)
**Goal:** Generate x86-64 code

- [ ] Target machine infrastructure
- [ ] x86-64 backend
- [ ] Instruction selection
- [ ] Register allocation
- [ ] Assembly emission
- [ ] Object file generation
- [ ] 2000+ codegen tests

**Success Criteria:**
- Generate working binaries
- Execute correctly
- Performance within 2x of LLVM -O0

---

### Phase 6: JIT (Weeks 29-32)
**Goal:** JIT compile and execute

- [ ] ORC JIT infrastructure
- [ ] Symbol resolution
- [ ] Memory management
- [ ] Lazy compilation
- [ ] 500+ JIT tests

**Success Criteria:**
- JIT simple functions
- Call from Rust successfully
- Reasonable overhead

---

## 11. Success Metrics

### 11.1 Correctness
- [ ] Pass 100% of LLVM verifier tests
- [ ] Pass 95%+ of LLVM optimization tests
- [ ] Generate correct code for test suite
- [ ] Zero known correctness bugs

### 11.2 Performance
- [ ] Parse time: within 2x of LLVM
- [ ] Optimization time: within 2x of LLVM
- [ ] Codegen time: within 2x of LLVM
- [ ] Generated code: within 1.5x of LLVM -O0

### 11.3 Robustness
- [ ] No crashes on any valid IR
- [ ] Graceful error on invalid IR
- [ ] No memory leaks
- [ ] Fuzz for 1 billion inputs without issues

### 11.4 Compatibility
- [ ] Read LLVM 14-19 IR/bitcode
- [ ] Write compatible IR/bitcode
- [ ] Interoperate with LLVM tools

---

## 12. Risk Assessment

### High-Risk Areas

1. **Use-Def Chains**
   - Complex to implement correctly
   - Easy to create memory leaks/cycles
   - Performance critical
   - **Mitigation:** Study LLVM impl, extensive testing, fuzzing

2. **Parser Completeness**
   - LLVM IR has edge cases and legacy constructs
   - **Mitigation:** Start with LLVM test suite, add incrementally

3. **Optimization Correctness**
   - Bugs cause wrong code (worst kind of bug)
   - **Mitigation:** Differential testing vs LLVM, formal verification where possible

4. **Backend Complexity**
   - x86-64 is complicated (encoding, calling conventions)
   - **Mitigation:** Start simple (no SIMD, no FP), grow incrementally

5. **Performance**
   - Risk of being too slow to be useful
   - **Mitigation:** Profile early, optimize hot paths, use efficient data structures

### Medium-Risk Areas

1. Bitcode format changes between LLVM versions
2. Metadata completeness (debug info)
3. Thread safety in pass manager
4. JIT security (executable memory)

### Low-Risk Areas

1. Basic type system (well understood)
2. IR builder (straightforward)
3. Simple analyses (dominators, loops)

---

## 13. Open Questions

1. **Scope:** Full LLVM or subset?
   - Proposal: Core IR + one backend to start, expand later

2. **LLVM Version Compatibility:** Support which versions?
   - Proposal: Target LLVM 18+, best effort for 14-17

3. **Performance vs Correctness Trade-off:**
   - Proposal: Correctness first, optimize later

4. **API Design:** Match LLVM API or idiomatic Rust?
   - Proposal: Idiomatic Rust where possible, LLVM-like where necessary

5. **Licensing:** Apache 2.0 like LLVM?
   - Proposal: Yes

---

## 14. Conclusion

This architecture represents a **real, complete LLVM port**. Key differences from current implementation:

**Missing Critical Components:**
- ❌ Use-def chains (foundation of IR)
- ❌ Working parser (current one is stub)
- ❌ Complete verifier (current catches ~10% of errors)
- ❌ Real optimization passes (current are stubs)
- ❌ Any backend (no code generation)
- ❌ Any JIT capability

**Required Work:**
- ~30,000 lines of production code (10x current)
- ~20,000 lines of test code
- 6-8 months of focused development
- Rigorous testing and validation

**Measurement of "Port Complete":**

A port is complete when:
1. ✅ Parses 100% of valid LLVM IR
2. ✅ Verifies 100% of invalid IR is caught
3. ✅ Optimizes correctly (differential testing)
4. ✅ Generates working code for x86-64
5. ✅ JIT executes code correctly
6. ✅ Performance within 2x of LLVM
7. ✅ Zero known correctness bugs
8. ✅ Passes 10,000+ tests

Current implementation: 0/8 criteria met.

This plan provides the roadmap to 8/8.
