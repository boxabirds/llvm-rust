use llvm_rust::{Context, parse};

fn main() {
    let tests = vec![
        ("declare", "declare ptr @f(ptr, ...)"),
        ("define", "define ptr @f_thunk(ptr %this, ...) { ret ptr null }"),
        ("call_type", "define void @test() { %x = call ptr (ptr, ...) @f(ptr null, ...) ret void }"),
    ];

    for (name, code) in tests {
        let ctx = Context::new();
        match parse(code, ctx) {
            Ok(_) => println!("✓ {}: Parsed successfully", name),
            Err(e) => println!("✗ {}: {:?}", name, e),
        }
    }
}
