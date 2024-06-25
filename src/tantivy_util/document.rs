use std::collections::HashMap;
use tantivy::schema::Field;
use crate::tantivy_util::{Document, extract_text_from_owned_value};

pub fn convert_document_to_json<'a>(
    doc: &&mut Document,
    field_to_name: HashMap<Field, &'a str>,
) -> HashMap<&'a str, serde_json::Value> {
    let mut result_json: HashMap<&str, serde_json::Value> = HashMap::new();
    result_json.insert("score", serde_json::to_value(doc.score).unwrap());
    let doc = &doc.tantivy_doc;
    for field_value in doc.field_values() {
        match field_to_name.get(&field_value.field) {
            Some(key) => {
                result_json.insert(key, serde_json::to_value(
                    extract_text_from_owned_value(
                        &field_value.value).unwrap()
                ).unwrap(), );
            }
            None => {}
        }
    }
    result_json
}