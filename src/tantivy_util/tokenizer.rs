use tantivy::{Index, TantivyError};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, RawTokenizer, RemoveLongFilter, SimpleTokenizer, TextAnalyzer};
use crate::tantivy_util::{EdgeNgramTokenizer};
use crate::tantivy_util::stemmer::create_stemmer;

fn register_tokenizer(index: &mut Index, tokenizer_name: &str, text_analyzer: TextAnalyzer) {
    index.tokenizers().register(tokenizer_name, text_analyzer)
}

pub fn register_edge_ngram_tokenizer(
    min_gram: usize,
    max_gram: usize,
    limit: usize,
    index: &mut Index,
    tokenizer_name: &str
) {
    let text_analyzer = TextAnalyzer::builder(
        EdgeNgramTokenizer::new(
            min_gram,
            max_gram,
            limit
        ))
        .filter(LowerCaser)
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn register_simple_tokenizer(
    text_limit: usize,
    index: &mut Index,
    tokenizer_name: &str,
    lang: &str
) {
    let text_analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(text_limit))
        .filter(LowerCaser)
        .filter(create_stemmer(lang))
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn register_raw_tokenizer(index: &mut Index, tokenizer_name: &str) {
    let text_analyzer = TextAnalyzer::builder(RawTokenizer::default()).build();
    register_tokenizer(index, tokenizer_name, text_analyzer);
}

pub fn register_ngram_tokenizer(
    min_gram: usize,
    max_gram: usize,
    prefix_only: bool,
    index: &mut Index,
    tokenizer_name: &str
) -> Result<(), TantivyError> {

    let tokenizer = NgramTokenizer::new(
        min_gram,
        max_gram,
        prefix_only,
    )?;

    let text_analyzer = TextAnalyzer::builder(tokenizer)
        .filter(LowerCaser)
        .build();

    register_tokenizer(index, tokenizer_name, text_analyzer);
    return Ok(());
}