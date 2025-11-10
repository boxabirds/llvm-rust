//! Tests for global variable parsing with full attributes support

use llvm_rust::{Context, parse};
use llvm_rust::module::{Linkage, Visibility, ThreadLocalMode, UnnamedAddr};

#[test]
fn test_global_with_linkage() {
    let ctx = Context::new();
    let ir = "@global = private global i32 42";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].name(), "global");
    assert_eq!(globals[0].linkage, Linkage::Private);
}

#[test]
fn test_global_with_visibility() {
    let ctx = Context::new();
    let ir = "@global = hidden global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].visibility, Visibility::Hidden);
}

#[test]
fn test_global_with_thread_local() {
    let ctx = Context::new();
    let ir = "@global = thread_local global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].thread_local_mode, ThreadLocalMode::GeneralDynamic);
}

#[test]
fn test_global_with_thread_local_mode() {
    let ctx = Context::new();
    let ir = "@global = thread_local(localdynamic) global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].thread_local_mode, ThreadLocalMode::LocalDynamic);
}

#[test]
fn test_global_with_unnamed_addr() {
    let ctx = Context::new();
    let ir = "@global = unnamed_addr global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].unnamed_addr, UnnamedAddr::Global);
}

#[test]
fn test_global_with_alignment() {
    let ctx = Context::new();
    let ir = "@global = global i32 0, align 16";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].alignment, Some(16));
}

#[test]
fn test_global_with_section() {
    let ctx = Context::new();
    let ir = r#"@global = global i32 0, section ".data""#;

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].section, Some(".data".to_string()));
}

#[test]
fn test_global_constant() {
    let ctx = Context::new();
    let ir = "@const = constant i32 42";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert!(globals[0].is_constant);
}

#[test]
fn test_global_with_multiple_attributes() {
    let ctx = Context::new();
    let ir = "@global = private unnamed_addr global i32 42, align 8";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].linkage, Linkage::Private);
    assert_eq!(globals[0].unnamed_addr, UnnamedAddr::Global);
    assert_eq!(globals[0].alignment, Some(8));
}

#[test]
fn test_global_with_zeroinitializer() {
    let ctx = Context::new();
    let ir = "@global = global i32 zeroinitializer";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert!(globals[0].initializer.is_some());
}

#[test]
fn test_global_external() {
    let ctx = Context::new();
    let ir = "@global = external global i32";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].linkage, Linkage::External);
}

#[test]
fn test_global_weak() {
    let ctx = Context::new();
    let ir = "@global = weak global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].linkage, Linkage::Weak);
}

#[test]
fn test_global_with_addrspace() {
    let ctx = Context::new();
    let ir = "@global = addrspace(1) global i32 0";

    let module = parse(ir, ctx).expect("Failed to parse");
    let globals = module.globals();

    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].addrspace, Some(1));
}

#[test]
fn test_complex_global() {
    let ctx = Context::new();
    // Simplified version without some attributes
    let ir = "@complex = internal unnamed_addr global i32 42, align 4";

    let module = parse(ir, ctx);
    if module.is_err() {
        eprintln!("Parse error: {:?}", module.err());
        panic!("Failed to parse complex global");
    }
    let module = module.unwrap();
    let globals = module.globals();

    assert_eq!(globals.len(), 1, "Expected 1 global but got {}", globals.len());
    let global = &globals[0];
    assert_eq!(global.linkage, Linkage::Internal);
    assert_eq!(global.unnamed_addr, UnnamedAddr::Global);
    assert_eq!(global.alignment, Some(4));
}
