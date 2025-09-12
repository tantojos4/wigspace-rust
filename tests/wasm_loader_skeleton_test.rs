//! Integration test for WASM loader skeleton
use wigspace_rust::modules::dynamic_loader::{WasmModule, DynamicModule};
use std::path::PathBuf;

#[test]
fn test_wasm_loader_skeleton() {
    let mut wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    wasm_path.push("src/modules/wasm_plugin_example/hello.wat");
    // For real use, compile .wat to .wasm and use .wasm file
    let module = WasmModule::load(&wasm_path).expect("Failed to load WASM module");
    let input = "hello wasm";
    let output = module.handle(input);
    assert!(output.contains("[WASM skeleton] would call WASM with input: hello wasm"), "Unexpected WASM skeleton output: {}", output);
    println!("WASM skeleton output: {}", output);
}
