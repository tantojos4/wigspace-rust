//! Dynamic module loader for multi-language plugins (C ABI, Rust dylib, WASM, scripting)
//! - C ABI: Loads `.so` modules via FFI (libloading)
//! - Rust dylib: Loads Rust plugins as `cdylib`/`dylib`
//! - WASM: Loads WASM modules via wasmtime/wasmer
//! - Scripting: Loads Lua, JS, Python, etc. via embedded engines

use libloading::{Library, Symbol};
use std::ffi::OsStr;
use std::ffi::c_void;

/// Trait for all dynamic modules (C ABI, Rust dylib, WASM, scripting)
pub trait DynamicModule: Send + Sync {
    fn handle(&self, input: &str) -> String;
}

/// C ABI module loader (legacy, ecosystem-wide)
pub struct CAbiModule {
    _lib: Library,
    handler: Symbol<'static, unsafe extern "C" fn(*const u8, usize) -> *mut c_void>,
}

impl CAbiModule {
    pub unsafe fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, libloading::Error> {
        let lib = unsafe { Library::new(path)? };
        let handler: Symbol<unsafe extern "C" fn(*const u8, usize) -> *mut c_void> =
            unsafe { lib.get(b"handle_request")? };
        // Extend lifetime for trait object safety
        let handler: Symbol<'static, unsafe extern "C" fn(*const u8, usize) -> *mut c_void> =
            unsafe { std::mem::transmute(handler) };
        Ok(CAbiModule { _lib: lib, handler })
    }
}

impl DynamicModule for CAbiModule {
    fn handle(&self, input: &str) -> String {
        let bytes = input.as_bytes();
        unsafe {
            let ptr = (self.handler)(bytes.as_ptr(), bytes.len());
            // Assume returned pointer is a null-terminated C string
            let cstr = std::ffi::CStr::from_ptr(ptr as *const i8);
            let result = cstr.to_string_lossy().into_owned();
            // Free the string if the module provides a free function (not shown here)
            result
        }
    }
}

/// Stub for Rust dylib loader
pub struct RustDylibModule;
impl DynamicModule for RustDylibModule {
    fn handle(&self, input: &str) -> String {
        format!("[Rust dylib stub] input: {}", input)
    }
}

/// Stub for WASM module loader
pub struct WasmModule;
impl DynamicModule for WasmModule {
    fn handle(&self, input: &str) -> String {
        format!("[WASM stub] input: {}", input)
    }
}

/// Stub for scripting module loader
pub struct ScriptingModule;
impl DynamicModule for ScriptingModule {
    fn handle(&self, input: &str) -> String {
        format!("[Scripting stub] input: {}", input)
    }
}
