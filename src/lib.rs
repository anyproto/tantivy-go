use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use log::debug;
use tantivy::{Index, schema::*};
use tantivy::directory::MmapDirectory;

#[no_mangle]
pub extern "C" fn schema_builder_new() -> *mut SchemaBuilder {
    Box::into_raw(Box::new(Schema::builder()))
}

#[no_mangle]
pub extern "C" fn schema_builder_add_text_field(builder: *mut SchemaBuilder, name: *const c_char, stored: bool) {
    let builder = unsafe {
        assert!(!builder.is_null());
        &mut *builder
    };

    let name_str = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
    if stored {
        builder.add_text_field(name_str, TEXT | STORED);
    } else {
        builder.add_text_field(name_str, TEXT);
    }
}

#[no_mangle]
pub extern "C" fn create_index_with_schema_builder(path: *const c_char, builder: *mut SchemaBuilder) -> *mut Index {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let builder = unsafe {
        assert!(!builder.is_null());
        Box::from_raw(builder)
    };

    let schema = builder.build();
    let dir = match MmapDirectory::open(path_str) {
        Ok(dir) => dir,
        Err(_) => return ptr::null_mut(),
    };

    match Index::open_or_create(dir, schema) {
        Ok(index) => Box::into_raw(Box::new(index)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn create_index(path: *const c_char) -> *mut Index {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT);
    let schema = schema_builder.build();

    let dir = match MmapDirectory::open(path_str) {
        Ok(dir) => dir,
        Err(_) => return ptr::null_mut(),
    };

    match Index::open_or_create(dir, schema) {
        Ok(index) => Box::into_raw(Box::new(index)),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn add_document(index_ptr: *mut Index, title: *const c_char, body: *const c_char) -> bool {
    let index = unsafe {
        assert!(!index_ptr.is_null());
        &mut *index_ptr
    };

    let title_str = unsafe { CStr::from_ptr(title) }.to_str().unwrap();
    let body_str = unsafe { CStr::from_ptr(body) }.to_str().unwrap();

    let mut index_writer = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(_) => return false,
    };

    let schema = index.schema();
    let mut doc = TantivyDocument::default();
    doc.add_text(schema.get_field("title").unwrap(), title_str);
    doc.add_text(schema.get_field("body").unwrap(), body_str);

    index_writer.add_document(doc);
    index_writer.commit().is_ok()
}

#[no_mangle]
pub extern "C" fn search_index(index_ptr: *mut Index, query: *const c_char) -> *mut c_char {
    let index = unsafe {
        assert!(!index_ptr.is_null());
        &mut *index_ptr
    };

    let query_str = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

    let reader = match index.reader() {
        Ok(reader) => reader,
        Err(_) => return ptr::null_mut(),
    };

    let searcher = reader.searcher();
    let schema = index.schema();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let query_parser = tantivy::query::QueryParser::for_index(index, vec![title, body]);
    let query = match query_parser.parse_query(query_str) {
        Ok(query) => query,
        Err(_) => return ptr::null_mut(),
    };

    let top_docs = match searcher.search(&query, &tantivy::collector::TopDocs::with_limit(10)) {
        Ok(top_docs) => top_docs,
        Err(_) => return ptr::null_mut(),
    };

    let mut result = String::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address).unwrap();
        let json_doc = retrieved_doc.to_json(&schema);
        result.push_str(&format!("{:?}\n", json_doc));
    }

    CString::new(result).unwrap().into_raw()
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