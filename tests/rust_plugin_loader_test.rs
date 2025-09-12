//! Integration test for Rust dylib plugin loader
use std::path::PathBuf;
use wigspace_rust::modules::dynamic_loader::{DynamicModule, RustDylibModule};

#[test]
fn test_load_rust_plugin_example_dylib() {
    // Path to the built .so file
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("src/modules/rust_plugin_example/target/release/librust_plugin_example.so");
    assert!(
        so_path.exists(),
        "Rust plugin dylib not found: {}",
        so_path.display()
    );

    // SAFETY: We trust the plugin to follow the vtable contract
    let module =
        unsafe { RustDylibModule::load(&so_path) }.expect("Failed to load rust plugin dylib");
    let input = "hello from rust dylib test";
    let output = module.handle(input);
    assert!(
        output.contains("[rust_plugin_example] got: hello from rust dylib test"),
        "Unexpected plugin output: {}",
        output
    );
    println!("Rust dylib plugin output: {}", output);
}
