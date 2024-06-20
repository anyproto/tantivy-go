use tantivy::schema::OwnedValue;

pub fn extract_text_from_owned_value(value: &OwnedValue) -> Option<&str> {
    match value {
        OwnedValue::Str(text) => Some(text),
        _ => { None }
    }
}

pub const DOCUMENT_BUDGET_BYTES: usize = 50_000_000;