use serde::Deserialize;
use serde_json::Deserializer;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::path::Path;
use std::{fs, io};
use tantivy::collector::TopDocs;
use tantivy::query::{PhrasePrefixQuery, QueryParser};
use tantivy::{schema::*, Index};
use tantivy::tokenizer::{AsciiFoldingFilter, Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer, TextAnalyzer};

#[derive(Debug, Deserialize)]
struct DocumentData {
    id: String,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct Query {
    query: String,
    relevant_docs: Vec<String>,
}

fn register_tokenizer(index: &Index, tokenizer_name: &str, text_analyzer: TextAnalyzer) {
    index.tokenizers().register(tokenizer_name, text_analyzer)
}

pub fn register_simple_tokenizer(
    text_limit: usize,
    index: &Index,
    tokenizer_name: &str,
) {
    let text_analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(text_limit))
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .filter(Stemmer::new(Language::English))
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn add_text_field(
    stored: bool,
    is_text: bool,
    is_fast: bool,
    builder: &mut SchemaBuilder,
    tokenizer_name: &str,
    field_name: &str,
    index_record_option: IndexRecordOption,
) -> Field {
    let mut text_options = if is_text { TEXT } else { STRING };
    text_options = if stored { text_options | STORED } else { text_options };
    text_options = if is_fast { text_options | FAST } else { text_options };
    text_options = text_options.set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(tokenizer_name)
            .set_index_option(index_record_option)
    );
    builder.add_text_field(field_name, text_options)
}

fn main() -> Result<(), Box<dyn Error>> {
    let index_dir = "../tantivy_index";

    if !Path::new(index_dir).exists() {
        fs::create_dir_all(index_dir)?;
    }

    let mut schema_builder = Schema::builder();

    let title_field = add_text_field(
        true,
        true,
        false,
        &mut schema_builder,
        "simple_tokenizer",
        "title",
        IndexRecordOption::WithFreqsAndPositions
    );

    let body_field = add_text_field(
        true,
        true,
        false,
        &mut schema_builder,
        "simple_tokenizer",
        "body",
        IndexRecordOption::WithFreqsAndPositions
    );

    let id_field = add_text_field(
        true,
        false,
        false,
        &mut schema_builder,
        "simple_tokenizer",
        "id",
        IndexRecordOption::WithFreqsAndPositions
    );

    let schema = schema_builder.build();

    let index = if Path::new(index_dir).read_dir()?.next().is_none() {
        let index = Index::create_in_dir(index_dir, schema.clone())?;
        register_simple_tokenizer(40, &index, "simple_tokenizer");

        let mut index_writer = index.writer(50_000_000)?;

        let file = File::open("../documents.json")?;
        let reader = BufReader::new(file);
        let docs: Vec<DocumentData> = serde_json::from_reader(reader)?;
        for doc_result in docs {
            let doc_data = doc_result;
            let mut doc = TantivyDocument::default();
            doc.add_text(title_field, &doc_data.title);
            doc.add_text(body_field, &doc_data.body);
            doc.add_text(id_field, &doc_data.id);
            index_writer.add_document(doc);
        }

        index_writer.commit()?;
        index
    } else {
        let index = Index::open_in_dir(index_dir)?;
        register_simple_tokenizer(40, &index, "simple_tokenizer");
        index
    };

    println!("All docs has been added to the index.");
    Ok(())
}
