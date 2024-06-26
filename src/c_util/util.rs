use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::{fs, slice};
use std::path::Path;
use log::debug;
use serde_json::json;
use tantivy::directory::MmapDirectory;
use tantivy::{Index, IndexWriter, Searcher, SnippetGenerator, TantivyDocument, Term};
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{Field, Schema};
use crate::tantivy_util::{convert_document_to_json, Document, DOCUMENT_BUDGET_BYTES, get_string_field_entry, Highlight, SearchResult};

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

pub fn create_index_with_schema(error_buffer: *mut *mut c_char, schema: Schema, path: &str) -> Result<*mut Index, ()> {
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

    Ok(match Index::open_or_create(dir, schema) {
        Ok(index) => Box::into_raw(Box::new(index)),
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    })
}

pub fn add_and_consume_documents(
    docs_ptr: *mut *mut Document,
    docs_len: usize,
    error_buffer: *mut *mut c_char,
    mut index_writer: IndexWriter,
) {
    if process_type_slice(docs_ptr, error_buffer, docs_len, |doc| {
        let doc = *box_from(doc);
        let _ = index_writer.add_document(doc.tantivy_doc);
        Ok(())
    }).is_err() {
        return;
    }

    if index_writer.commit().is_err() {
        set_error("Failed to commit document", error_buffer)
    }
}

pub fn delete_docs(
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    error_buffer: *mut *mut c_char,
    index: &mut Index,
    field_name: &str,
) {
    let mut index_writer: IndexWriter<TantivyDocument> = match index.writer(DOCUMENT_BUDGET_BYTES) {
        Ok(writer) => writer,
        Err(_) => return
    };

    let schema = index.schema();

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
        let _ = index_writer.delete_term(Term::from_field_text(field, id_value));
        Ok(())
    }).is_err() {
        return;
    }

    if index_writer.commit().is_err() {
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
    index: &mut Index,
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
    index: &mut Index,
    with_highlights: bool,
) -> Result<*mut SearchResult, ()> {
    let reader = match index.reader() {
        Ok(reader) => reader,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(());
        }
    };

    let searcher = reader.searcher();
    let schema = index.schema();

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

    let query_parser = QueryParser::for_index(index, fields);

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
                let highlights = find_highlights(error_buffer, with_highlights, &searcher, &query, &doc)?;
                documents.push(Document {
                    tantivy_doc: doc,
                    highlights,
                    score: score as usize,
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

fn find_highlights(
    error_buffer: *mut *mut c_char,
    with_highlights: bool,
    searcher: &Searcher,
    query: &Box<dyn Query>,
    doc: &TantivyDocument,
) -> Result<Vec<Highlight>, ()> {
    let mut highlights: Vec<Highlight> = vec![];
    if with_highlights {
        for field_value in doc.field_values() {
            let snippet_generator = match SnippetGenerator::create(
                &searcher, query, field_value.field) {
                Err(err) => {
                    set_error(&err.to_string(), error_buffer);
                    return Err(());
                }
                Ok(snippet_generator) => snippet_generator
            };
            let snippet = snippet_generator.snippet_from_doc(doc);
            let highlighted: Vec<(usize, usize)> = snippet.highlighted().to_owned().iter().filter_map(|highlight| {
                if highlight.is_empty() { None } else { Some((highlight.start, highlight.end)) }
            }).collect();

            if highlighted.is_empty() {
                continue;
            }
            highlights.push(Highlight {
                fragment: snippet.fragment().to_owned(),
                highlighted,
            });
        }
    }
    Ok(highlights)
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