use std::ffi::{CStr, CString};
use std::os::raw::{c_char};

pub fn set_error(err: &str, error_buffer: *mut *mut c_char) {
    let err_str = match CString::new(err) {
        Ok(s) => s,
        Err(_) => return
    };
    write_buffer(error_buffer, err_str);
}

fn write_buffer(error_buffer: *mut *mut c_char, err_str: CString) {
    unsafe {
        if !error_buffer.is_null() {
            let c_str = err_str.into_raw();
            *error_buffer = c_str;
        }
    }
}

pub fn assert_string<'a>(str_ptr: *const c_char, error_buffer: *mut *mut c_char) -> Option<&'a str> {
    let result = unsafe {
        if str_ptr.is_null() {
            set_error(POINTER_IS_NULL, error_buffer);
            return None
        }
        CStr::from_ptr(str_ptr)
    }.to_str();
    match result {
        Ok(str) => Some(str),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return None
        }
    }
}

pub fn assert_pointer<'a, T>(ptr: *mut T, error_buffer: *mut *mut c_char) -> Option<&'a mut T> {
    let result = unsafe {
        if ptr.is_null() {
            set_error(POINTER_IS_NULL, error_buffer);
            return None
        }
        &mut *ptr
    };
    Some(result)
}

const POINTER_IS_NULL: &'static str = "Pointer is null";