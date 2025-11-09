define void @test() {
  %alloca_scalar_no_align = alloca i32, addrspace(0)
  ret void
}
