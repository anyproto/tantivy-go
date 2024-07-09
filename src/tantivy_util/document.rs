use std::collections::HashMap;
use tantivy::schema::Field;
use crate::tantivy_util::{Document, extract_text_from_owned_value};

pub fn convert_document_to_json<'a>(
    doc: &&mut Document,
    field_to_name: HashMap<Field, &'a str>,
) -> HashMap<&'a str, serde_json::Value> {
    let mut result_json: HashMap<&str, serde_json::Value> = HashMap::new();

    let _ = serde_json::to_value(doc.score).is_ok_and(
        |score| result_json.insert("score", score).is_some()
    );

    let _ = serde_json::to_value(&doc.highlights).is_ok_and(
        |highlights| result_json.insert("highlights", highlights).is_some()
    );

    let doc = &doc.tantivy_doc;
    for field_value in doc.field_values() {
        match field_to_name.get(&field_value.field) {
            Some(key) => {
                let _ = extract_text_from_owned_value(&field_value.value).is_some_and(
                    |value| serde_json::to_value(value).is_ok_and(
                        |value| result_json.insert(key, value).is_some())
                );
            }
            None => {}
        }
    }
    result_json
}