//! Minimal Rust dylib plugin for DynamicModule FFI
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct PluginVTable {
    pub handle: extern "C" fn(*const c_char) -> *mut c_char,
}

extern "C" fn handle(_input: *const c_char) -> *mut c_char {
    let response = "[rust_plugin] hello rust";
    CString::new(response).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn get_plugin_vtable() -> *const PluginVTable {
    &PLUGIN_VTABLE
}

static PLUGIN_VTABLE: PluginVTable = PluginVTable {
    handle,
};

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
