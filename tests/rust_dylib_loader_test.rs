//! Integration test for Rust dylib loader stub
use wigspace_rust::modules::dynamic_loader::{DynamicModule, RustDylibModule};

#[test]
fn test_rust_dylib_loader_stub() {
    let module = RustDylibModule;
    let input = "hello dylib";
    let output = module.handle(input);
    assert!(
        output.contains("[Rust dylib stub] input: hello dylib"),
        "Unexpected dylib stub output: {}",
        output
    );
    println!("Rust dylib stub output: {}", output);
}
