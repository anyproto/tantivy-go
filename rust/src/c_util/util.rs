use std::{fmt, fs, panic, slice};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float};
use std::panic::PanicInfo;
use std::path::Path;
use log::debug;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::Visitor;
use serde_json::json;
use tantivy::{Index, IndexWriter, Score, TantivyDocument, TantivyError, Term};
use tantivy::directory::MmapDirectory;
use tantivy::query::{BooleanQuery, Occur, PhraseQuery, Query, QueryParser};
use tantivy::query_grammar::parse_query;
use tantivy::schema::{Field, Schema};
use crate::c_util::util::GoQuery::PhrasePrefixQuery;
use crate::config;
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


fn process_c_str<'a>(str_ptr: *const c_char, error_buffer: *mut *mut c_char) -> Result<Cow<'a, str>, String> {
    unsafe {
        if str_ptr.is_null() {
            set_error(POINTER_IS_NULL, error_buffer);
            return Err(POINTER_IS_NULL.to_owned());
        }
        let is_lenient = match config::CONFIG.read() {
            Ok(config) => {
                config.utf8_lenient
            }
            Err(err) => {
                let error_message = err.to_string();
                set_error(&error_message, error_buffer);
                return Err(error_message);
            }
        };
        let cstr = CStr::from_ptr(str_ptr);
        if is_lenient {
            Ok(cstr.to_string_lossy())
        } else {
            match cstr.to_str() {
                Ok(valid_str) => Ok(Cow::Borrowed(valid_str)),
                Err(err) => {
                    let error_message = err.to_string();
                    set_error(&error_message, error_buffer);
                    Err(error_message)
                }
            }
        }
    }
}

// Always copy long-living strings for safety reasons
pub fn assert_string(str_ptr: *const c_char, error_buffer: *mut *mut c_char) -> Option<String> {
    match process_c_str(str_ptr, error_buffer) {
        Ok(Cow::Borrowed(original_str)) => Some(original_str.to_owned()),
        Ok(Cow::Owned(fixed_str)) => Some(fixed_str),
        Err(_) => None,
    }
}

// Try not to copy one-time-living strings if possible
pub fn assert_str<'a>(str_ptr: *const c_char, error_buffer: *mut *mut c_char) -> Option<Cow<'a, str>> {
    match process_c_str(str_ptr, error_buffer) {
        Err(_) => None,
        Ok(res) => Some(res)
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
    F: FnMut(Cow<'a, str>) -> Result<(), ()>,
{
    let slice = match assert_pointer(ptr, error_buffer) {
        Some(ptr) => unsafe { slice::from_raw_parts(ptr, len) },
        None => return Err(()),
    };

    for &item in slice {
        let value = match assert_str(item, error_buffer) {
            Some(value) => value,
            None => return Err(()),
        };

        if func(value).is_err() {
            return Err(());
        }
    }

    Ok(())
}

pub fn process_slice<'a, F, T>(
    ptr: *mut T,
    error_buffer: *mut *mut c_char,
    len: usize,
    mut func: F,
) -> Result<(), ()>
where
    F: FnMut(usize, T) -> Result<(), ()>,
    T: Copy,
{
    let slice = match assert_pointer(ptr, error_buffer) {
        Some(ptr) => unsafe { slice::from_raw_parts(ptr, len) },
        None => return Err(()),
    };

    for (i, item) in slice.iter().enumerate() {
        if func(i, *item).is_err() {
            return Err(());
        }
    }

    Ok(())
}

pub fn schema_apply_for_field<'a, T, K, F: FnMut(Field, Cow<'a, str>) -> Result<T, ()>>(
    error_buffer: *mut *mut c_char,
    schema: Schema,
    field_name: Cow<'a, str>,
    mut func: F,
) -> Result<T, ()>
{
    match schema.get_field(&field_name) {
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
    doc: &mut Document,
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

    let doc_json = convert_document_to_json(doc, &field_to_name);

    Ok(json!(doc_json).to_string())
}

pub fn start_lib_init(
    log_level: &str,
    clear_on_panic: bool,
    utf8_lenient: bool,
) {
    let old_hook = panic::take_hook();
    if clear_on_panic {
        handle_panic(old_hook);
    }

    set_utf8_lenient(utf8_lenient);

    let _ = env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level)
    ).try_init();
}

fn set_utf8_lenient(utf8_lenient: bool) {
    match config::CONFIG.write() {
        Ok(mut config) => {
            config.update_utf8_lenient(utf8_lenient);
        }
        Err(e) => {
            debug!("Set utf8_lenient err: {}", e);
        }
    }
}

fn handle_panic(old_hook: Box<dyn Fn(&PanicInfo) + Sync + Send>) {
    panic::set_hook(Box::new(move |panic_info| {
        match config::CONFIG.read() {
            Ok(config) => {
                let fts_path = config.fts_path.as_str();
                if fts_path.is_empty() {
                    debug!("fts path is empty");
                } else {
                    let _ = fs::remove_dir_all(Path::new(fts_path));
                }
            }
            Err(e) => {
                debug!("Set hook err: {}", e);
            }
        }
        old_hook(panic_info)
    }));
}

pub fn create_context_with_schema(
    error_buffer: *mut *mut c_char,
    schema: Schema,
    path: String,
) -> Result<*mut TantivyContext, ()> {
    match config::CONFIG.write() {
        Ok(mut config) => {
            config.update_fts_path(path.clone());
        }
        Err(e) => { debug!("Failed to set path: {}", e) }
    }
    match fs::create_dir_all(Path::new(path.as_str())) {
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
        rollback(error_buffer, writer, "Failed to add the document");
        return;
    }

    commit(writer, "Failed to commit the document", error_buffer)
}

fn commit(writer: &mut IndexWriter, message: &str, error_buffer: *mut *mut c_char) {
    let result = writer.commit();

    if result.is_err() {
        rollback(error_buffer, writer, format!("{}: {}", message, result.unwrap_err()).as_str());
    }
}

pub fn delete_docs<'a>(
    delete_ids_ptr: *mut *const c_char,
    delete_ids_len: usize,
    error_buffer: *mut *mut c_char,
    context: &mut TantivyContext,
    field_name: Cow<'a, str>,
) {
    let schema = context.index.schema();

    let field = match schema_apply_for_field::<Field, (), _>
        (error_buffer, schema.clone(), field_name, |field, _|
            match get_string_field_entry(schema.clone(), field) {
                Ok(value) => Ok(value),
                Err(_) => Err(())
            },
        ) {
        Ok(value) => value,
        Err(_) => {
            rollback(error_buffer, &mut context.writer, "Failed to apply schema for field");
            return;
        }
    };

    if process_string_slice(delete_ids_ptr, error_buffer, delete_ids_len, |id_value| {
        let _ = context.writer.delete_term(Term::from_field_text(field, &id_value));
        Ok(())
    }).is_err() {
        rollback(error_buffer, &mut context.writer, "Failed to process string slice");
        return;
    }

    commit(&mut context.writer, "Failed to commit removing", error_buffer);
}

fn rollback(
    error_buffer: *mut *mut c_char,
    writer: &mut IndexWriter,
    message: &str,
) {
    let _ = writer.rollback();
    set_error(message, error_buffer);
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

pub fn add_field<'a>(
    error_buffer: *mut *mut c_char,
    doc: &mut Document,
    index: &Index,
    field_name: Cow<'a, str>,
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
    field_weights_ptr: *mut c_float,
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

    let mut weights = HashMap::with_capacity(field_names_len);

    if process_slice(field_weights_ptr, error_buffer, field_names_len, |i, field_weight| {
        weights.insert(fields[i], field_weight);
        Ok(())
    }).is_err() {
        return Err(());
    }

    let query = match assert_string(query_ptr, error_buffer) {
        Some(value) => value,
        None => return Err(())
    };

    let mut query_parser = QueryParser::for_index(&context.index, fields);
    for (field, weight) in weights {
        query_parser.set_field_boost(field, weight as Score);
    }

    let query = match query_parser.parse_query(query.as_str()) {
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

    let size = documents.len();
    Ok(Box::into_raw(Box::new(SearchResult {
        documents,
        size,
    })))
}

pub fn search2(
    query_ptr: *const c_char,
    error_buffer: *mut *mut c_char,
    docs_limit: usize,
    context: &mut TantivyContext,
    with_highlights: bool,
) -> Result<*mut SearchResult,  Box<dyn Error>> {
    let searcher = &context.reader().searcher();
    let schema = context.index.schema();

    let query = match assert_string(query_ptr, error_buffer) {
        Some(value) => value,
        None => return Err(Box::new(fmt::Error))
    };
    debug!("###1 {:?}", query);

    let query = parse_query_from_json(&context.index, &schema, &query)?;
    debug!("###2 {:?}", query);

    let top_docs = match searcher.search(
        &query,
        &tantivy::collector::TopDocs::with_limit(docs_limit),
    ) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            set_error(&err.to_string(), error_buffer);
            return Err(Box::new(fmt::Error));
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
                        return Err(Box::new(fmt::Error));
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
                return Err(Box::new(fmt::Error));
            }
        };
    }

    let size = documents.len();
    Ok(Box::into_raw(Box::new(SearchResult {
        documents,
        size,
    })))
}

fn parse_query_from_json(
    index: &Index,
    schema: &Schema,
    json: &str) -> Result<Box<dyn Query>, Box<dyn Error>> {
    debug!("### 31 {:?}", json);
    let parsed = serde_json::from_str(json)?;
    debug!("### 32 {:?}", parsed);
    convert_to_tantivy(index, parsed, schema)
}

// Convert your `QueryModifier` to Tantivy's `Occur`
fn modifier_to_occur(modifier: &QueryModifier) -> Occur {
    match modifier {
        QueryModifier::Must => Occur::Must,
        QueryModifier::Should => Occur::Should,
        QueryModifier::MustNot => Occur::MustNot,
    }
}

// Main conversion function
fn convert_to_tantivy(
    index: &Index,
    parsed: FinalQuery,
    schema: &Schema,
) -> Result<Box<dyn Query>, Box<dyn Error>> {
    // Validate the schema and ensure field mappings exist
    if parsed.fields.is_empty() || parsed.texts.is_empty() {
        return Err("Fields or texts cannot be empty".into());
    }

    // Recursive function to convert `QueryElement` to Tantivy's queries
    fn element_to_query(
        index: &Index,
        element: &QueryElement,
        schema: &Schema,
        texts: &[String],
        fields: &[String],
    ) -> Result<(Occur, Box<dyn Query>), Box<dyn Error>> {

        let occur = modifier_to_occur(&element.modifier);

        if let Some(go_query) = &element.query {
            match go_query {
                GoQuery::PhraseQuery {
                    field_index,
                    text_index,
                    boost: _,
                } => {
                    let field = fields.get(*field_index)
                        .ok_or("Invalid field index in PhraseQuery")?;
                    let text = texts.get(*text_index)
                        .ok_or("Invalid text index in PhraseQuery")?;
                    let field = schema.get_field(field)
                        .or(Err("Invalid field name"))?;

                    let terms =  exract_terms(&index, field, text)?;
                    let phrase_query = PhraseQuery::new(terms);
                    Ok((occur, Box::new(phrase_query)))
                }

                GoQuery::PhrasePrefixQuery {
                    field_index,
                    text_index,
                    boost: _,
                } => {
                    let field = fields.get(*field_index)
                        .ok_or("Invalid field index in PhraseQuery")?;
                    let text = texts.get(*text_index)
                        .ok_or("Invalid text index in PhraseQuery")?;
                    let field = schema.get_field(field)
                        .or(Err("Invalid field name"))?;

                    let terms =  exract_terms(&index, field, text)?;
                    let phrase_query = tantivy::query::PhrasePrefixQuery::new(terms);
                    Ok((occur, Box::new(phrase_query)))
                }

                GoQuery::SingleTermPrefixQuery {
                    field_index,
                    text_index,
                    boost: _,
                } => {
                    let field = fields.get(*field_index)
                        .ok_or("Invalid field index in PhraseQuery")?;
                    let text = texts.get(*text_index)
                        .ok_or("Invalid text index in PhraseQuery")?;
                    let field = schema.get_field(field)
                        .or(Err("Invalid field name"))?;

                    let terms =  exract_terms(&index, field, text)?;
                    let phrase_query = tantivy::query::PhrasePrefixQuery::new(vec![terms[0].clone()]); //todo
                    Ok((occur, Box::new(phrase_query)))
                }

                GoQuery::BoolQuery { subqueries } => {
                    let mut sub_queries = vec![];
                    for subquery in subqueries {
                        sub_queries.push(element_to_query(index, subquery, schema, texts, fields)?);
                    }
                    let bool_query = BooleanQuery::from(sub_queries);
                    Ok((occur, Box::new(bool_query)))
                }
                _ => Err("Unsupported GoQuery variant".into()),
            }
        } else {
            Err("Query is None in QueryElement".into())
        }
    }
    fn exract_terms(
        index: &Index,
        field: Field,
        query: &str
    ) -> Result<(Vec<Term>), Box<dyn Error>> {
        let mut tokenizer = index.tokenizer_for_field(field)?;
        let mut token_stream = tokenizer.token_stream(query);
        let mut terms = Vec::new();
        while token_stream.advance() {
            terms.push(token_stream.token().text.clone());
            //println!("### {}", token_stream.token().text); // Выводим текст каждого токена
        }
        let term_queries: Vec<Term> = terms
            .iter()
            .map(|term| Term::from_field_text(field, term))
            .collect();
        Ok(term_queries)
    }

    // Convert top-level BoolQuery
    let mut sub_queries = vec![];
    for subquery in &parsed.query.subqueries {
        sub_queries.push(element_to_query(
            index,
            subquery,
            schema,
            &parsed.texts,
            &parsed.fields,
        )?);
    }

    let bool_query = BooleanQuery::from(sub_queries);
    Ok(Box::new(bool_query))
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    BoolQuery,
    PhraseQuery,
    PhrasePrefixQuery,
    SingleTermPrefixQuery,
    None,
}

impl QueryType {
    fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(QueryType::BoolQuery),
            1 => Some(QueryType::PhraseQuery),
            2 => Some(QueryType::PhrasePrefixQuery),
            3 => Some(QueryType::SingleTermPrefixQuery),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for QueryType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryTypeVisitor;

        impl<'de> Visitor<'de> for QueryTypeVisitor {
            type Value = QueryType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number representing the QueryType")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                QueryType::from_u64(value).ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(value), &self))
            }
        }

        deserializer.deserialize_u64(QueryTypeVisitor)
    }
}

impl<'de> Deserialize<'de> for QueryModifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryModifierVisitor;

        impl<'de> Visitor<'de> for QueryModifierVisitor {
            type Value = QueryModifier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number representing the QueryType")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                QueryModifier::from_u64(value).ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(value), &self))
            }
        }

        deserializer.deserialize_u64(QueryModifierVisitor)
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryModifier {
    Must,
    Should,
    MustNot,
}

impl QueryModifier {
    fn from_u64(val: u64) -> Option<Self> {
        match val {
            0 =>  Some(QueryModifier::Must),
            1 =>  Some(QueryModifier::Should),
            2 =>  Some(QueryModifier::MustNot),
            _ =>  None
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GoQuery {
    BoolQuery {
        subqueries: Vec<QueryElement>,
    },
    PhraseQuery {
        field_index: usize,
        text_index: usize,
        boost: f64,
    },
    PhrasePrefixQuery {
        field_index: usize,
        text_index: usize,
        boost: f64,
    },
    SingleTermPrefixQuery {
        field_index: usize,
        text_index: usize,
        boost: f64,
    },
}


#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct QueryElement {
    pub query: Option<GoQuery>,
    pub modifier: QueryModifier,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BoolQuery {
    pub subqueries: Vec<QueryElement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct FinalQuery {
    pub texts: Vec<String>,
    pub fields: Vec<String>,
    pub query: BoolQuery,
}

impl<'de> Deserialize<'de> for QueryElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Используем Value для начального этапа
        let map: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        // Десериализуем modifier как QueryModifier
        let modifier = map
            .get("query_modifier")
            .ok_or_else(|| serde::de::Error::missing_field("query_modifier"))?
            .as_u64()
            .and_then(QueryModifier::from_u64)
            .ok_or_else(|| serde::de::Error::custom("Invalid query_modifier"))?;

        // Десериализуем query_type как QueryType
        let query_type = map
            .get("query_type")
            .ok_or_else(|| serde::de::Error::missing_field("query_type"))?
            .as_u64()
            .and_then(QueryType::from_u64)
            .ok_or_else(|| serde::de::Error::custom("Invalid query_type"))?;

        // Обрабатываем поле query
        let query = match query_type {
            QueryType::BoolQuery => {
                let subqueries = map
                    .get("query")
                    .and_then(|q| q.get("subqueries"))
                    .ok_or_else(|| serde::de::Error::missing_field("subqueries"))?;
                Some(GoQuery::BoolQuery {
                    subqueries: serde_json::from_value(subqueries.clone())
                        .map_err(serde::de::Error::custom)?,
                })
            }
            QueryType::PhraseQuery => {
                let query_data = map
                    .get("query")
                    .and_then(|q| q.as_object())
                    .ok_or_else(|| serde::de::Error::missing_field("query"))?;
                Some(GoQuery::PhraseQuery {
                    field_index: query_data
                        .get("field_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    text_index: query_data
                        .get("text_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    boost: query_data
                        .get("boost")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0),
                })
            }
            QueryType::PhrasePrefixQuery => {
                let query_data = map
                    .get("query")
                    .and_then(|q| q.as_object())
                    .ok_or_else(|| serde::de::Error::missing_field("query"))?;
                Some(GoQuery::PhrasePrefixQuery {
                    field_index: query_data
                        .get("field_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    text_index: query_data
                        .get("text_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    boost: query_data
                        .get("boost")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0),
                })
            }
            QueryType::SingleTermPrefixQuery => {
                let query_data = map
                    .get("query")
                    .and_then(|q| q.as_object())
                    .ok_or_else(|| serde::de::Error::missing_field("query"))?;
                Some(GoQuery::SingleTermPrefixQuery {
                    field_index: query_data
                        .get("field_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    text_index: query_data
                        .get("text_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize,
                    boost: query_data
                        .get("boost")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0),
                })
            }
            QueryType::None => None,
        };

        // Создаем и возвращаем QueryElement
        Ok(QueryElement { query, modifier })
    }
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