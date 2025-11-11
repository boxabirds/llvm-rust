//! x86-64 Instruction Definitions

/// x86-64 instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Opcode {
    // Data movement
    MOV,
    MOVSX,
    MOVZX,
    LEA,
    PUSH,
    POP,

    // Arithmetic
    ADD,
    SUB,
    IMUL,
    MUL,
    IDIV,
    DIV,
    INC,
    DEC,
    NEG,

    // Logical
    AND,
    OR,
    XOR,
    NOT,
    SHL,
    SHR,
    SAR,

    // Comparison
    CMP,
    TEST,

    // Control flow
    JMP,
    JE,
    JNE,
    JL,
    JLE,
    JG,
    JGE,
    CALL,
    RET,

    // Other
    NOP,
}
