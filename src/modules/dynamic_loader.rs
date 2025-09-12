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

/// Real Rust dylib loader (vtable pattern)
use std::os::raw::c_char;

#[repr(C)]
pub struct PluginVTable {
    pub handle: extern "C" fn(*const c_char) -> *mut c_char,
}

pub struct RustDylibModule {
    _lib: Library,
    vtable: &'static PluginVTable,
}

impl RustDylibModule {
    pub unsafe fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, libloading::Error> {
        let lib = unsafe { Library::new(path)? };
        let vtable_sym: Symbol<unsafe extern "C" fn() -> *const PluginVTable> =
            unsafe { lib.get(b"get_plugin_vtable")? };
        let vtable = unsafe { vtable_sym() };
        let vtable: &'static PluginVTable = unsafe { std::mem::transmute(vtable) };
        Ok(RustDylibModule { _lib: lib, vtable })
    }
}

impl DynamicModule for RustDylibModule {
    fn handle(&self, input: &str) -> String {
        let c_input = std::ffi::CString::new(input).unwrap();
        unsafe {
            let ptr = (self.vtable.handle)(c_input.as_ptr());
            let cstr = std::ffi::CStr::from_ptr(ptr);
            let result = cstr.to_string_lossy().into_owned();
            // Free the string if the module provides a free function (optional)
            result
        }
    }
}

/// WASM module loader (wasmtime skeleton)
pub struct WasmModule {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    linker: wasmtime::Linker<()>,
    memory_ty: wasmtime::MemoryType,
}

impl WasmModule {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::from_file(&engine, path)?;
        let linker = wasmtime::Linker::new(&engine);
        let memory_ty = wasmtime::MemoryType::new(1, None);
        Ok(WasmModule {
            engine,
            module,
            linker,
            memory_ty,
        })
    }
}

impl DynamicModule for WasmModule {
    fn handle(&self, input: &str) -> String {
        use wasmtime::{Store, Val, Memory};
        let mut store = Store::new(&self.engine, ());
        let memory = Memory::new(&mut store, self.memory_ty.clone()).unwrap();
        let mut linker = self.linker.clone();
        linker.define(&mut store, "env", "memory", memory.clone()).unwrap();
        let instance = match linker.instantiate(&mut store, &self.module) {
            Ok(i) => i,
            Err(e) => return format!("[WASM error] instantiation failed: {}", e),
        };
        // Find exported function
        let func = match instance.get_func(&mut store, "handle") {
            Some(f) => f,
            None => return "[WASM error] no exported 'handle' function".to_string(),
        };
        // Write input string to memory (at offset 100)
        let input_bytes = input.as_bytes();
        let offset = 100u32;
        if let Err(e) = memory.write(&mut store, offset as usize, input_bytes) {
            return format!("[WASM error] memory write: {}", e);
        }
        // Call handle(ptr, len)
        let mut results = [Val::I32(0)];
        let result = func.call(
            &mut store,
            &[Val::I32(offset as i32), Val::I32(input_bytes.len() as i32)],
            &mut results,
        );
        let out_ptr = match result {
            Ok(()) => match results[0] {
                Val::I32(ptr) => ptr as u32,
                _ => return "[WASM error] unexpected return type".to_string(),
            },
            Err(e) => return format!("[WASM error] call failed: {}", e),
        };
        // Read null-terminated string from memory at out_ptr
        let mut buf = Vec::new();
        let mut cur = out_ptr as usize;
        loop {
            let byte = match memory.data(&store).get(cur) {
                Some(b) => *b,
                None => break,
            };
            if byte == 0 { break; }
            buf.push(byte);
            cur += 1;
        }
        String::from_utf8_lossy(&buf).into_owned()
    }
}

/// Lua scripting module loader (rlua skeleton)
pub struct ScriptingModule {
    script: String,
}

impl ScriptingModule {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let script = std::fs::read_to_string(path)?;
        Ok(ScriptingModule { script })
    }
}

impl DynamicModule for ScriptingModule {
    fn handle(&self, input: &str) -> String {
    use rlua::Lua;
        let lua = Lua::new();
        let script = &self.script;
        if let Err(e) = lua.load(script).exec() {
            return format!("[Lua error] script load: {}", e);
        }
        let func = match lua.globals().get::<_, rlua::Function>("handle") {
            Ok(f) => f,
            Err(e) => return format!("[Lua error] no 'handle' function: {}", e),
        };
        match func.call::<_, rlua::Value>(input) {
            Ok(rlua::Value::String(s)) => s.to_str().unwrap_or("").to_string(),
            Ok(v) => format!("[Lua] Non-string return: {:?}", v),
            Err(e) => format!("[Lua error] call: {}", e),
        }
    }
}
