use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::slice;
use serde_json::json;
use tantivy::schema::{Field, Schema};
use crate::box_from;
use crate::tantivy_util::{convert_document_to_json, Document};

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

pub fn process_type_slice<'a, T, F>(
    ptr: *mut *mut T,
    error_buffer: *mut *mut c_char,
    len: usize,
    mut func: F,
) -> Result<(), ()>
    where
        F: FnMut(*mut T) -> Result<(), ()> {
    let slice = match assert_pointer(ptr, error_buffer) {
        Some(ptr) => unsafe { slice::from_raw_parts(ptr, len) },
        None => return Err(()),
    };

    for item in slice {
        let value = match assert_pointer(*item, error_buffer) {
            Some(value) => value,
            None => return Err(()),
        };
        if func(value).is_err() {
            return Err(());
        }
    }

    Ok(())
}

pub fn process_string_slice<'a, F>(
    ptr: *mut *const c_char,
    error_buffer: *mut *mut c_char,
    len: usize,
    mut func: F,
) -> Result<(), ()>
    where
        F: FnMut(&'a str) -> Result<(), ()> {
    let slice = match assert_pointer(ptr, error_buffer) {
        Some(ptr) => unsafe { slice::from_raw_parts(ptr, len) },
        None => return Err(()),
    };

    for &item in slice {
        let value = match assert_string(item, error_buffer) {
            Some(value) => value,
            None => return Err(()),
        };

        if func(value).is_err() {
            return Err(());
        }
    }

    Ok(())
}

pub fn schema_apply_for_field<'a, T, K, F: FnMut(Field, &'a str) -> Result<T, ()>>(
    error_buffer: *mut *mut c_char,
    schema: Schema,
    field_name: &'a str,
    mut func: F,
) -> Result<T, ()>
{
    match schema.get_field(field_name) {
        Ok(field) => func(field, field_name),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            Err(())
        }
    }
}

pub fn convert_document_as_json(
    include_fields_ptr: *mut *const c_char,
    include_fields_len: usize,
    error_buffer: *mut *mut c_char,
    doc: &&mut Document,
    schema: Schema,
) -> Result<String, ()> {
    let mut field_to_name = HashMap::new();

    if process_string_slice(include_fields_ptr, error_buffer, include_fields_len, |field_name| {
        schema_apply_for_field::<(), (), _>(error_buffer, schema.clone(), field_name, |field, field_name| {
            field_to_name.insert(field, field_name);
            Ok(())
        })
    }).is_err() {
        return Err(());
    }

    let doc_json = convert_document_to_json(&doc, field_to_name);

    Ok(json!(doc_json).to_string())
}

pub fn start_lib_init(log_level: &str) {
    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level)
    ).try_init();
}

const POINTER_IS_NULL: &'static str = "Pointer is null";