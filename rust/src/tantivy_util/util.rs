use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use tantivy::schema::{Field, OwnedValue};
use tantivy::tokenizer::{Token, TokenStream};
use tantivy::{Index, Term};

pub const DOCUMENT_BUDGET_BYTES: usize = 50_000_000;

pub fn extract_text_from_owned_value<'a>(
    value: &'a OwnedValue,
) -> Result<Cow<'a, str>, TantivyGoError> {
    if let OwnedValue::Str(text) = value {
        Ok(Cow::Borrowed(text))
    } else {
        Err(TantivyGoError("Only OwnedValue::Str is supported".to_string()))
    }
}

pub fn extract_terms(
    index: &Index,
    field: Field,
    query: &str,
) -> Result<Vec<(usize, Term)>, TantivyGoError> {
    let mut tokenizer = match index.tokenizer_for_field(field) {
        Ok(tokenizer) => tokenizer,
        Err(err) => return Err(TantivyGoError::from_err("", &err.to_string())),
    };
    let mut token_stream = tokenizer.token_stream(query);
    let mut terms = Vec::new();
    token_stream.process(&mut |token: &Token| {
        terms.push((token.position, Term::from_field_text(field, &token.text)))
    });
    if terms.len() > 0 {
        Ok(terms)
    } else {
        Err(TantivyGoError("Zero terms were extracted".to_string()))
    }
}

#[derive(Debug)]
pub struct TantivyGoError(pub String);

impl TantivyGoError {
    pub fn from_err(message: &str, err: &str) -> TantivyGoError {
        TantivyGoError(format!("{}: {}", message, err).to_string())
    }

    pub fn from_str(value: &str) -> TantivyGoError {
        TantivyGoError(value.to_string())
    }
}

impl fmt::Display for TantivyGoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TantivyGoError {}
