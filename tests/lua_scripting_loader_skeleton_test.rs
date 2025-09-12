//! Integration test for Lua scripting loader skeleton
use wigspace_rust::modules::dynamic_loader::{ScriptingModule, DynamicModule};
use std::path::PathBuf;

#[test]
fn test_lua_scripting_loader_skeleton() {
    let mut lua_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lua_path.push("src/modules/lua_plugin_example/hello.lua");
    let module = ScriptingModule::load(&lua_path).expect("Failed to load Lua script");
    let input = "hello lua";
    let output = module.handle(input);
    assert!(output.contains("[lua_plugin] got: hello lua"), "Unexpected Lua plugin output: {}", output);
    println!("Lua skeleton output: {}", output);
}
