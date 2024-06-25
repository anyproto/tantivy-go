mod tantivy_util;
mod c_util;

use std::ffi::{CString};
use std::os::raw::{c_char};
use std::path::Path;
use std::{fs, ptr, slice};
use std::ptr::null_mut;
use log::debug;
use tantivy::{Index, IndexWriter, schema::*};
use tantivy::directory::MmapDirectory;
use tantivy::query::{QueryParser};
use crate::c_util::{assert_pointer, assert_string, convert_document_as_json, process_string_slice, process_type_slice, schema_apply_for_field, set_error, start_lib_init};
use crate::tantivy_util::{Document, SearchResult, DOCUMENT_BUDGET_BYTES, register_edge_ngram_tokenizer, register_simple_tokenizer, register_raw_tokenizer, add_text_field, register_ngram_tokenizer, get_string_field_entry};

#[no_mangle]
pub extern "C" fn schema_builder_new() -> *mut SchemaBuilder {
    Box::into_raw(Box::new(Schema::builder()))
}

#[no_mangle]
pub extern "C" fn schema_builder_add_text_field(
    builder_ptr: *mut SchemaBuilder,
    field_name_ptr: *const c_char,
    stored: bool,
    is_text: bool,
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

    add_text_field(stored, is_text, builder, tokenizer_name, field_name, index_record_option);
}

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

    register_ngram_tokenizer(min_gram, max_gram, prefix_only, index, tokenizer_name);
}

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

#[no_mangle]
pub extern "C" fn index_search(
    index_ptr: *mut Index,
    field_names_ptr: *mut *const c_char,
    field_names_len: usize,
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
) -> *mut SearchResult {
    let index = match assert_pointer(index_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut()
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

    let mut fields = Vec::with_capacity(field_names_len);

    let field_names_slice = match assert_pointer(field_names_ptr, error_buffer) {
        Some(field_names_ptr) => unsafe { slice::from_raw_parts(field_names_ptr, field_names_len) },
        None => return ptr::null_mut()
    };

    //todo

    for field_name in field_names_slice {
        match assert_string(*field_name, error_buffer) {
            Some(field_name) => {
                match schema.get_field(field_name) {
                    Ok(field) => {
                        fields.push(field);
                    }
                    Err(err) => {
                        set_error(&err.to_string(), error_buffer);
                        return null_mut();
                    }
                };
            }
            None => return null_mut()
        }
    };

    let query = match assert_string(query_ptr, error_buffer) {
        Some(value) => value,
        None => return ptr::null_mut()
    };

    let query_parser = QueryParser::for_index(index, fields);

    let query = match query_parser.parse_query(query) {
        Ok(query) => query,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let top_docs = match searcher.search(
        &query,
        &tantivy::collector::TopDocs::with_limit(docs_limit),
    ) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return ptr::null_mut();
        }
    };

    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        match searcher.doc(doc_address) {
            Ok(doc) => {
                documents.push(Document {
                    tantivy_doc: doc,
                    score: score as usize,
                });
            }
            Err(err) => {
                set_error(&err.to_string(), error_buffer);
                return ptr::null_mut();
            }
        };
    }
    let len = documents.len();
    Box::into_raw(Box::new(SearchResult {
        documents: documents,
        size: len,
    }))
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn index_free(index_ptr: *mut Index) {
    drop_any(index_ptr)
}

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
#[no_mangle]
pub extern "C" fn search_result_free(result_ptr: *mut SearchResult) {
    drop_any(result_ptr)
}

#[no_mangle]
pub extern "C" fn document_create() -> *mut Document {
    Box::into_raw(Box::new(Document {
        tantivy_doc: TantivyDocument::new(),
        score: 0,
    }))
}

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
#[no_mangle]
pub extern "C" fn document_free(doc_ptr: *mut Document) {
    drop_any(doc_ptr)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
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

fn drop_any<T>(ptr: *mut T) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}

fn box_from<T>(ptr: *mut T) -> Box<T> {
    unsafe { Box::from_raw(ptr) }
}

fn create_index_with_schema(error_buffer: *mut *mut c_char, schema: Schema, path: &str) -> Result<*mut Index, ()> {
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

fn add_and_consume_documents(
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

fn delete_docs(delete_ids_ptr: *mut *const c_char, delete_ids_len: usize, error_buffer: *mut *mut c_char, index: &mut Index, field_name: &str) {
    let mut index_writer: IndexWriter<TantivyDocument> = match index.writer(DOCUMENT_BUDGET_BYTES) {
        Ok(writer) => writer,
        Err(err) => return
    };

    let schema = index.schema();

    let field = match schema_apply_for_field::<Field, (), _>(error_buffer, schema.clone(), field_name, |field, field_name| {
        match get_string_field_entry(schema.clone(), field) {
            Ok(value) => Ok(value),
            Err(err) => Err(())
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

fn get_doc(index: usize, error_buffer: *mut *mut c_char, result: &mut SearchResult) -> Result<*mut Document, ()> {
    if index > result.documents.len() - 1 {
        set_error("wrong index", error_buffer);
        return Err(());
    }

    let doc = result.documents[index].clone();
    Ok(Box::into_raw(Box::new(doc)))
}

fn add_field(error_buffer: *mut *mut c_char, doc: &mut Document, index: &mut Index, field_name: &str, field_value: &str){
    let schema = index.schema();
    let field = match schema_apply_for_field::<Field, (), _>(error_buffer, schema, field_name, |field,_| {
        Ok(field)
    }) {
        Ok(field) => field,
        Err(_) => return
    };

    doc.tantivy_doc.add_text(field, field_value);
}