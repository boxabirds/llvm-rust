use llvm_rust::{Context, parse};

fn main() {
    let ctx = Context::new();
    let source = "@GV = global [10 x x86_amx] zeroinitializer";

    match parse(source, ctx) {
        Ok(module) => {
            println!("Parse succeeded - checking module:");
            let globals = module.globals();
            println!("Globals count: {}", globals.len());
            for g in globals {
                println!("Global: {} type: {:?}", g.name, g.ty);
                if let Some((elem_ty, size)) = g.ty.array_info() {
                    println!("  Array element type: {:?}, size: {}", elem_ty, size);
                    println!("  Is x86_amx: {}", elem_ty.is_x86_amx());
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
}
