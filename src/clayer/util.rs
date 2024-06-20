use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

pub fn set_error(err: &str, error_buffer: *mut *mut c_char) -> c_int {
    let err_str = match CString::new(err) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    write_buffer(error_buffer, err_str);
    -1
}

fn write_buffer(error_buffer: *mut *mut c_char, err_str: CString) {
    unsafe {
        if !error_buffer.is_null() {
            let c_str = err_str.into_raw();
            *error_buffer = c_str;
        }
    }
}

pub fn assert_string<'a>(str_ptr: *const c_char, error_buffer: *mut *mut c_char) -> Result<&'a str, c_int> {
    let result = unsafe {
        if str_ptr.is_null() {
            return Err(set_error(POINTER_IS_NULL, error_buffer));
        }
        CStr::from_ptr(str_ptr)
    }.to_str();
    match result {
        Ok(str) => Ok(str),
        Err(err) => Err(set_error(&err.to_string(), error_buffer))
    }
}

pub fn assert_pointer<'a, T>(ptr: *mut T, error_buffer: *mut *mut c_char) -> Result<&'a mut T, c_int> {
    let result = unsafe {
        if ptr.is_null() {
            return Err(set_error(POINTER_IS_NULL, error_buffer));
        }
        &mut *ptr
    };
    Ok(result)
}

const POINTER_IS_NULL: &'static str = "Pointer is null";