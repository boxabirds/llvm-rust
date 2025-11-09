define void @test(ptr %x) {
  load atomic i32, ptr %x unordered, align 4
  ret void
}
