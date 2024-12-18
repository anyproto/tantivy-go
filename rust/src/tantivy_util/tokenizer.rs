use tantivy::{Index};
use tantivy::tokenizer::{AsciiFoldingFilter, LowerCaser, NgramTokenizer, RawTokenizer, RemoveLongFilter, SimpleTokenizer, Stemmer, TextAnalyzer};
use crate::tantivy_util::{EdgeNgramTokenizer, TantivyGoError};
use crate::tantivy_util::stemmer::create_stemmer;

fn register_tokenizer(index: &Index, tokenizer_name: &str, text_analyzer: TextAnalyzer) {
    index.tokenizers().register(tokenizer_name, text_analyzer)
}

pub fn register_edge_ngram_tokenizer(
    min_gram: usize,
    max_gram: usize,
    limit: usize,
    index: &Index,
    tokenizer_name: &str,
) {
    let text_analyzer = TextAnalyzer::builder(
        EdgeNgramTokenizer::new(
            min_gram,
            max_gram,
            limit,
        ))
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn register_simple_tokenizer(
    text_limit: usize,
    index: &Index,
    tokenizer_name: &str,
    lang: &str,
) -> Result<(), TantivyGoError> {
    let stemmer = create_stemmer(lang)?;
    let text_analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(text_limit))
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .filter(stemmer)
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
    Ok(())
}


#[cfg(feature = "jieba")]
pub fn register_jieba_tokenizer(
    text_limit: usize,
    index: &Index,
    tokenizer_name: &str,
) {
    let text_analyzer = TextAnalyzer::builder(tantivy_jieba::JiebaTokenizer {})
        .filter(RemoveLongFilter::limit(text_limit))
        .filter(LowerCaser)
        .filter(Stemmer::default())
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
}

#[cfg(not(feature = "jieba"))]
pub fn register_jieba_tokenizer(
    text_limit: usize,
    index: &Index,
    tokenizer_name: &str,
) {
    panic!("Jieba support not compiled in")
}

pub fn register_raw_tokenizer(index: &Index, tokenizer_name: &str) {
    let text_analyzer = TextAnalyzer::builder(RawTokenizer::default()).build();
    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn register_ngram_tokenizer(
    min_gram: usize,
    max_gram: usize,
    prefix_only: bool,
    index: &Index,
    tokenizer_name: &str,
) -> Result<(), TantivyGoError> {
    let tokenizer = NgramTokenizer::new(
        min_gram,
        max_gram,
        prefix_only,
    ).map_err(|e|TantivyGoError::from_err("ngram tokenizer", &e.to_string()))?;

    let text_analyzer = TextAnalyzer::builder(tokenizer)
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
    Ok(())
}