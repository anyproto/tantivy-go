use std::{fs, slice};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use log::debug;
use serde_json::json;
use tantivy::{Index, IndexWriter, TantivyDocument, TantivyError, Term};
use tantivy::directory::MmapDirectory;
use tantivy::query::{QueryParser};
use tantivy::schema::{Field, Schema};

use crate::tantivy_util::{convert_document_to_json, Document, TantivyContext, DOCUMENT_BUDGET_BYTES, find_highlights, get_string_field_entry, SearchResult};

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
            return None;
        }
        CStr::from_ptr(str_ptr)
    }.to_str();
    match result {
        Ok(str) => Some(str),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return None;
        }
    }
}

pub fn assert_pointer<'a, T>(ptr: *mut T, error_buffer: *mut *mut c_char) -> Option<&'a mut T> {
    let result = unsafe {
        if ptr.is_null() {
            set_error(POINTER_IS_NULL, error_buffer);
            return None;
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
    F: FnMut(*mut T) -> Result<(), ()>,
{
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
    F: FnMut(&'a str) -> Result<(), ()>,
{
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
        schema_apply_for_field::<(), (), _>
            (error_buffer, schema.clone(), field_name, |field, field_name| {
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

pub fn create_context_with_schema(error_buffer: *mut *mut c_char, schema: Schema, path: &str) -> Result<*mut TantivyContext, ()> {
    match fs::create_dir_all(Path::new(path)) {
        Err(e) => {
            debug!("Failed to create directories: {}", e);
            set_error(&e.to_string(), error_buffer);
            return Err(());
        }
        _ => {}
    }

    let dir = match MmapDirectory::open(path) {
        Ok(dir) => dir,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    };

    Ok(match create_tantivy_context(dir, schema) {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    })
}

fn create_tantivy_context(dir: MmapDirectory, schema: Schema) -> Result<TantivyContext, TantivyError> {
    let index = Index::open_or_create(dir, schema)?;
    let writer = index.writer(DOCUMENT_BUDGET_BYTES)?;
    let reader = index.reader()?;
    return Ok(TantivyContext::new(
        index,
        writer,
        reader,
    ));
}

pub fn add_and_consume_documents(
    docs_ptr: *mut *mut Document,
    docs_len: usize,
    error_buffer: *mut *mut c_char,
    writer: &mut IndexWriter,
) {
    if process_type_slice(docs_ptr, error_buffer, docs_len, |doc| {
        let doc = *box_from(doc);
        let _ = writer.add_document(doc.tantivy_doc);
        Ok(())
    }).is_err() {
        return;
    }

    if writer.commit().is_err() {
        set_error("Failed to commit document", error_buffer)
    }
}

pub fn delete_docs(
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    error_buffer: *mut *mut c_char,
    context: &mut TantivyContext,
    field_name: &str,
) {
    let schema = context.index.schema();

    let field = match schema_apply_for_field::<Field, (), _>
        (error_buffer, schema.clone(), field_name, |field, _| {
            match get_string_field_entry(schema.clone(), field) {
                Ok(value) => Ok(value),
                Err(_) => Err(())
            }
        }) {
        Ok(value) => value,
        Err(_) => return
    };

    if process_string_slice(delete_ids_ptr, error_buffer, delete_ids_len, |id_value| {
        let _ = context.writer.delete_term(Term::from_field_text(field, id_value));
        Ok(())
    }).is_err() {
        return;
    }

    if context.writer.commit().is_err() {
        set_error("Failed to commit removing", error_buffer)
    }
}

pub fn get_doc<'a>(
    index: usize,
    error_buffer: *mut *mut c_char,
    result: &mut SearchResult,
) -> Result<*mut Document, ()> {
    if index > result.documents.len() - 1 {
        set_error("wrong index", error_buffer);
        return Err(());
    }

    let doc = result.documents[index].clone();
    Ok(Box::into_raw(Box::new(doc)))
}

pub fn add_field(
    error_buffer: *mut *mut c_char,
    doc: &mut Document,
    index: &Index,
    field_name: &str,
    field_value: &str,
) {
    let schema = index.schema();
    let field = match schema_apply_for_field::<Field, (), _>
        (error_buffer, schema, field_name, |field, _| {
            Ok(field)
        }) {
        Ok(field) => field,
        Err(_) => return
    };

    doc.tantivy_doc.add_text(field, field_value);
}

pub fn search(
    field_names_ptr: *mut *const c_char,
    field_names_len: usize,
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
    context: &mut TantivyContext,
    with_highlights: bool,
) -> Result<*mut SearchResult, ()> {
    let searcher = &context.reader().searcher();
    let schema = context.index.schema();

    let mut fields = Vec::with_capacity(field_names_len);

    if process_string_slice(field_names_ptr, error_buffer, field_names_len, |field_name| {
        schema_apply_for_field::<(), (), _>
            (error_buffer, schema.clone(), field_name, |field, _| {
                fields.push(field);
                Ok(())
            })
    }).is_err() {
        return Err(());
    }

    let query = match assert_string(query_ptr, error_buffer) {
        Some(value) => value,
        None => return Err(())
    };

    let query_parser = QueryParser::for_index(&context.index, fields);

    let query = match query_parser.parse_query(query) {
        Ok(query) => query,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    };

    let top_docs = match searcher.search(
        &query,
        &tantivy::collector::TopDocs::with_limit(docs_limit),
    ) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    };

    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        match searcher.doc::<TantivyDocument>(doc_address) {
            Ok(doc) => {
                let highlights = match find_highlights(
                    with_highlights, &searcher, &query, &doc, schema.clone()) {
                    Ok(highlights) => highlights,
                    Err(err) => {
                        set_error(&err.to_string(), error_buffer);
                        return Err(());
                    }
                };
                documents.push(Document {
                    tantivy_doc: doc,
                    highlights,
                    score: score,
                });
            }

            Err(err) => {
                set_error(&err.to_string(), error_buffer);
                return Err(());
            }
        };
    }

    let len = documents.len();
    Ok(Box::into_raw(Box::new(SearchResult {
        documents: documents,
        size: len,
    })))
}

pub fn drop_any<T>(ptr: *mut T) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}

pub fn box_from<T>(ptr: *mut T) -> Box<T> {
    unsafe { Box::from_raw(ptr) }
}

const POINTER_IS_NULL: &'static str = "Pointer is null";