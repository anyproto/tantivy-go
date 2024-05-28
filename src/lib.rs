use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use log::debug;
use tantivy::{Index, schema::*};
use tantivy::directory::MmapDirectory;
use tantivy::doc as TantivyDocument;

pub struct SearchResult {
    documents: Vec<TantivyDocument>,
    index: usize,
}

fn set_error(err: &str, error_buffer: *mut *mut c_char) -> c_int {
    let err_str = match CString::new(err) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    unsafe {
        if !error_buffer.is_null() {
            let c_str = err_str.into_raw();
            *error_buffer = c_str;
        }
    }
    -1
}

fn set_error_owned(err: String, error_buffer: *mut *mut c_char) -> c_int {
    let err_str = match CString::new(err) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    unsafe {
        if !error_buffer.is_null() {
            let c_str = err_str.into_raw();
            *error_buffer = c_str;
        }
    }
    -1
}

#[no_mangle]
pub extern "C" fn schema_builder_new(error_buffer: *mut *mut c_char) -> *mut SchemaBuilder {
    Box::into_raw(Box::new(Schema::builder()))
}

#[no_mangle]
pub extern "C" fn schema_builder_add_text_field(builder: *mut SchemaBuilder, name: *const c_char, stored: bool, error_buffer: *mut *mut c_char) -> c_int {
    let builder = unsafe {
        if builder.is_null() {
            return set_error("SchemaBuilder is null", error_buffer);
        }
        &mut *builder
    };

    let name_str = unsafe { CStr::from_ptr(name) }.to_str();
    match name_str {
        Ok(name) => {
            if stored {
                builder.add_text_field(name, TEXT | STORED);
            } else {
                builder.add_text_field(name, TEXT);
            }
            0
        }
        Err(err) => set_error(&err.to_string(), error_buffer)
    }
}

#[no_mangle]
pub extern "C" fn create_index_with_schema_builder(path: *const c_char, builder: *mut SchemaBuilder, error_buffer: *mut *mut c_char) -> *mut Index {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let builder = unsafe {
        if builder.is_null() {
            set_error("SchemaBuilder is null", error_buffer);
            return ptr::null_mut();
        }
        Box::from_raw(builder)
    };

    let schema = builder.build();
    let dir = match MmapDirectory::open(path_str) {
        Ok(dir) => dir,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    match Index::open_or_create(dir, schema) {
        Ok(index) => Box::into_raw(Box::new(index)),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn create_document() -> *mut TantivyDocument {
    Box::into_raw(Box::new(TantivyDocument::new()))
}

#[no_mangle]
pub extern "C" fn add_field(doc_ptr: *mut TantivyDocument, field_name: *const c_char, field_value: *const c_char, index_ptr: *mut Index, error_buffer: *mut *mut c_char) -> c_int {
    let doc = unsafe {
        if doc_ptr.is_null() {
            return set_error("Document is null", error_buffer);
        }
        &mut *doc_ptr
    };

    let index = unsafe {
        if index_ptr.is_null() {
            return set_error("Index is null", error_buffer);
        }
        &mut *index_ptr
    };

    let field_name_str = unsafe { CStr::from_ptr(field_name) }.to_str();
    let field_value_str = unsafe { CStr::from_ptr(field_value) }.to_str();
    debug!("field_name_str: {:?}, value {:?}", field_name_str, field_value_str);
    let field_name = match field_name_str {
        Ok(s) => s,
        Err(err) => {
            return set_error(&err.to_string(), error_buffer);
        }
    };

    let field_value = match field_value_str {
        Ok(s) => s,
        Err(err) => {
            return set_error(&err.to_string(), error_buffer);
        }
    };

    let schema = index.schema();
    let field = match schema.get_field(field_name) {
        Ok(field) => field,
        Err(err) => return set_error(&err.to_string(), error_buffer),
    };

    doc.add_text(field, field_value);
    0
}

#[no_mangle]
pub extern "C" fn add_document(index_ptr: *mut Index, doc_ptr: *mut TantivyDocument, error_buffer: *mut *mut c_char) -> c_int {
    let index = unsafe {
        if index_ptr.is_null() {
            return set_error("Index is null", error_buffer);
        }
        &mut *index_ptr
    };

    let doc = unsafe {
        if doc_ptr.is_null() {
            return set_error("Document is null", error_buffer);
        }
        Box::from_raw(doc_ptr)
    };

    let mut index_writer = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(err) => {
            return set_error(&err.to_string(), error_buffer);
        }
    };

    index_writer.add_document(*doc);
    if index_writer.commit().is_ok() {
        0
    } else {
        set_error("Failed to commit document", error_buffer)
    }
}

#[no_mangle]
pub extern "C" fn search_index(index_ptr: *mut Index, query: *const c_char, error_buffer: *mut *mut c_char) -> *mut SearchResult {
    let index = unsafe {
        if index_ptr.is_null() {
            set_error("Index is null", error_buffer);
            return ptr::null_mut();
        }
        &mut *index_ptr
    };

    let query_str = unsafe { CStr::from_ptr(query) }.to_str();
    let query = match query_str {
        Ok(s) => s,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let reader = match index.reader() {
        Ok(reader) => reader,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let searcher = reader.searcher();
    let schema = index.schema();
    let title_field = match schema.get_field("title") {
        Ok(field) => field,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };
    let body_field = match schema.get_field("body") {
        Ok(field) => field,
        Err(err) => return {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        },
    };

    let query_parser = tantivy::query::QueryParser::for_index(index, vec![title_field, body_field]);
    let query = match query_parser.parse_query(query) {
        Ok(query) => query,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let top_docs = match searcher.search(&query, &tantivy::collector::TopDocs::with_limit(10)) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let mut documents = Vec::new();
    for (_score, doc_address) in top_docs {
        match searcher.doc(doc_address) {
            Ok(doc) => documents.push(doc),
            Err(err) => {
                set_error(&err.to_string(), error_buffer);
                return ptr::null_mut();
            }
        };
    }

    Box::into_raw(Box::new(SearchResult {
        documents,
        index: 0,
    }))
}

#[no_mangle]
pub extern "C" fn get_next_result(result_ptr: *mut SearchResult, error_buffer: *mut *mut c_char) -> *mut TantivyDocument {
    let result = unsafe {
        if result_ptr.is_null() {
            set_error("SearchResult is null", error_buffer);
            return ptr::null_mut();
        }
        &mut *result_ptr
    };

    if result.index >= result.documents.len() {
        return ptr::null_mut();
    }

    let doc = &result.documents[result.index];
    result.index += 1;
    Box::into_raw(Box::new(doc.clone()))
}

#[no_mangle]
pub extern "C" fn get_document_json(doc_ptr: *mut TantivyDocument, error_buffer: *mut *mut c_char) -> *mut c_char {
    let doc = unsafe {
        if doc_ptr.is_null() {
            set_error("Document is null", error_buffer);
            return ptr::null_mut();
        }
        &*doc_ptr
    };

    let json_doc = match serde_json::to_string(doc) {
        Ok(json) => json,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    match CString::new(json_doc) {
        Ok(cstr) => cstr.into_raw(),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn free_search_result(result_ptr: *mut SearchResult) {
    if !result_ptr.is_null() {
        unsafe {
            Box::from_raw(result_ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_index(index_ptr: *mut Index) {
    if !index_ptr.is_null() {
        unsafe {
            Box::from_raw(index_ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_schema_builder(builder_ptr: *mut SchemaBuilder) {
    if !builder_ptr.is_null() {
        unsafe {
            Box::from_raw(builder_ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_document(doc_ptr: *mut TantivyDocument) {
    if !doc_ptr.is_null() {
        unsafe {
            Box::from_raw(doc_ptr);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() -> u8 {
    let mut log_level: &str = "info";
    let parse_val: String;
    if let Ok(existing_value) = std::env::var("ELV_RUST_LOG") {
        parse_val = existing_value;
        log_level = &parse_val;
    }
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .try_init();
    0
}