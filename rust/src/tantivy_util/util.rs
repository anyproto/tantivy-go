use std::borrow::Cow;
use tantivy::schema::OwnedValue;

pub fn extract_text_from_owned_value<'a>(value: &'a OwnedValue) -> Option<Cow<'a, str>> {
    match value {
        OwnedValue::Str(text) => Some(Cow::Borrowed(text)),
        _ => { None }
    }
}

pub const DOCUMENT_BUDGET_BYTES: usize = 50_000_000;