//! LLVM Intrinsics
//!
//! Intrinsics are special built-in functions that the compiler can optimize
//! specially. This includes memory operations, math functions, and more.

use std::fmt;

/// LLVM intrinsic functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intrinsic {
    // Memory operations
    MemCpy,
    MemMove,
    MemSet,

    // Lifetime markers
    LifetimeStart,
    LifetimeEnd,

    // Arithmetic with overflow
    SAddWithOverflow,
    UAddWithOverflow,
    SSubWithOverflow,
    USubWithOverflow,
    SMulWithOverflow,
    UMulWithOverflow,

    // Saturating arithmetic
    SAddSat,
    UAddSat,
    SSubSat,
    USubSat,

    // Bit manipulation
    Bswap,
    Ctpop,
    Ctlz,
    Cttz,
    FshlRotate,
    FshrRotate,

    // Math operations
    Sqrt,
    Sin,
    Cos,
    Pow,
    Exp,
    Exp2,
    Log,
    Log10,
    Log2,
    Fma,
    Fabs,
    Copysign,
    Floor,
    Ceil,
    Trunc,
    Rint,
    Nearbyint,
    Round,

    // Min/Max
    MinNum,
    MaxNum,
    Minimum,
    Maximum,

    // Vector reductions
    VectorReduceAdd,
    VectorReduceMul,
    VectorReduceAnd,
    VectorReduceOr,
    VectorReduceXor,
    VectorReduceSMax,
    VectorReduceSMin,
    VectorReduceUMax,
    VectorReduceUMin,
    VectorReduceFAdd,
    VectorReduceFMul,
    VectorReduceFMax,
    VectorReduceFMin,

    // Saturating ops
    SShlSat,
    UShlSat,

    // Trap and debugging
    Trap,
    Debugtrap,

    // Stack operations
    StackSave,
    StackRestore,

    // Prefetch
    Prefetch,

    // Assume
    Assume,

    // Expect
    Expect,

    // Object size
    ObjectSize,

    // Overflow arithmetic
    SAddO,
    UAddO,
    SSubO,
    USubO,
    SMulO,
    UMulO,

    // Conversion
    ConvertFromFp16,
    ConvertToFp16,

    // Masked operations
    MaskedLoad,
    MaskedStore,
    MaskedGather,
    MaskedScatter,

    // Constrained FP
    ExperimentalConstrainedFAdd,
    ExperimentalConstrainedFSub,
    ExperimentalConstrainedFMul,
    ExperimentalConstrainedFDiv,

    // Coroutines
    CoroId,
    CoroAlloc,
    CoroBegin,
    CoroEnd,
    CoroSuspend,
    CoroResume,
    CoroDestroy,
    CoroPromise,

    // Experimental
    ExperimentalGCStatepoint,
    ExperimentalGCRelocate,
    ExperimentalGCResult,

    // Platform-specific
    X86SSE,
    X86AVX,
    ARMV7NEON,
    AArch64NEON,
}

impl Intrinsic {
    /// Get the name of this intrinsic as it appears in LLVM IR
    pub fn name(&self) -> &'static str {
        match self {
            Self::MemCpy => "llvm.memcpy",
            Self::MemMove => "llvm.memmove",
            Self::MemSet => "llvm.memset",
            Self::LifetimeStart => "llvm.lifetime.start",
            Self::LifetimeEnd => "llvm.lifetime.end",
            Self::SAddWithOverflow => "llvm.sadd.with.overflow",
            Self::UAddWithOverflow => "llvm.uadd.with.overflow",
            Self::SSubWithOverflow => "llvm.ssub.with.overflow",
            Self::USubWithOverflow => "llvm.usub.with.overflow",
            Self::SMulWithOverflow => "llvm.smul.with.overflow",
            Self::UMulWithOverflow => "llvm.umul.with.overflow",
            Self::SAddSat => "llvm.sadd.sat",
            Self::UAddSat => "llvm.uadd.sat",
            Self::SSubSat => "llvm.ssub.sat",
            Self::USubSat => "llvm.usub.sat",
            Self::Bswap => "llvm.bswap",
            Self::Ctpop => "llvm.ctpop",
            Self::Ctlz => "llvm.ctlz",
            Self::Cttz => "llvm.cttz",
            Self::FshlRotate => "llvm.fshl",
            Self::FshrRotate => "llvm.fshr",
            Self::Sqrt => "llvm.sqrt",
            Self::Sin => "llvm.sin",
            Self::Cos => "llvm.cos",
            Self::Pow => "llvm.pow",
            Self::Exp => "llvm.exp",
            Self::Exp2 => "llvm.exp2",
            Self::Log => "llvm.log",
            Self::Log10 => "llvm.log10",
            Self::Log2 => "llvm.log2",
            Self::Fma => "llvm.fma",
            Self::Fabs => "llvm.fabs",
            Self::Copysign => "llvm.copysign",
            Self::Floor => "llvm.floor",
            Self::Ceil => "llvm.ceil",
            Self::Trunc => "llvm.trunc",
            Self::Rint => "llvm.rint",
            Self::Nearbyint => "llvm.nearbyint",
            Self::Round => "llvm.round",
            Self::MinNum => "llvm.minnum",
            Self::MaxNum => "llvm.maxnum",
            Self::Minimum => "llvm.minimum",
            Self::Maximum => "llvm.maximum",
            Self::VectorReduceAdd => "llvm.vector.reduce.add",
            Self::VectorReduceMul => "llvm.vector.reduce.mul",
            Self::VectorReduceAnd => "llvm.vector.reduce.and",
            Self::VectorReduceOr => "llvm.vector.reduce.or",
            Self::VectorReduceXor => "llvm.vector.reduce.xor",
            Self::VectorReduceSMax => "llvm.vector.reduce.smax",
            Self::VectorReduceSMin => "llvm.vector.reduce.smin",
            Self::VectorReduceUMax => "llvm.vector.reduce.umax",
            Self::VectorReduceUMin => "llvm.vector.reduce.umin",
            Self::VectorReduceFAdd => "llvm.vector.reduce.fadd",
            Self::VectorReduceFMul => "llvm.vector.reduce.fmul",
            Self::VectorReduceFMax => "llvm.vector.reduce.fmax",
            Self::VectorReduceFMin => "llvm.vector.reduce.fmin",
            Self::SShlSat => "llvm.sshl.sat",
            Self::UShlSat => "llvm.ushl.sat",
            Self::Trap => "llvm.trap",
            Self::Debugtrap => "llvm.debugtrap",
            Self::StackSave => "llvm.stacksave",
            Self::StackRestore => "llvm.stackrestore",
            Self::Prefetch => "llvm.prefetch",
            Self::Assume => "llvm.assume",
            Self::Expect => "llvm.expect",
            Self::ObjectSize => "llvm.objectsize",
            Self::SAddO => "llvm.sadd.o",
            Self::UAddO => "llvm.uadd.o",
            Self::SSubO => "llvm.ssub.o",
            Self::USubO => "llvm.usub.o",
            Self::SMulO => "llvm.smul.o",
            Self::UMulO => "llvm.umul.o",
            Self::ConvertFromFp16 => "llvm.convert.from.fp16",
            Self::ConvertToFp16 => "llvm.convert.to.fp16",
            Self::MaskedLoad => "llvm.masked.load",
            Self::MaskedStore => "llvm.masked.store",
            Self::MaskedGather => "llvm.masked.gather",
            Self::MaskedScatter => "llvm.masked.scatter",
            Self::ExperimentalConstrainedFAdd => "llvm.experimental.constrained.fadd",
            Self::ExperimentalConstrainedFSub => "llvm.experimental.constrained.fsub",
            Self::ExperimentalConstrainedFMul => "llvm.experimental.constrained.fmul",
            Self::ExperimentalConstrainedFDiv => "llvm.experimental.constrained.fdiv",
            Self::CoroId => "llvm.coro.id",
            Self::CoroAlloc => "llvm.coro.alloc",
            Self::CoroBegin => "llvm.coro.begin",
            Self::CoroEnd => "llvm.coro.end",
            Self::CoroSuspend => "llvm.coro.suspend",
            Self::CoroResume => "llvm.coro.resume",
            Self::CoroDestroy => "llvm.coro.destroy",
            Self::CoroPromise => "llvm.coro.promise",
            Self::ExperimentalGCStatepoint => "llvm.experimental.gc.statepoint",
            Self::ExperimentalGCRelocate => "llvm.experimental.gc.relocate",
            Self::ExperimentalGCResult => "llvm.experimental.gc.result",
            Self::X86SSE => "llvm.x86.sse",
            Self::X86AVX => "llvm.x86.avx",
            Self::ARMV7NEON => "llvm.arm.neon",
            Self::AArch64NEON => "llvm.aarch64.neon",
        }
    }

    /// Check if this intrinsic is overloaded (type-parametric)
    pub fn is_overloaded(&self) -> bool {
        matches!(self,
            Self::MemCpy | Self::MemMove | Self::MemSet |
            Self::Bswap | Self::Ctpop | Self::Ctlz | Self::Cttz |
            Self::Sqrt | Self::Sin | Self::Cos | Self::Pow |
            Self::Exp | Self::Exp2 | Self::Log | Self::Log10 | Self::Log2 |
            Self::Fma | Self::Fabs | Self::Copysign |
            Self::Floor | Self::Ceil | Self::Trunc | Self::Rint |
            Self::Nearbyint | Self::Round
        )
    }

    /// Check if this intrinsic has side effects
    pub fn has_side_effects(&self) -> bool {
        matches!(self,
            Self::MemCpy | Self::MemMove | Self::MemSet |
            Self::LifetimeStart | Self::LifetimeEnd |
            Self::Trap | Self::Debugtrap |
            Self::StackRestore | Self::Prefetch
        )
    }
}

impl fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intrinsic_name() {
        assert_eq!(Intrinsic::MemCpy.name(), "llvm.memcpy");
        assert_eq!(Intrinsic::LifetimeStart.name(), "llvm.lifetime.start");
    }

    #[test]
    fn test_intrinsic_is_overloaded() {
        assert!(Intrinsic::MemCpy.is_overloaded());
        assert!(!Intrinsic::Trap.is_overloaded());
    }

    #[test]
    fn test_intrinsic_has_side_effects() {
        assert!(Intrinsic::MemCpy.has_side_effects());
        assert!(!Intrinsic::Sqrt.has_side_effects());
    }
}
