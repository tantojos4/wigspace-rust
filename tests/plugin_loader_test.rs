//! Integration test for dynamic_loader.rs with a real .so plugin
use std::path::PathBuf;
use wigspace_rust::modules::dynamic_loader::{CAbiModule, DynamicModule};

#[test]
fn test_load_plugin_example_so() {
    // Path to the built .so file
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("src/modules/plugin_example/target/release/libplugin_example.so");
    assert!(
        so_path.exists(),
        "Plugin .so not found: {}",
        so_path.display()
    );

    // SAFETY: We trust the plugin to follow the C ABI contract
    let module = unsafe { CAbiModule::load(&so_path) }.expect("Failed to load plugin .so");
    let input = "hello from test";
    let output = module.handle(input);
    assert!(
        output.contains("[plugin] got: hello from test"),
        "Unexpected plugin output: {}",
        output
    );
    println!("Plugin output: {}", output);
}
