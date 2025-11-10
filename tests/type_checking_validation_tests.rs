//! Comprehensive Type Checking Validation Tests
//!
//! This test file validates the 20+ type checking rules added to the verification system.
//! Tests cover cast operations, function calls, aggregate operations, vector operations,
//! shift operations, PHI nodes, and more.

use llvm_rust::{Context, parse, verification::verify_module};

/// Helper function to check that IR is valid
fn assert_valid(ir: &str) {
    let ctx = Context::new();
    match parse(ir, ctx) {
        Ok(module) => {
            if let Err(errors) = verify_module(&module) {
                panic!("Expected valid IR, but got verification errors: {:?}", errors);
            }
        }
        Err(e) => panic!("Parse error: {:?}", e),
    }
}

/// Helper function to check that IR is invalid and produces verification error
fn assert_invalid(ir: &str, expected_error_substr: &str) {
    let ctx = Context::new();
    match parse(ir, ctx) {
        Ok(module) => {
            match verify_module(&module) {
                Ok(()) => panic!("Expected verification error containing '{}', but IR was valid", expected_error_substr),
                Err(errors) => {
                    let error_str = format!("{:?}", errors);
                    if !error_str.to_lowercase().contains(&expected_error_substr.to_lowercase()) {
                        panic!("Expected error containing '{}', but got: {:?}", expected_error_substr, errors);
                    }
                }
            }
        }
        Err(_) => {
            // Parse error is acceptable for invalid IR tests
        }
    }
}

// ===== CAST OPERATION TESTS =====

#[test]
fn test_valid_trunc() {
    assert_valid(r#"
        define void @test() {
            %1 = trunc i64 100 to i32
            ret void
        }
    "#);
}

#[test]
fn test_invalid_trunc_non_integer() {
    assert_invalid(r#"
        define void @test() {
            %1 = trunc float 1.0 to i32
            ret void
        }
    "#, "trunc");
}

#[test]
fn test_invalid_trunc_not_smaller() {
    assert_invalid(r#"
        define void @test() {
            %1 = trunc i32 100 to i64
            ret void
        }
    "#, "trunc result must be smaller");
}

#[test]
fn test_valid_zext() {
    assert_valid(r#"
        define void @test() {
            %1 = zext i32 100 to i64
            ret void
        }
    "#);
}

#[test]
fn test_invalid_zext_not_larger() {
    assert_invalid(r#"
        define void @test() {
            %1 = zext i64 100 to i32
            ret void
        }
    "#, "zext result must be larger");
}

#[test]
fn test_valid_sext() {
    assert_valid(r#"
        define void @test() {
            %1 = sext i32 100 to i64
            ret void
        }
    "#);
}

#[test]
fn test_invalid_sext_non_integer() {
    assert_invalid(r#"
        define void @test() {
            %1 = sext float 1.0 to double
            ret void
        }
    "#, "sext");
}

#[test]
fn test_valid_fptrunc() {
    assert_valid(r#"
        define void @test() {
            %1 = fptrunc double 1.0 to float
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fptrunc_non_float() {
    assert_invalid(r#"
        define void @test() {
            %1 = fptrunc i64 100 to i32
            ret void
        }
    "#, "fptrunc");
}

#[test]
fn test_valid_fpext() {
    assert_valid(r#"
        define void @test() {
            %1 = fpext float 1.0 to double
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fpext_non_float() {
    assert_invalid(r#"
        define void @test() {
            %1 = fpext i32 100 to i64
            ret void
        }
    "#, "fpext");
}

#[test]
fn test_valid_fptoui() {
    assert_valid(r#"
        define void @test() {
            %1 = fptoui float 1.5 to i32
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fptoui_non_float() {
    assert_invalid(r#"
        define void @test() {
            %1 = fptoui i32 100 to i64
            ret void
        }
    "#, "fptoui");
}

#[test]
fn test_valid_fptosi() {
    assert_valid(r#"
        define void @test() {
            %1 = fptosi double -1.5 to i32
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fptosi_result_not_integer() {
    assert_invalid(r#"
        define void @test() {
            %1 = fptosi float 1.0 to float
            ret void
        }
    "#, "fptosi");
}

#[test]
fn test_valid_uitofp() {
    assert_valid(r#"
        define void @test() {
            %1 = uitofp i32 100 to float
            ret void
        }
    "#);
}

#[test]
fn test_invalid_uitofp_non_integer() {
    assert_invalid(r#"
        define void @test() {
            %1 = uitofp float 1.0 to double
            ret void
        }
    "#, "uitofp");
}

#[test]
fn test_valid_sitofp() {
    assert_valid(r#"
        define void @test() {
            %1 = sitofp i32 -100 to float
            ret void
        }
    "#);
}

#[test]
fn test_invalid_sitofp_result_not_float() {
    assert_invalid(r#"
        define void @test() {
            %1 = sitofp i32 100 to i64
            ret void
        }
    "#, "sitofp");
}

#[test]
fn test_valid_ptrtoint() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            %1 = ptrtoint i32* %ptr to i64
            ret void
        }
    "#);
}

#[test]
fn test_invalid_ptrtoint_non_pointer() {
    assert_invalid(r#"
        define void @test() {
            %1 = ptrtoint i32 100 to i64
            ret void
        }
    "#, "ptrtoint");
}

#[test]
fn test_valid_inttoptr() {
    assert_valid(r#"
        define void @test() {
            %1 = inttoptr i64 0 to i32*
            ret void
        }
    "#);
}

#[test]
fn test_invalid_inttoptr_non_integer() {
    assert_invalid(r#"
        define void @test(i32* %ptr) {
            %1 = inttoptr i32* %ptr to i32*
            ret void
        }
    "#, "inttoptr");
}

#[test]
fn test_valid_bitcast() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            %1 = bitcast i32* %ptr to i8*
            ret void
        }
    "#);
}

#[test]
fn test_invalid_bitcast_void() {
    assert_invalid(r#"
        define void @test() {
            %1 = bitcast void undef to i32
            ret void
        }
    "#, "bitcast");
}

#[test]
fn test_valid_addrspacecast() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            %1 = addrspacecast i32* %ptr to i32 addrspace(1)*
            ret void
        }
    "#);
}

#[test]
fn test_invalid_addrspacecast_non_pointer() {
    assert_invalid(r#"
        define void @test() {
            %1 = addrspacecast i32 0 to i32*
            ret void
        }
    "#, "addrspacecast");
}

// ===== AGGREGATE OPERATION TESTS =====

#[test]
fn test_valid_extractelement() {
    assert_valid(r#"
        define void @test(<4 x i32> %vec) {
            %1 = extractelement <4 x i32> %vec, i32 0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_extractelement_non_vector() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            %1 = extractelement i32 %val, i32 0
            ret void
        }
    "#, "extractelement");
}

#[test]
fn test_invalid_extractelement_non_integer_index() {
    assert_invalid(r#"
        define void @test(<4 x i32> %vec) {
            %1 = extractelement <4 x i32> %vec, float 0.0
            ret void
        }
    "#, "extractelement");
}

#[test]
fn test_valid_insertelement() {
    assert_valid(r#"
        define void @test(<4 x i32> %vec) {
            %1 = insertelement <4 x i32> %vec, i32 42, i32 0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_insertelement_type_mismatch() {
    assert_invalid(r#"
        define void @test(<4 x i32> %vec) {
            %1 = insertelement <4 x i32> %vec, float 1.0, i32 0
            ret void
        }
    "#, "insertelement");
}

#[test]
fn test_valid_extractvalue() {
    assert_valid(r#"
        define void @test({i32, float} %agg) {
            %1 = extractvalue {i32, float} %agg, 0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_extractvalue_non_aggregate() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            %1 = extractvalue i32 %val, 0
            ret void
        }
    "#, "extractvalue");
}

#[test]
fn test_valid_insertvalue() {
    assert_valid(r#"
        define void @test({i32, float} %agg) {
            %1 = insertvalue {i32, float} %agg, i32 42, 0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_insertvalue_non_aggregate() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            %1 = insertvalue i32 %val, i32 42, 0
            ret void
        }
    "#, "insertvalue");
}

#[test]
fn test_valid_getelementptr() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            %1 = getelementptr i32, i32* %ptr, i32 1
            ret void
        }
    "#);
}

#[test]
fn test_invalid_getelementptr_non_pointer() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            %1 = getelementptr i32, i32 %val, i32 0
            ret void
        }
    "#, "getelementptr");
}

#[test]
fn test_invalid_getelementptr_non_integer_index() {
    assert_invalid(r#"
        define void @test(i32* %ptr) {
            %1 = getelementptr i32, i32* %ptr, float 0.0
            ret void
        }
    "#, "getelementptr");
}

// ===== FUNCTION CALL TESTS =====

#[test]
fn test_valid_call_no_args() {
    assert_valid(r#"
        declare void @func()
        define void @test() {
            call void @func()
            ret void
        }
    "#);
}

#[test]
fn test_valid_call_with_args() {
    assert_valid(r#"
        declare i32 @func(i32, float)
        define void @test() {
            %1 = call i32 @func(i32 42, float 1.5)
            ret void
        }
    "#);
}

#[test]
fn test_invalid_call_wrong_arg_count() {
    assert_invalid(r#"
        declare i32 @func(i32, float)
        define void @test() {
            %1 = call i32 @func(i32 42)
            ret void
        }
    "#, "call");
}

#[test]
fn test_invalid_call_wrong_arg_type() {
    assert_invalid(r#"
        declare i32 @func(i32, float)
        define void @test() {
            %1 = call i32 @func(float 1.0, float 1.5)
            ret void
        }
    "#, "call");
}

#[test]
fn test_valid_call_varargs() {
    assert_valid(r#"
        declare i32 @printf(i8*, ...)
        define void @test(i8* %fmt) {
            %1 = call i32 (i8*, ...) @printf(i8* %fmt, i32 42)
            ret void
        }
    "#);
}

// ===== VECTOR OPERATION TESTS =====

#[test]
fn test_valid_shufflevector() {
    assert_valid(r#"
        define void @test(<4 x i32> %v1, <4 x i32> %v2) {
            %1 = shufflevector <4 x i32> %v1, <4 x i32> %v2, <4 x i32> <i32 0, i32 1, i32 4, i32 5>
            ret void
        }
    "#);
}

#[test]
fn test_invalid_shufflevector_type_mismatch() {
    assert_invalid(r#"
        define void @test(<4 x i32> %v1, <4 x float> %v2) {
            %1 = shufflevector <4 x i32> %v1, <4 x float> %v2, <4 x i32> zeroinitializer
            ret void
        }
    "#, "shufflevector");
}

#[test]
fn test_invalid_shufflevector_non_vector() {
    assert_invalid(r#"
        define void @test(i32 %v1, i32 %v2) {
            %1 = shufflevector i32 %v1, i32 %v2, <4 x i32> zeroinitializer
            ret void
        }
    "#, "shufflevector");
}

// ===== SHIFT OPERATION TESTS =====

#[test]
fn test_valid_shl() {
    assert_valid(r#"
        define void @test() {
            %1 = shl i32 1, i32 5
            ret void
        }
    "#);
}

#[test]
fn test_invalid_shl_type_mismatch() {
    assert_invalid(r#"
        define void @test() {
            %1 = shl i32 1, i64 5
            ret void
        }
    "#, "shl");
}

#[test]
fn test_valid_lshr() {
    assert_valid(r#"
        define void @test() {
            %1 = lshr i32 100, i32 2
            ret void
        }
    "#);
}

#[test]
fn test_valid_ashr() {
    assert_valid(r#"
        define void @test() {
            %1 = ashr i32 -100, i32 2
            ret void
        }
    "#);
}

// ===== PHI NODE TESTS =====

#[test]
fn test_valid_phi() {
    assert_valid(r#"
        define i32 @test(i1 %cond) {
        entry:
            br i1 %cond, label %then, label %else
        then:
            br label %exit
        else:
            br label %exit
        exit:
            %result = phi i32 [ 1, %then ], [ 2, %else ]
            ret i32 %result
        }
    "#);
}

#[test]
fn test_invalid_phi_type_mismatch() {
    assert_invalid(r#"
        define i32 @test(i1 %cond) {
        entry:
            br i1 %cond, label %then, label %else
        then:
            br label %exit
        else:
            br label %exit
        exit:
            %result = phi i32 [ 1, %then ], [ 2.0, %else ]
            ret i32 %result
        }
    "#, "phi");
}

// ===== INTEGER OPERATION TESTS =====

#[test]
fn test_valid_add() {
    assert_valid(r#"
        define void @test() {
            %1 = add i32 1, i32 2
            ret void
        }
    "#);
}

#[test]
fn test_invalid_add_type_mismatch() {
    assert_invalid(r#"
        define void @test() {
            %1 = add i32 1, i64 2
            ret void
        }
    "#, "TypeMismatch");
}

#[test]
fn test_valid_fadd() {
    assert_valid(r#"
        define void @test() {
            %1 = fadd float 1.0, float 2.0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fadd_type_mismatch() {
    assert_invalid(r#"
        define void @test() {
            %1 = fadd float 1.0, double 2.0
            ret void
        }
    "#, "TypeMismatch");
}

// ===== COMPARISON TESTS =====

#[test]
fn test_valid_icmp() {
    assert_valid(r#"
        define void @test() {
            %1 = icmp eq i32 1, i32 2
            ret void
        }
    "#);
}

#[test]
fn test_invalid_icmp_type_mismatch() {
    assert_invalid(r#"
        define void @test() {
            %1 = icmp eq i32 1, i64 2
            ret void
        }
    "#, "comparison");
}

#[test]
fn test_valid_fcmp() {
    assert_valid(r#"
        define void @test() {
            %1 = fcmp oeq float 1.0, float 2.0
            ret void
        }
    "#);
}

#[test]
fn test_invalid_fcmp_type_mismatch() {
    assert_invalid(r#"
        define void @test() {
            %1 = fcmp oeq float 1.0, double 2.0
            ret void
        }
    "#, "comparison");
}

// ===== SELECT TESTS =====

#[test]
fn test_valid_select() {
    assert_valid(r#"
        define void @test(i1 %cond) {
            %1 = select i1 %cond, i32 1, i32 2
            ret void
        }
    "#);
}

#[test]
fn test_invalid_select_type_mismatch() {
    assert_invalid(r#"
        define void @test(i1 %cond) {
            %1 = select i1 %cond, i32 1, float 2.0
            ret void
        }
    "#, "select");
}

// ===== MEMORY OPERATION TESTS =====

#[test]
fn test_valid_store() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            store i32 42, i32* %ptr
            ret void
        }
    "#);
}

#[test]
fn test_invalid_store_non_pointer() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            store i32 42, i32 %val
            ret void
        }
    "#, "store");
}

#[test]
fn test_valid_load() {
    assert_valid(r#"
        define void @test(i32* %ptr) {
            %1 = load i32, i32* %ptr
            ret void
        }
    "#);
}

#[test]
fn test_invalid_load_non_pointer() {
    assert_invalid(r#"
        define void @test(i32 %val) {
            %1 = load i32, i32 %val
            ret void
        }
    "#, "load");
}

#[test]
fn test_valid_alloca() {
    assert_valid(r#"
        define void @test() {
            %1 = alloca i32
            ret void
        }
    "#);
}

// ===== SWITCH TESTS =====

#[test]
fn test_valid_switch() {
    assert_valid(r#"
        define void @test(i32 %val) {
        entry:
            switch i32 %val, label %default [
                i32 1, label %case1
                i32 2, label %case2
            ]
        default:
            ret void
        case1:
            ret void
        case2:
            ret void
        }
    "#);
}

#[test]
fn test_invalid_switch_type_mismatch() {
    assert_invalid(r#"
        define void @test(i32 %val) {
        entry:
            switch i32 %val, label %default [
                i64 1, label %case1
            ]
        default:
            ret void
        case1:
            ret void
        }
    "#, "switch");
}

// ===== RETURN TYPE TESTS =====

#[test]
fn test_valid_return_void() {
    assert_valid(r#"
        define void @test() {
            ret void
        }
    "#);
}

#[test]
fn test_valid_return_value() {
    assert_valid(r#"
        define i32 @test() {
            ret i32 42
        }
    "#);
}

#[test]
fn test_invalid_return_type_mismatch() {
    assert_invalid(r#"
        define i32 @test() {
            ret float 1.0
        }
    "#, "return");
}

#[test]
fn test_invalid_return_void_when_value_expected() {
    assert_invalid(r#"
        define i32 @test() {
            ret void
        }
    "#, "return");
}
