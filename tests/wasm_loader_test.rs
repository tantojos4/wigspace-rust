//! Integration test for WASM loader stub
use wigspace_rust::modules::dynamic_loader::{DynamicModule, WasmModule};

#[test]
fn test_wasm_loader_stub() {
    let module = WasmModule;
    let input = "hello wasm";
    let output = module.handle(input);
    assert!(
        output.contains("[WASM stub] input: hello wasm"),
        "Unexpected WASM stub output: {}",
        output
    );
    println!("WASM stub output: {}", output);
}
