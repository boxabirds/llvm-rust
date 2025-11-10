; RUN: llvm-as < %s | llvm-dis | llvm-as | llvm-dis | FileCheck --check-prefix=CHECK --check-prefix=CHECK-UNMAT %s

define void @test() !dbg !1 {
  add i32 2, 1, !bar !0
  add i32 1, 2, !foo !1
  ret void
}

!0 = !DILocation(line: 662302, column: 26, scope: !1)
!1 = distinct !DISubprogram(name: "foo", isDefinition: true, unit: !5)
!5 = distinct !DICompileUnit(language: DW_LANG_C99, producer: "clang",
                             file: !6,
                             isOptimized: true, flags: "-O2",
                             splitDebugFilename: "abc.debug", emissionKind: 2)
!6 = !DIFile(filename: "path/to/file", directory: "/path/to/dir")
