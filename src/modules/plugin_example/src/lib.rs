//! Minimal C ABI plugin for dynamic_loader.rs
//! Exports: handle_request(input_ptr, input_len) -> *mut c_char

use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

// Note: If you expand the FFI interface, you may need to reintroduce CStr or c_void.

#[no_mangle]
pub unsafe extern "C" fn handle_request(
    input_ptr: *const c_uchar,
    input_len: usize,
) -> *mut c_char {
    if input_ptr.is_null() || input_len == 0 {
        let s = CString::new("[plugin] empty input").unwrap();
        return s.into_raw();
    }
    let slice = std::slice::from_raw_parts(input_ptr, input_len);
    let input = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => "[plugin] invalid utf8",
    };
    let response = format!("[plugin] got: {}", input);
    CString::new(response).unwrap().into_raw()
}

/// Optional: free string returned by handle_request
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
