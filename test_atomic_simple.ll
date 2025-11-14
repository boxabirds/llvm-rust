define void @test(ptr %x) {
  store atomic i32 3, ptr %x release, align 4
  ret void
}
