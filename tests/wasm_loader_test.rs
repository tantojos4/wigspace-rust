//! Integration test for WASM loader stub
use std::path::PathBuf;
use wigspace_rust::modules::dynamic_loader::{DynamicModule, WasmModule};

#[test]
fn test_wasm_loader_skeleton() {
    let mut wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    wasm_path.push("src/modules/wasm_plugin_example/hello.wat");
    // For real use, compile .wat to .wasm and use .wasm file
    let module = WasmModule::load(&wasm_path).expect("Failed to load WASM module");
    let input = "hello wasm";
    let output = module.handle(input);
    assert!(
        output.contains("[wasm_plugin] hello wasm"),
        "Unexpected WASM plugin output: {}",
        output
    );
    println!("WASM plugin output: {}", output);
}
