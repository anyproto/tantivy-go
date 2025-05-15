use logcall::logcall;
use std::ffi::{c_uint, CString};
use std::os::raw::{c_char, c_float};
use std::ptr;
use tantivy::schema::*;

use crate::c_util::{
    add_and_consume_documents, add_field, add_fields, assert_pointer, assert_str, assert_string,
    box_from, convert_document_as_json, create_context_with_schema, delete_docs, drop_any, get_doc,
    search, search_json, set_error, start_lib_init,
};
use crate::tantivy_util::{
    add_text_field, register_edge_ngram_tokenizer, register_jieba_tokenizer,
    register_ngram_tokenizer, register_raw_tokenizer, register_simple_tokenizer, Document,
    SearchResult, TantivyContext, TantivyGoError,
};

mod c_util;
mod config;
mod queries;
mod tantivy_util;

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
) -> u32 {
    let result = || -> Result<u32, TantivyGoError> {
        let builder = assert_pointer(builder_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        let field_name = assert_string(field_name_ptr)?;

        let index_record_option = match index_record_option_const {
            0 => IndexRecordOption::Basic,
            1 => IndexRecordOption::WithFreqs,
            2 => IndexRecordOption::WithFreqsAndPositions,
            _ => {
                return Err(TantivyGoError(
                    "Invalid index_record_option_const".to_string(),
                ))
            }
        };

        Ok(add_text_field(
            stored,
            is_text,
            is_fast,
            builder,
            tokenizer_name.as_str(),
            field_name.as_str(),
            index_record_option,
        ))
    };

    match result() {
        Ok(val) => val,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            0
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn schema_builder_build(
    builder_ptr: *mut SchemaBuilder,
    error_buffer: *mut *mut c_char,
) -> *mut Schema {
    let builder = match assert_pointer(builder_ptr) {
        Ok(value) => box_from(value),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(builder.build()))
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_create_with_schema(
    path_ptr: *const c_char,
    schema_ptr: *mut Schema,
    error_buffer: *mut *mut c_char,
) -> *mut TantivyContext {
    let result = || -> Result<*mut TantivyContext, TantivyGoError> {
        let schema = assert_pointer(schema_ptr)?.clone();
        let path = assert_string(path_ptr)?;
        create_context_with_schema(schema, path)
    };

    match result() {
        Ok(context) => context,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_register_text_analyzer_ngram(
    context_ptr: *mut TantivyContext,
    tokenizer_name_ptr: *const c_char,
    min_gram: usize,
    max_gram: usize,
    prefix_only: bool,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        register_ngram_tokenizer(
            min_gram,
            max_gram,
            prefix_only,
            &context.index,
            tokenizer_name.as_str(),
        )?;
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_register_text_analyzer_edge_ngram(
    context_ptr: *mut TantivyContext,
    tokenizer_name_ptr: *const c_char,
    min_gram: usize,
    max_gram: usize,
    limit: usize,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        register_edge_ngram_tokenizer(
            min_gram,
            max_gram,
            limit,
            &context.index,
            tokenizer_name.as_str(),
        );
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_register_text_analyzer_simple(
    context_ptr: *mut TantivyContext,
    tokenizer_name_ptr: *const c_char,
    text_limit: usize,
    lang_str_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        let lang = assert_string(lang_str_ptr)?;
        register_simple_tokenizer(text_limit, &context.index, tokenizer_name.as_str(), &lang)?;
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_register_jieba_tokenizer(
    context_ptr: *mut TantivyContext,
    tokenizer_name_ptr: *const c_char,
    text_limit: usize,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        register_jieba_tokenizer(text_limit, &context.index, tokenizer_name.as_str());
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_register_text_analyzer_raw(
    context_ptr: *mut TantivyContext,
    tokenizer_name_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        let tokenizer_name = assert_string(tokenizer_name_ptr)?;
        register_raw_tokenizer(&context.index, tokenizer_name.as_str());
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_add_and_consume_documents(
    context_ptr: *mut TantivyContext,
    docs_ptr: *mut *mut Document,
    docs_len: usize,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        add_and_consume_documents(docs_ptr, docs_len, &mut context.writer)?;
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_delete_documents(
    context_ptr: *mut TantivyContext,
    field_id: c_uint,
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        delete_docs(delete_ids_ptr, delete_ids_len, context, field_id)?;
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_num_docs(
    context_ptr: *mut TantivyContext,
    error_buffer: *mut *mut c_char,
) -> u64 {
    let result = || -> Result<u64, TantivyGoError> {
        let context = assert_pointer(context_ptr)?;
        Ok(context.reader().searcher().num_docs())
    };

    match result() {
        Ok(num_docs) => num_docs,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            0
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_search(
    context_ptr: *mut TantivyContext,
    field_ids_ptr: *mut c_uint,
    field_weights_ptr: *mut c_float,
    field_ids_len: usize,
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
    with_highlights: bool,
) -> *mut SearchResult {
    let result = || -> Result<*mut SearchResult, TantivyGoError> {
        let context = assert_pointer(context_ptr)?;

        search(
            field_ids_ptr,
            field_weights_ptr,
            field_ids_len,
            query_ptr,
            docs_limit,
            context,
            with_highlights,
        )
    };

    match result() {
        Ok(search_result) => search_result,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_search_json(
    context_ptr: *mut TantivyContext,
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
    with_highlights: bool,
) -> *mut SearchResult {
    let result = || -> Result<*mut SearchResult, TantivyGoError> {
        let context = assert_pointer(context_ptr)?;

        search_json(query_ptr, docs_limit, context, with_highlights)
    };

    match result() {
        Ok(search_result) => search_result,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub extern "C" fn context_free(context_ptr: *mut TantivyContext) {    
    drop_any(context_ptr)
}

#[logcall]
#[no_mangle]
pub extern "C" fn search_result_get_size(
    result_ptr: *mut SearchResult,
    error_buffer: *mut *mut c_char,
) -> usize {
    let result = || -> Result<usize, TantivyGoError> {
        let result = assert_pointer(result_ptr)?;
        Ok(result.size)
    };

    match result() {
        Ok(size) => size,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            0
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn search_result_get_doc(
    result_ptr: *mut SearchResult,
    index: usize,
    error_buffer: *mut *mut c_char,
) -> *mut Document {
    let result = || -> Result<*mut Document, TantivyGoError> {
        let result = assert_pointer(result_ptr)?;
        get_doc(index, result)
    };

    match result() {
        Ok(doc) => doc,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
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
    field_id: c_uint,
    field_value_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let doc = assert_pointer(doc_ptr)?;
        let field_value = assert_str(field_value_ptr)?;

        add_field(doc, field_id, &field_value)
    };

    match result() {
        Ok(_) => {}
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn document_add_fields(
    doc_ptr: *mut Document,
    field_ids_ptr: *mut c_uint,
    field_ids_len: usize,
    field_value_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
) {
    let result = || -> Result<(), TantivyGoError> {
        let doc = assert_pointer(doc_ptr)?;
        let field_ids = assert_pointer(field_ids_ptr)?;
        let field_value = assert_str(field_value_ptr)?;

        add_fields(doc, field_ids, field_ids_len, &field_value)
    };

    match result() {
        Ok(_) => {}
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
        }
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn document_as_json(
    doc_ptr: *mut Document,
    include_field_ids_ptr: *mut c_uint,
    include_field_ids_len: usize,
    schema_ptr: *mut Schema,
    error_buffer: *mut *mut c_char,
) -> *mut c_char {
    let result = || -> Result<String, TantivyGoError> {
        let doc = assert_pointer(doc_ptr)?;
        let schema = assert_pointer(schema_ptr)?.clone();

        convert_document_as_json(include_field_ids_ptr, include_field_ids_len, doc, schema)
    };

    match result() {
        Ok(json) => match CString::new(json) {
            Ok(cstr) => cstr.into_raw(),
            Err(err) => {
                set_error(&err.to_string(), error_buffer);
                ptr::null_mut()
            }
        },
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
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
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[logcall]
#[no_mangle]
pub unsafe extern "C" fn init_lib(
    log_level_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    clear_on_panic: bool,
    utf8_lenient: bool,
) {
    let result = || -> Result<(), TantivyGoError> {
        let log_level = assert_string(log_level_ptr)?;
        start_lib_init(log_level.as_str(), clear_on_panic, utf8_lenient);
        Ok(())
    };

    match result() {
        Ok(_) => (),
        Err(err) => set_error(&err.to_string(), error_buffer),
    }
}

#[logcall]
#[no_mangle]
pub extern "C" fn context_wait_and_free(context_ptr: *mut TantivyContext, error_buffer: *mut *mut c_char) {
    if context_ptr.is_null() {
        return;
    }
    
    let result = || -> Result<(), TantivyGoError> {
        // Get ownership of the context
        let context = unsafe { Box::from_raw(context_ptr) };
        
        // Call wait_merging_threads on the writer
        context.writer.wait_merging_threads().map_err(|err| {
            TantivyGoError::from_err("Failed to wait for merging threads", &err.to_string())
        })?;
        
        // Box drops automatically when this function ends
        Ok(())
    };

    if let Err(err) = result() {
        set_error(&err.to_string(), error_buffer);
    }
    // Don't call drop_any - we've already taken ownership with Box::from_raw
}
