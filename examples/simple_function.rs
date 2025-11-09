//! Simple Function Example
//!
//! This example demonstrates how to use llvm-rust to construct a simple
//! function that adds two integers and returns the result.
//!
//! The generated LLVM IR represents:
//! ```llvm
//! define i32 @add(i32 %a, i32 %b) {
//! entry:
//!   %sum = add i32 %a, %b
//!   ret i32 %sum
//! }
//! ```

use llvm_rust::{Context, Module, Function, BasicBlock, Builder, Value};

fn main() {
    // Create a context - this owns all LLVM types and constants
    let ctx = Context::new();

    // Create a module - this is the top-level container for functions
    let module = Module::new("example_module".to_string(), ctx.clone());

    // Create the function signature: i32 add(i32, i32)
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(
        i32_type.clone(),           // return type
        vec![i32_type.clone(), i32_type.clone()],  // parameter types
        false                        // not vararg
    );

    // Create the function
    let function = Function::new("add".to_string(), fn_type);

    // Create function arguments
    let arg_a = Value::argument(i32_type.clone(), 0, Some("a".to_string()));
    let arg_b = Value::argument(i32_type.clone(), 1, Some("b".to_string()));
    function.set_arguments(vec![arg_a.clone(), arg_b.clone()]);

    // Create a basic block for the function body
    let entry_bb = BasicBlock::new(Some("entry".to_string()));
    function.add_basic_block(entry_bb.clone());

    // Create a builder to construct instructions
    let mut builder = Builder::new(ctx.clone());
    builder.position_at_end(entry_bb);

    // Build the add instruction: %sum = add i32 %a, %b
    let sum = builder.build_add(arg_a, arg_b, Some("sum".to_string()));

    // Build the return instruction: ret i32 %sum
    builder.build_ret(sum);

    // Add the function to the module
    module.add_function(function);

    // Print the module
    println!("{}", module);
    println!("\nSuccessfully constructed LLVM IR for a simple add function!");
}
