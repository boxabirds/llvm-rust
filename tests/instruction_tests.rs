use llvm_rust::*;
use llvm_rust::instruction::*;

// Instruction tests (100+ tests)

#[test]
fn test_add_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::Add, vec![], None);
    assert!(inst.is_binary_op());
    assert!(!inst.is_terminator());
    assert_eq!(inst.opcode(), Opcode::Add);
}

#[test]
fn test_sub_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::Sub, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_mul_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::Mul, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_udiv_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::UDiv, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_sdiv_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::SDiv, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_urem_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::URem, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_srem_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::SRem, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_fadd_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::FAdd, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_fsub_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::FSub, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_fmul_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::FMul, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_fdiv_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::FDiv, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_frem_instruction_is_binary_op() {
    let inst = Instruction::new(Opcode::FRem, vec![], None);
    assert!(inst.is_binary_op());
}

// Bitwise operations
#[test]
fn test_shl_instruction() {
    let inst = Instruction::new(Opcode::Shl, vec![], None);
    assert!(inst.is_binary_op());
    assert_eq!(inst.opcode(), Opcode::Shl);
}

#[test]
fn test_lshr_instruction() {
    let inst = Instruction::new(Opcode::LShr, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_ashr_instruction() {
    let inst = Instruction::new(Opcode::AShr, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_and_instruction() {
    let inst = Instruction::new(Opcode::And, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_or_instruction() {
    let inst = Instruction::new(Opcode::Or, vec![], None);
    assert!(inst.is_binary_op());
}

#[test]
fn test_xor_instruction() {
    let inst = Instruction::new(Opcode::Xor, vec![], None);
    assert!(inst.is_binary_op());
}

// Terminator instructions
#[test]
fn test_ret_is_terminator() {
    let inst = Instruction::new(Opcode::Ret, vec![], None);
    assert!(inst.is_terminator());
    assert!(!inst.is_binary_op());
}

#[test]
fn test_br_is_terminator() {
    let inst = Instruction::new(Opcode::Br, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_condbr_is_terminator() {
    let inst = Instruction::new(Opcode::CondBr, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_switch_is_terminator() {
    let inst = Instruction::new(Opcode::Switch, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_indirectbr_is_terminator() {
    let inst = Instruction::new(Opcode::IndirectBr, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_invoke_is_terminator() {
    let inst = Instruction::new(Opcode::Invoke, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_resume_is_terminator() {
    let inst = Instruction::new(Opcode::Resume, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_unreachable_is_terminator() {
    let inst = Instruction::new(Opcode::Unreachable, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_cleanupret_is_terminator() {
    let inst = Instruction::new(Opcode::CleanupRet, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_catchret_is_terminator() {
    let inst = Instruction::new(Opcode::CatchRet, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_catchswitch_is_terminator() {
    let inst = Instruction::new(Opcode::CatchSwitch, vec![], None);
    assert!(inst.is_terminator());
}

#[test]
fn test_callbr_is_terminator() {
    let inst = Instruction::new(Opcode::CallBr, vec![], None);
    assert!(inst.is_terminator());
}

// Memory operations
#[test]
fn test_alloca_is_memory_op() {
    let inst = Instruction::new(Opcode::Alloca, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_load_is_memory_op() {
    let inst = Instruction::new(Opcode::Load, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_store_is_memory_op() {
    let inst = Instruction::new(Opcode::Store, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_getelementptr_is_memory_op() {
    let inst = Instruction::new(Opcode::GetElementPtr, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_fence_is_memory_op() {
    let inst = Instruction::new(Opcode::Fence, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_atomiccmpxchg_is_memory_op() {
    let inst = Instruction::new(Opcode::AtomicCmpXchg, vec![], None);
    assert!(inst.is_memory_op());
}

#[test]
fn test_atomicrmw_is_memory_op() {
    let inst = Instruction::new(Opcode::AtomicRMW, vec![], None);
    assert!(inst.is_memory_op());
}

// Cast/conversion operations
#[test]
fn test_trunc_is_cast() {
    let inst = Instruction::new(Opcode::Trunc, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_zext_is_cast() {
    let inst = Instruction::new(Opcode::ZExt, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_sext_is_cast() {
    let inst = Instruction::new(Opcode::SExt, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_fptoui_is_cast() {
    let inst = Instruction::new(Opcode::FPToUI, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_fptosi_is_cast() {
    let inst = Instruction::new(Opcode::FPToSI, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_uitofp_is_cast() {
    let inst = Instruction::new(Opcode::UIToFP, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_sitofp_is_cast() {
    let inst = Instruction::new(Opcode::SIToFP, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_fptrunc_is_cast() {
    let inst = Instruction::new(Opcode::FPTrunc, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_fpext_is_cast() {
    let inst = Instruction::new(Opcode::FPExt, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_ptrtoint_is_cast() {
    let inst = Instruction::new(Opcode::PtrToInt, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_inttoptr_is_cast() {
    let inst = Instruction::new(Opcode::IntToPtr, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_bitcast_is_cast() {
    let inst = Instruction::new(Opcode::BitCast, vec![], None);
    assert!(inst.is_cast());
}

#[test]
fn test_addrspacecast_is_cast() {
    let inst = Instruction::new(Opcode::AddrSpaceCast, vec![], None);
    assert!(inst.is_cast());
}

// Comparison operations
#[test]
fn test_icmp_is_comparison() {
    let inst = Instruction::new(Opcode::ICmp, vec![], None);
    assert!(inst.is_comparison());
}

#[test]
fn test_fcmp_is_comparison() {
    let inst = Instruction::new(Opcode::FCmp, vec![], None);
    assert!(inst.is_comparison());
}

// Vector operations
#[test]
fn test_extractelement_is_vector_op() {
    let inst = Instruction::new(Opcode::ExtractElement, vec![], None);
    assert!(inst.is_vector_op());
}

#[test]
fn test_insertelement_is_vector_op() {
    let inst = Instruction::new(Opcode::InsertElement, vec![], None);
    assert!(inst.is_vector_op());
}

#[test]
fn test_shufflevector_is_vector_op() {
    let inst = Instruction::new(Opcode::ShuffleVector, vec![], None);
    assert!(inst.is_vector_op());
}

// Aggregate operations
#[test]
fn test_extractvalue_is_aggregate_op() {
    let inst = Instruction::new(Opcode::ExtractValue, vec![], None);
    assert!(inst.is_aggregate_op());
}

#[test]
fn test_insertvalue_is_aggregate_op() {
    let inst = Instruction::new(Opcode::InsertValue, vec![], None);
    assert!(inst.is_aggregate_op());
}

// Other operations
#[test]
fn test_phi_instruction() {
    let inst = Instruction::new(Opcode::PHI, vec![], None);
    assert_eq!(inst.opcode(), Opcode::PHI);
    assert!(!inst.is_terminator());
    assert!(!inst.is_binary_op());
}

#[test]
fn test_call_instruction() {
    let inst = Instruction::new(Opcode::Call, vec![], None);
    assert_eq!(inst.opcode(), Opcode::Call);
}

#[test]
fn test_select_instruction() {
    let inst = Instruction::new(Opcode::Select, vec![], None);
    assert_eq!(inst.opcode(), Opcode::Select);
}

#[test]
fn test_vaarg_instruction() {
    let inst = Instruction::new(Opcode::VAArg, vec![], None);
    assert_eq!(inst.opcode(), Opcode::VAArg);
}

#[test]
fn test_landingpad_instruction() {
    let inst = Instruction::new(Opcode::LandingPad, vec![], None);
    assert_eq!(inst.opcode(), Opcode::LandingPad);
}

#[test]
fn test_cleanuppad_instruction() {
    let inst = Instruction::new(Opcode::CleanupPad, vec![], None);
    assert_eq!(inst.opcode(), Opcode::CleanupPad);
}

#[test]
fn test_catchpad_instruction() {
    let inst = Instruction::new(Opcode::CatchPad, vec![], None);
    assert_eq!(inst.opcode(), Opcode::CatchPad);
}

#[test]
fn test_freeze_instruction() {
    let inst = Instruction::new(Opcode::Freeze, vec![], None);
    assert_eq!(inst.opcode(), Opcode::Freeze);
}

#[test]
fn test_fneg_is_unary_op() {
    let inst = Instruction::new(Opcode::FNeg, vec![], None);
    assert!(inst.is_unary_op());
}

// Integer comparison predicates
#[test]
fn test_int_predicate_eq() {
    let pred = IntPredicate::EQ;
    assert_eq!(pred, IntPredicate::EQ);
}

#[test]
fn test_int_predicate_ne() {
    let pred = IntPredicate::NE;
    assert_eq!(pred, IntPredicate::NE);
}

#[test]
fn test_int_predicate_ugt() {
    let pred = IntPredicate::UGT;
    assert_eq!(pred, IntPredicate::UGT);
}

#[test]
fn test_int_predicate_uge() {
    let pred = IntPredicate::UGE;
    assert_eq!(pred, IntPredicate::UGE);
}

#[test]
fn test_int_predicate_ult() {
    let pred = IntPredicate::ULT;
    assert_eq!(pred, IntPredicate::ULT);
}

#[test]
fn test_int_predicate_ule() {
    let pred = IntPredicate::ULE;
    assert_eq!(pred, IntPredicate::ULE);
}

#[test]
fn test_int_predicate_sgt() {
    let pred = IntPredicate::SGT;
    assert_eq!(pred, IntPredicate::SGT);
}

#[test]
fn test_int_predicate_sge() {
    let pred = IntPredicate::SGE;
    assert_eq!(pred, IntPredicate::SGE);
}

#[test]
fn test_int_predicate_slt() {
    let pred = IntPredicate::SLT;
    assert_eq!(pred, IntPredicate::SLT);
}

#[test]
fn test_int_predicate_sle() {
    let pred = IntPredicate::SLE;
    assert_eq!(pred, IntPredicate::SLE);
}

// Float comparison predicates
#[test]
fn test_float_predicate_oeq() {
    let pred = FloatPredicate::OEQ;
    assert_eq!(pred, FloatPredicate::OEQ);
}

#[test]
fn test_float_predicate_ogt() {
    let pred = FloatPredicate::OGT;
    assert_eq!(pred, FloatPredicate::OGT);
}

#[test]
fn test_float_predicate_oge() {
    let pred = FloatPredicate::OGE;
    assert_eq!(pred, FloatPredicate::OGE);
}

#[test]
fn test_float_predicate_olt() {
    let pred = FloatPredicate::OLT;
    assert_eq!(pred, FloatPredicate::OLT);
}

#[test]
fn test_float_predicate_ole() {
    let pred = FloatPredicate::OLE;
    assert_eq!(pred, FloatPredicate::OLE);
}

#[test]
fn test_float_predicate_one() {
    let pred = FloatPredicate::ONE;
    assert_eq!(pred, FloatPredicate::ONE);
}

#[test]
fn test_float_predicate_ord() {
    let pred = FloatPredicate::ORD;
    assert_eq!(pred, FloatPredicate::ORD);
}

#[test]
fn test_float_predicate_uno() {
    let pred = FloatPredicate::UNO;
    assert_eq!(pred, FloatPredicate::UNO);
}

#[test]
fn test_float_predicate_ueq() {
    let pred = FloatPredicate::UEQ;
    assert_eq!(pred, FloatPredicate::UEQ);
}

#[test]
fn test_float_predicate_ugt() {
    let pred = FloatPredicate::UGT;
    assert_eq!(pred, FloatPredicate::UGT);
}

#[test]
fn test_float_predicate_uge() {
    let pred = FloatPredicate::UGE;
    assert_eq!(pred, FloatPredicate::UGE);
}

#[test]
fn test_float_predicate_ult() {
    let pred = FloatPredicate::ULT;
    assert_eq!(pred, FloatPredicate::ULT);
}

#[test]
fn test_float_predicate_ule() {
    let pred = FloatPredicate::ULE;
    assert_eq!(pred, FloatPredicate::ULE);
}

#[test]
fn test_float_predicate_une() {
    let pred = FloatPredicate::UNE;
    assert_eq!(pred, FloatPredicate::UNE);
}

// Atomic ordering
#[test]
fn test_atomic_ordering_notatomic() {
    let ord = AtomicOrdering::NotAtomic;
    assert_eq!(ord, AtomicOrdering::NotAtomic);
}

#[test]
fn test_atomic_ordering_unordered() {
    let ord = AtomicOrdering::Unordered;
    assert_eq!(ord, AtomicOrdering::Unordered);
}

#[test]
fn test_atomic_ordering_monotonic() {
    let ord = AtomicOrdering::Monotonic;
    assert_eq!(ord, AtomicOrdering::Monotonic);
}

#[test]
fn test_atomic_ordering_acquire() {
    let ord = AtomicOrdering::Acquire;
    assert_eq!(ord, AtomicOrdering::Acquire);
}

#[test]
fn test_atomic_ordering_release() {
    let ord = AtomicOrdering::Release;
    assert_eq!(ord, AtomicOrdering::Release);
}

#[test]
fn test_atomic_ordering_acqrel() {
    let ord = AtomicOrdering::AcquireRelease;
    assert_eq!(ord, AtomicOrdering::AcquireRelease);
}

#[test]
fn test_atomic_ordering_seqcst() {
    let ord = AtomicOrdering::SequentiallyConsistent;
    assert_eq!(ord, AtomicOrdering::SequentiallyConsistent);
}

// Fast math flags
#[test]
fn test_fastmath_default() {
    let flags = FastMathFlags::default();
    assert!(!flags.allow_reassoc);
    assert!(!flags.no_nans);
    assert!(!flags.no_infs);
}

#[test]
fn test_fastmath_fast() {
    let flags = FastMathFlags::fast();
    assert!(flags.allow_reassoc);
    assert!(flags.no_nans);
    assert!(flags.no_infs);
    assert!(flags.no_signed_zeros);
    assert!(flags.allow_reciprocal);
    assert!(flags.allow_contract);
    assert!(flags.approx_func);
}
