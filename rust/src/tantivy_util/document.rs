use crate::tantivy_util::{extract_text_from_owned_value, Document, TantivyGoError};
use std::borrow::Cow;
use std::collections::HashMap;
use tantivy::schema::{Field, Value};

pub fn convert_document_to_json<'a>(
    doc: &mut Document,
    field_to_name: &'a HashMap<Field, Cow<'a, str>>,
) -> Result<HashMap<Cow<'a, str>, serde_json::Value>, TantivyGoError> {
    let mut result_json = HashMap::new();

    let score = serde_json::to_value(doc.score)
        .map_err(|err| TantivyGoError::from_err("Failed to serialize score", &err.to_string()))?;
    result_json.insert(Cow::from("score"), score);

    let highlights = serde_json::to_value(&doc.highlights).map_err(|err| {
        TantivyGoError::from_err("Failed to serialize highlights", &err.to_string())
    })?;
    result_json.insert(Cow::from("highlights"), highlights);

    for (field_value, doc) in doc.tantivy_doc.field_values() {
        let key = match field_to_name.get(&field_value) {
            Some(value) => value,
            None => continue,
        };

        let leaf = match doc.as_leaf() {
            Some(value) => value,
            None => continue,
        };

        let value = extract_text_from_owned_value(&leaf)?;
        let json_value = serde_json::to_value(value).map_err(|err| {
            TantivyGoError::from_err("Failed to serialize field value", &err.to_string())
        })?;
        result_json.insert(Cow::Borrowed(key), json_value);
    }

    Ok(result_json)
}
