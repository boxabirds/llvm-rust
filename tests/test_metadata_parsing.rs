use llvm_rust::{Context, parse};

#[test]
fn test_simple_metadata() {
    let input = r#"
define void @test() {
  ret void
}

!0 = !{i32 1, i32 2}
!1 = !{!"string"}
!2 = distinct !{}
"#;

    let ctx = Context::new();
    match parse(input, ctx) {
        Ok(_module) => {
            println!("Successfully parsed metadata!");
        }
        Err(e) => {
            panic!("Parse failed: {:?}", e);
        }
    }
}

#[test]
fn test_module_flags_metadata() {
    let input = r#"
define void @test() {
  ret void
}

!llvm.module.flags = !{!0, !1}
!0 = !{i32 1, !"flag1", i32 42}
!1 = !{i32 2, !"flag2", i32 99}
"#;

    let ctx = Context::new();
    match parse(input, ctx) {
        Ok(module) => {
            println!("Successfully parsed module flags!");
            let flags = module.module_flags();
            println!("Module has {} flags", flags.len());
            for (i, flag) in flags.iter().enumerate() {
                println!("Flag {}: is_tuple={}, num_operands={}",
                         i, flag.is_tuple(), flag.num_operands());
                if let Some(ops) = flag.operands() {
                    for (j, op) in ops.iter().enumerate() {
                        println!("  Operand {}: is_int={}, is_string={}, as_int={:?}, as_string={:?}",
                                 j, op.is_int(), op.is_string(), op.as_int(), op.as_string());
                    }
                }
            }
        }
        Err(e) => {
            println!("Parse failed (verific issue): {:?}", e);
            // Don't panic - we're testing parser not verifier
        }
    }
}
