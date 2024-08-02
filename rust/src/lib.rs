use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use logcall::logcall;
use tantivy::{Index, schema::*};

use crate::c_util::{add_and_consume_documents, add_field, assert_pointer, assert_string, box_from, convert_document_as_json, create_index_with_schema, delete_docs, drop_any, get_doc, search, set_error, start_lib_init};
use crate::tantivy_util::{add_text_field, Document, DOCUMENT_BUDGET_BYTES, register_edge_ngram_tokenizer, register_ngram_tokenizer, register_raw_tokenizer, register_simple_tokenizer, SearchResult};

mod tantivy_util;
mod c_util;

#[logcall]
#[no_mangle]
pub extern "C" fn schema_builder_new() -> *mut SchemaBuilder {
    Box::into_raw(Box::new(Schema::builder()))
}

#[logcall]
#[no_mangle]
pub extern "C" fn schema_builder_add_text_field(
    builder_ptr: *mut SchemaBuilder,
    field_name_ptr: *const c_char,
    stored: bool,
    is_text: bool,
    is_fast: bool,
    index_record_option_const: usize,
    tokenizer_name_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let builder = match assert_pointer(builder_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let tokenizer_name = match assert_string(tokenizer_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let field_name = match assert_string(field_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let index_record_option = match index_record_option_const {
        0 => IndexRecordOption::Basic,
        1 => IndexRecordOption::WithFreqs,
        2 => IndexRecordOption::WithFreqsAndPositions,
        _ => return set_error("index_record_option_const is wrong", error_buffer)
    };

    add_text_field(stored, is_text, is_fast, builder, tokenizer_name, field_name, index_record_option);
}

#[logcall]
#[no_mangle]
pub extern "C" fn schema_builder_build(
    builder_ptr: *mut SchemaBuilder,
    error_buffer: *mut *mut c_char,
) -> *mut Schema {
    let builder = match assert_pointer(builder_ptr, error_buffer) {
        Some(value) => box_from(value),
        None => return ptr::null_mut()
    };

    Box::into_raw(Box::new(builder.build()))
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_create_with_schema(
    path_ptr: *const c_char,
    schema_ptr: *mut Schema,
    error_buffer: *mut *mut c_char,
) -> *mut Index {
    let schema = match assert_pointer(schema_ptr, error_buffer) {
        Some(value) => value.clone(),
        None => return ptr::null_mut(),
    };

    let path = match assert_string(path_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut(),
    };

    match create_index_with_schema(error_buffer, schema, path) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut()
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_register_text_analyzer_ngram(
    index_ptr: *mut Index,
    tokenizer_name_ptr: *const c_char,
    min_gram: usize,
    max_gram: usize,
    prefix_only: bool,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let tokenizer_name = match assert_string(tokenizer_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    match register_ngram_tokenizer(min_gram, max_gram, prefix_only, index, tokenizer_name) {
        Err(err) => return set_error(&err.to_string(), error_buffer),
        _ => return
    };
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_register_text_analyzer_edge_ngram(
    index_ptr: *mut Index,
    tokenizer_name_ptr: *const c_char,
    min_gram: usize,
    max_gram: usize,
    limit: usize,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let tokenizer_name = match assert_string(tokenizer_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    register_edge_ngram_tokenizer(min_gram, max_gram, limit, index, tokenizer_name);
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_register_text_analyzer_simple(
    index_ptr: *mut Index,
    tokenizer_name_ptr: *const c_char,
    text_limit: usize,
    lang_str_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let tokenizer_name = match assert_string(tokenizer_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let lang = match assert_string(lang_str_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    register_simple_tokenizer(text_limit, index, tokenizer_name, lang);
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_register_text_analyzer_raw(
    index_ptr: *mut Index,
    tokenizer_name_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let tokenizer_name = match assert_string(tokenizer_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    register_raw_tokenizer(index, tokenizer_name);
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_add_and_consume_documents(
    index_ptr: *mut Index,
    docs_ptr: *mut *mut Document,
    docs_len: usize,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let index_writer = match index.writer(DOCUMENT_BUDGET_BYTES) {
        Ok(writer) => writer,
        Err(err) => return set_error(&err.to_string(), error_buffer)
    };

    add_and_consume_documents(docs_ptr, docs_len, error_buffer, index_writer);
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_delete_documents(
    index_ptr: *mut Index,
    field_name_ptr: *const c_char,
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    error_buffer: *mut *mut c_char,
) {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let field_name = match assert_string(field_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    delete_docs(delete_ids_ptr, delete_ids_len, error_buffer, index, field_name);
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_num_docs(
    index_ptr: *mut Index,
    error_buffer: *mut *mut c_char,
) -> u64 {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return 0,
    };

    match index.reader() {
        Ok(reader) => reader.searcher().num_docs(),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return 0;
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn index_search(
    index_ptr: *mut Index,
    field_names_ptr: *mut *const c_char,
    field_names_len: usize,
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
    with_highlights: bool,
) -> *mut SearchResult {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut()
    };

    match search(field_names_ptr, field_names_len, query_ptr, error_buffer, docs_limit, index, with_highlights) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut()
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub extern "C" fn index_free(index_ptr: *mut Index) {
    drop_any(index_ptr)
}

#[logcall]
#[no_mangle]
pub extern "C" fn search_result_get_size(
    result_ptr: *mut SearchResult,
    error_buffer: *mut *mut c_char,
) -> usize {
    match assert_pointer(result_ptr, error_buffer) {
        Some(value) => value.size,
        None => 0
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn search_result_get_doc(
    result_ptr: *mut SearchResult,
    index: usize,
    error_buffer: *mut *mut c_char,
) -> *mut Document {
    let result = match assert_pointer(result_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut()
    };

    match get_doc(index, error_buffer, result) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut(),
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub extern "C" fn search_result_free(result_ptr: *mut SearchResult) {
    drop_any(result_ptr)
}

#[logcall]
#[no_mangle]
pub extern "C" fn document_create() -> *mut Document {
    Box::into_raw(Box::new(Document {
        tantivy_doc: TantivyDocument::new(),
        highlights: vec![],
        score: 0.0,
    }))
}

#[logcall]
#[no_mangle]
pub extern "C" fn document_add_field(
    doc_ptr: *mut Document,
    field_name_ptr: *const c_char,
    field_value_ptr: *const c_char,
    index_ptr: *mut Index,
    error_buffer: *mut *mut c_char,
) {
    let doc = match assert_pointer(doc_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let field_name = match assert_string(field_name_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    let field_value = match assert_string(field_value_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };

    add_field(error_buffer, doc, index, field_name, field_value);
}

#[logcall]
#[no_mangle]
pub extern "C" fn document_as_json(
    doc_ptr: *mut Document,
    include_fields_ptr: *mut *const c_char,
    include_fields_len: usize,
    schema_ptr: *mut Schema,
    error_buffer: *mut *mut c_char,
) -> *mut c_char {
    let doc = match assert_pointer(doc_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut()
    };

    let schema = match assert_pointer(schema_ptr, error_buffer) {
        Some(value) => value.clone(),
        None => return ptr::null_mut()
    };

    let json = match convert_document_as_json(include_fields_ptr, include_fields_len, error_buffer, &doc, schema) {
        Ok(value) => value,
        Err(_) => return ptr::null_mut()
    };

    match CString::new(json) {
        Ok(cstr) => cstr.into_raw(),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub extern "C" fn document_free(doc_ptr: *mut Document) {
    drop_any(doc_ptr)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub unsafe extern "C" fn init_lib(
    log_level_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let log_level = match assert_string(log_level_ptr, error_buffer) {
        Some(value) => value,
        None => return
    };
    start_lib_init(log_level);
}