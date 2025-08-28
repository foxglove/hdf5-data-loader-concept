use std::ffi::*;

#[unsafe(no_mangle)]
pub extern "C" fn dlopen(_filename: *const c_char, _flag: c_int) -> *mut c_void {
    // This function should fail, so we return a null pointer.
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn dlsym(_handle: *mut c_void, _symbol: *const c_char) -> *mut c_void {
    // Fails, returns a null pointer.
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn dlclose(_handle: *mut c_void) -> c_int {
    // Fails, returns a non-zero error code.
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn dlerror() -> *mut c_char {
    std::ptr::null_mut()
}
