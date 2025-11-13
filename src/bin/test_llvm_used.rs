use llvm_rust::{Context, parse};

fn main() {
    let ctx = Context::new();
    let source = r#"
@a = global i32 42
@llvm.used = appending global [1 x ptr] [ptr @a], section "llvm.metadata"
"#;

    // Try parsing without verification first
    match parse(source, ctx) {
        Ok(module) => {
            println!("Parse succeeded");
            for g in module.globals() {
                println!("Global: {}", g.name);
                if let Some(init) = &g.initializer {
                    println!("  Has initializer");
                    println!("  Is zero: {}", init.is_zero());
                    println!("  Is null: {}", init.is_null());
                    if let Some(elems) = init.array_elements() {
                        println!("  Array elements: {}", elems.len());
                        for (i, elem) in elems.iter().enumerate() {
                            println!("    [{}]: is_null={}", i, elem.is_null());
                        }
                    }
                } else {
                    println!("  No initializer");
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
}
