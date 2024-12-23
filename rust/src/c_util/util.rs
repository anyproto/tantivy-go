use crate::config;
use crate::queries::parse_query_from_json;
use crate::tantivy_util::{
    convert_document_to_json, find_highlights, get_string_field_entry, Document, SearchResult,
    TantivyContext, TantivyGoError, DOCUMENT_BUDGET_BYTES,
};
use log::debug;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float};
use std::panic::PanicInfo;
use std::path::Path;
use std::{fs, panic, slice};
use tantivy::directory::MmapDirectory;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{Field, Schema};
use tantivy::{Index, IndexWriter, Opstamp, Score, TantivyDocument, TantivyError, Term};

pub fn set_error(err: &str, error_buffer: *mut *mut c_char) {
    let err_str = match CString::new(err) {
        Ok(s) => s,
        Err(_) => return,
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

// Try not to copy one-time-living strings if possible
pub fn assert_str<'a>(str_ptr: *const c_char) -> Result<Cow<'a, str>, TantivyGoError> {
    unsafe {
        if str_ptr.is_null() {
            return Err(TantivyGoError(POINTER_IS_NULL.to_owned()));
        }
        let is_lenient = match config::CONFIG.read() {
            Ok(config) => config.utf8_lenient,
            Err(err) => {
                return Err(TantivyGoError(err.to_string()));
            }
        };
        let cstr = CStr::from_ptr(str_ptr);
        if is_lenient {
            Ok(cstr.to_string_lossy())
        } else {
            match cstr.to_str() {
                Ok(valid_str) => Ok(Cow::Borrowed(valid_str)),
                Err(err) => Err(TantivyGoError(err.to_string())),
            }
        }
    }
}

// Always copy long-living strings for safety reasons
pub fn assert_string(str_ptr: *const c_char) -> Result<String, TantivyGoError> {
    assert_str(str_ptr).map(|cow| cow.into_owned())
}

pub fn assert_pointer<'a, T>(ptr: *mut T) -> Result<&'a mut T, TantivyGoError> {
    if ptr.is_null() {
        return Err(TantivyGoError(POINTER_IS_NULL.to_owned()));
    }
    unsafe { Ok(&mut *ptr) }
}

pub fn process_type_slice<'a, T, F>(
    ptr: *mut *mut T,
    len: usize,
    mut func: F,
) -> Result<(), TantivyGoError>
where
    F: FnMut(*mut T) -> Result<(), TantivyGoError>,
{
    let slice = unsafe { slice::from_raw_parts(assert_pointer(ptr)?, len) };
    slice.iter().try_for_each(|&item| func(assert_pointer(item)?) )?;
    Ok(())
}

pub fn process_string_slice<'a, F>(
    ptr: *mut *const c_char,
    len: usize,
    mut func: F,
) -> Result<(), TantivyGoError>
where
    F: FnMut(Cow<'a, str>) -> Result<(), TantivyGoError>,
{
    let slice = unsafe { slice::from_raw_parts(assert_pointer(ptr)?, len) };
    slice.iter().try_for_each(|&item| func(assert_str(item)?))?;
    Ok(())
}


pub fn process_slice<'a, F, T>(ptr: *mut T, len: usize, mut func: F) -> Result<(), TantivyGoError>
where
    F: FnMut(usize, T) -> Result<(), TantivyGoError>,
    T: Copy,
{
    let slice = unsafe { slice::from_raw_parts(assert_pointer(ptr)?, len) };
    slice.iter().enumerate().try_for_each(|(i, &item)| func(i, item))?;
    Ok(())
}


pub fn schema_apply_for_field<
    'a, T, K,
    F: FnMut(Field, Cow<'a, str>) -> Result<T, TantivyGoError>,
>(
    schema: Schema,
    field_name: Cow<'a, str>,
    mut func: F,
) -> Result<T, TantivyGoError> {
    schema.get_field(&field_name)
        .map(|field| func(field, field_name))
        .unwrap_or_else(|err| Err(TantivyGoError::from_err("Get field error", &err.to_string())))
}

pub fn convert_document_as_json(
    include_fields_ptr: *mut *const c_char,
    include_fields_len: usize,
    doc: &mut Document,
    schema: Schema,
) -> Result<String, TantivyGoError> {
    let mut field_to_name = HashMap::new();

    process_string_slice(include_fields_ptr, include_fields_len, |field_name| {
        schema_apply_for_field::<(), (), _>(schema.clone(), field_name, |field, field_name| {
            field_to_name.insert(field, field_name);
            Ok(())
        })
    })?;

    let doc_json = convert_document_to_json(doc, &field_to_name)?;

    Ok(json!(doc_json).to_string())
}

pub fn start_lib_init(log_level: &str, clear_on_panic: bool, utf8_lenient: bool) {
    let old_hook = panic::take_hook();
    if clear_on_panic {
        handle_panic(old_hook);
    }

    set_utf8_lenient(utf8_lenient);

    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .try_init();
}

fn set_utf8_lenient(utf8_lenient: bool) {
    if let Err(e) = config::CONFIG.write().map(|mut config| {
        config.update_utf8_lenient(utf8_lenient);
    }) {
        debug!("Failed to set utf8_lenient: {}", e);
    }
}


fn handle_panic(old_hook: Box<dyn Fn(&PanicInfo) + Sync + Send>) {
    panic::set_hook(Box::new(move |panic_info| {
        if let Ok(config) = config::CONFIG.read() {
            let fts_path = config.fts_path.as_str();
            if fts_path.is_empty() {
                debug!("fts path is empty");
            } else if let Err(e) = fs::remove_dir_all(Path::new(fts_path)) {
                debug!("Failed to remove directory: {}", e);
            }
        } else {
            debug!("Failed to read config.");
        }
        old_hook(panic_info)
    }));
}

pub fn create_context_with_schema(
    schema: Schema,
    path: String,
) -> Result<*mut TantivyContext, TantivyGoError> {
    config::CONFIG
        .write()
        .map_err(|e| TantivyGoError::from_err("Failed to set path", &e.to_string()))?
        .update_fts_path(path.clone());

    fs::create_dir_all(Path::new(&path))
        .map_err(|e| TantivyGoError::from_err("Failed to create directories", &e.to_string()))?;

    let dir =
        MmapDirectory::open(&path).map_err(|err| TantivyGoError::from_str(&err.to_string()))?;

    let ctx = create_tantivy_context(dir, schema)
        .map_err(|err| TantivyGoError::from_str(&err.to_string()))?;

    Ok(Box::into_raw(Box::new(ctx)))
}

fn create_tantivy_context(
    dir: MmapDirectory,
    schema: Schema,
) -> Result<TantivyContext, TantivyError> {
    let index = Index::open_or_create(dir, schema)?;
    let writer = index.writer(DOCUMENT_BUDGET_BYTES)?;
    let reader = index.reader()?;
    Ok(TantivyContext::new(index, writer, reader))
}

pub fn add_and_consume_documents(
    docs_ptr: *mut *mut Document,
    docs_len: usize,
    writer: &mut IndexWriter,
) -> Result<(), TantivyGoError> {
    process_type_slice(docs_ptr, docs_len, |doc| {
        let doc = *box_from(doc);
        let _ = writer.add_document(doc.tantivy_doc);
        Ok(())
    })
    .map_err(|err| {
        rollback(writer);
        TantivyGoError(format!("Failed to add the document: {}", err))
    })?;

    commit(writer, "Failed to commit the document")?;
    Ok(())
}

fn commit(writer: &mut IndexWriter, message: &str) -> Result<Opstamp, TantivyGoError> {
    writer.commit().map_err(|err| {
        rollback(writer);
        TantivyGoError::from_err(message, &err.to_string())
    })
}

pub fn delete_docs<'a>(
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    context: &mut TantivyContext,
    field_name: Cow<'a, str>,
) -> Result<(), TantivyGoError> {
    let schema = context.index.schema();

    let field = schema_apply_for_field::<Field, (), _>(schema.clone(), field_name, |field, _| {
        get_string_field_entry(schema.clone(), field)
    })
    .map_err(|err| {
        rollback(&mut context.writer);
        err
    })?;

    process_string_slice(delete_ids_ptr, delete_ids_len, |id_value| {
        context
            .writer
            .delete_term(Term::from_field_text(field, &id_value));
        Ok(())
    })
    .map_err(|err| {
        rollback(&mut context.writer);
        err
    })?;

    commit(&mut context.writer, "Failed to commit removing")?;
    Ok(())
}

fn rollback(writer: &mut IndexWriter) {
    let _ = writer.rollback();
}

pub fn get_doc<'a>(
    index: usize,
    result: &mut SearchResult,
) -> Result<*mut Document, TantivyGoError> {
    if index >= result.documents.len() {
        return Err(TantivyGoError(
            format!("{} is more than {}", index, result.documents.len() - 1).to_string(),
        ));
    }

    let doc = result.documents[index].clone();
    Ok(Box::into_raw(Box::new(doc)))
}

pub fn add_field<'a>(
    doc: &mut Document,
    index: &Index,
    field_name: Cow<'a, str>,
    field_value: &str,
) -> Result<(), TantivyGoError> {
    let field =
        schema_apply_for_field::<Field, (), _>(index.schema(), field_name, |field, _| Ok(field))
            .map_err(|err| err)?;

    doc.tantivy_doc.add_text(field, field_value);
    Ok(())
}

fn perform_search<F>(
    query_parser_fn: F,
    docs_limit: usize,
    context: &mut TantivyContext,
    with_highlights: bool,
) -> Result<*mut SearchResult, TantivyGoError>
where
    F: FnOnce(&Index) -> Result<Box<dyn Query>, String>,
{
    let searcher = &context.reader().searcher();
    let schema = context.index.schema();

    let query = query_parser_fn(&context.index).map_err(|err| TantivyGoError(err))?;

    let top_docs = searcher
        .search(&query, &tantivy::collector::TopDocs::with_limit(docs_limit))
        .map_err(|err| TantivyGoError::from_err("Search err", &err.to_string()))?;

    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        //let explanation = query.explain(&searcher, doc_address).unwrap();
        //debug!("### exp {:#?}", explanation);
        let doc = searcher
            .doc::<TantivyDocument>(doc_address)
            .map_err(|err| TantivyGoError(err.to_string()))?;
        let highlights = find_highlights(with_highlights, &searcher, &query, &doc, schema.clone())
            .map_err(|err| TantivyGoError(err.to_string()))?;
        documents.push(Document {
            tantivy_doc: doc,
            highlights,
            score,
        });
    }

    let size = documents.len();
    Ok(Box::into_raw(Box::new(SearchResult {
        documents,
        size,
    })))
}

pub fn search(
    field_names_ptr: *mut *const c_char,
    field_weights_ptr: *mut c_float,
    field_names_len: usize,
    query_ptr: *const c_char,
    docs_limit: usize,
    context: &mut TantivyContext,
    with_highlights: bool,
) -> Result<*mut SearchResult, TantivyGoError> {
    let schema = context.index.schema();

    let mut fields = Vec::with_capacity(field_names_len);
    process_string_slice(field_names_ptr, field_names_len, |field_name| {
        schema_apply_for_field::<(), (), _>(schema.clone(), field_name, |field, _| {
            fields.push(field);
            Ok(())
        })
    })?;

    let mut weights = HashMap::with_capacity(field_names_len);
    process_slice(field_weights_ptr, field_names_len, |i, field_weight| {
        weights.insert(fields[i], field_weight);
        Ok(())
    })?;

    let query_str = assert_string(query_ptr)?;

    perform_search(
        |index: &Index| {
            let mut query_parser = QueryParser::for_index(index, fields);
            for (field, weight) in weights {
                query_parser.set_field_boost(field, weight as Score);
            }
            query_parser
                .parse_query(query_str.as_str())
                .map_err(|e| e.to_string())
        },
        docs_limit,
        context,
        with_highlights,
    )
}

pub fn search_json(
    query_ptr: *const c_char,
    docs_limit: usize,
    context: &mut TantivyContext,
    with_highlights: bool,
) -> Result<*mut SearchResult, TantivyGoError> {
    let schema = context.index.schema();

    let query_str = assert_string(query_ptr)?;

    perform_search(
        |index: &Index| {
            parse_query_from_json(index, &schema, &query_str).map_err(|e| e.to_string())
        },
        docs_limit,
        context,
        with_highlights,
    )
}

pub fn drop_any<T>(ptr: *mut T) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}

pub fn box_from<T>(ptr: *mut T) -> Box<T> {
    unsafe { Box::from_raw(ptr) }
}

const POINTER_IS_NULL: &'static str = "Pointer is null";
