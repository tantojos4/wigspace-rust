//! Example: Integrate main logic with plugin_example via dynamic loader
use std::path::PathBuf;
use wigspace_rust::modules::dynamic_loader::{CAbiModule, DynamicModule};

fn main() {
    // Path to the built .so file
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("src/modules/plugin_example/target/release/libplugin_example.so");
    if !so_path.exists() {
        eprintln!("Plugin .so not found: {}", so_path.display());
        std::process::exit(1);
    }

    // SAFETY: We trust the plugin to follow the C ABI contract
    let module = unsafe { CAbiModule::load(&so_path) }.expect("Failed to load plugin .so");
    let input = "hello from main integration";
    let output = module.handle(input);
    println!("[main integration] Plugin output: {}", output);
}
