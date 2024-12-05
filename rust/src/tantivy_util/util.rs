use std::borrow::Cow;
use std::error::Error;
use tantivy::schema::{Field, OwnedValue};
use tantivy::tokenizer::{Token, TokenStream};
use tantivy::{Index, Term};

pub fn extract_text_from_owned_value<'a>(value: &'a OwnedValue) -> Option<Cow<'a, str>> {
    match value {
        OwnedValue::Str(text) => Some(Cow::Borrowed(text)),
        _ => None,
    }
}

pub fn extract_terms(
    index: &Index,
    field: Field,
    query: &str,
) -> Result<Vec<(usize, Term)>, Box<dyn Error>> {
    let mut tokenizer = index.tokenizer_for_field(field)?;
    let mut token_stream = tokenizer.token_stream(query);
    let mut terms = Vec::new();
    token_stream
        .process(&mut |token: &Token| terms.push((token.position, Term::from_field_text(field, &token.text))));
    if terms.len() > 0 {
        Ok(terms)
    } else {
        Err("Zero terms were extracted".into())
    }
}

pub const DOCUMENT_BUDGET_BYTES: usize = 50_000_000;
