declare i32 @llvm.nvvm.atomic.load.add.f32.p0(ptr, float)
declare i32 @llvm.nvvm.atomic.load.add.f64.p0(ptr, double)

define i32 @atomics(ptr %p0, float %b, double %c) {
  %r3 = call i32 @llvm.nvvm.atomic.load.add.f32.p0(ptr %p0, float %b)
  %r4 = call i32 @llvm.nvvm.atomic.load.add.f64.p0(ptr %p0, double %c)
  ret i32 %r3
}
