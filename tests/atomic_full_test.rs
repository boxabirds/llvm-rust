use llvm_rust::{Context, parse};

#[test]
fn test_atomic_full_sequence() {
    let content = r#"
; Basic smoke test for atomic operations.

define void @f(ptr %x) {
  ; CHECK: load atomic i32, ptr %x unordered, align 4
  load atomic i32, ptr %x unordered, align 4
  ; CHECK: load atomic volatile i32, ptr %x syncscope("singlethread") acquire, align 4
  load atomic volatile i32, ptr %x syncscope("singlethread") acquire, align 4
  ; CHECK: load atomic volatile i32, ptr %x syncscope("agent") acquire, align 4
  load atomic volatile i32, ptr %x syncscope("agent") acquire, align 4
  ; CHECK: store atomic i32 3, ptr %x release, align 4
  store atomic i32 3, ptr %x release, align 4
  ; CHECK: store atomic volatile i32 3, ptr %x syncscope("singlethread") monotonic, align 4
  store atomic volatile i32 3, ptr %x syncscope("singlethread") monotonic, align 4
  ; CHECK: store atomic volatile i32 3, ptr %x syncscope("workgroup") monotonic, align 4
  store atomic volatile i32 3, ptr %x syncscope("workgroup") monotonic, align 4
  ; CHECK: cmpxchg ptr %x, i32 1, i32 0 syncscope("singlethread") monotonic monotonic
  cmpxchg ptr %x, i32 1, i32 0 syncscope("singlethread") monotonic monotonic
  ; CHECK: cmpxchg ptr %x, i32 1, i32 0 syncscope("workitem") monotonic monotonic
  cmpxchg ptr %x, i32 1, i32 0 syncscope("workitem") monotonic monotonic
  ; CHECK: cmpxchg volatile ptr %x, i32 0, i32 1 acq_rel acquire
  cmpxchg volatile ptr %x, i32 0, i32 1 acq_rel acquire
  ; CHECK: cmpxchg ptr %x, i32 42, i32 0 acq_rel monotonic
  cmpxchg ptr %x, i32 42, i32 0 acq_rel monotonic
  ; CHECK: cmpxchg weak ptr %x, i32 13, i32 0 seq_cst monotonic
  cmpxchg weak ptr %x, i32 13, i32 0 seq_cst monotonic
  ret void
}
"#;
    let ctx = Context::new();
    match parse(content, ctx) {
        Ok(_) => println!("✓ Full atomic sequence test passed"),
        Err(e) => panic!("Failed to parse full atomic sequence: {:?}", e),
    }
}
