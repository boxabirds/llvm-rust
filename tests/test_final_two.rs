use llvm_rust::{Context, parse};

#[test]
fn test_store_to_null_minimal() {
    let content = r#"
define void @_Z3foov() {
entry:
  tail call void null(), !dbg !4
  ret void
}

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!3}

!0 = distinct !DICompileUnit(language: DW_LANG_C_plus_plus_14, file: !1, producer: "Clang", isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug, enums: !2, globals: !2, splitDebugInlining: false, debugInfoForProfiling: true, nameTableKind: None)
!1 = !DIFile(filename: "test_sym_mod.cpp", directory: "/workspace/asaravan/test_bugpoint")
!2 = !{}
!3 = !{i32 2, !"Debug Info Version", i32 3}
!4 = !DILocation(line: 83, column: 3, scope: !5)
!5 = distinct !DISubprogram(name: "foo", linkageName: "_Z3foov", scope: !1, file: !1, line: 77, type: !6, scopeLine: 78, flags: DIFlagPrototyped | DIFlagAllCallsDescribed, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !0, retainedNodes: !2)
!6 = !DISubroutineType(types: !7)
!7 = !{null}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ StoreToNull minimal passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}

#[test]
fn test_abs_intrinsic_minimal() {
    let content = r#"
define i8 @test(i8 %a, i8 %b) {
  %abs1 = call i8 @llvm.abs.i8(i8 %a, i1 true)
  %mul = mul i8 %abs1, %b
  %abs2 = call i8 @llvm.abs.i8(i8 %mul, i1 true)
  ret i8 %abs2
}

declare i8 @llvm.abs.i8(i8, i1)
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ abs intrinsic minimal passed"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
