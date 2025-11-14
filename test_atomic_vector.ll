define void @f(ptr %x) {
  store atomic <1 x i32> <i32 3>, ptr %x release, align 4
  ret void
}
