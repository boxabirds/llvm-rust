define void @test() {
  add i32 1, 2, !foo !1
  ret void
}

!1 = distinct !{}
