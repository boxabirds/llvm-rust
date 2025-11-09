use llvm_rust::{Context, parse};

#[test]
fn test_two_functions() {
    let content = r##"; RUN: not opt -S %s -passes=verify 2>&1 | FileCheck %s

; CHECK: Number of label constraints does not match number of callbr dests
; CHECK-NEXT: #too_few_label_constraints
define void @too_few_label_constraints() {
  callbr void asm sideeffect "#too_few_label_constraints", "!i"()
  to label %1 [label %2, label %3]
1:
  ret void
2:
  ret void
3:
  ret void
}

; CHECK-NOT: Number of label constraints does not match number of callbr dests
define void @correct_label_constraints() {
  callbr void asm sideeffect "${0:l} ${1:l}", "!i,!i"()
  to label %1 [label %2, label %3]
1:
  ret void
2:
  ret void
3:
  ret void
}
"##;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Parsed two functions"),
        Err(e) => panic!("Failed: {:?}", e),
    }
}
