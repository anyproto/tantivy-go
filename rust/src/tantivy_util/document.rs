use std::borrow::Cow;
use std::collections::HashMap;
use tantivy::schema::Field;
use crate::tantivy_util::{Document, extract_text_from_owned_value};

pub fn convert_document_to_json<'a>(
    doc: &mut Document,
    field_to_name: &'a HashMap<Field, Cow<'a, str>>,
) -> HashMap<Cow<'a, str>, serde_json::Value> {
    let mut result_json: HashMap<Cow<'a, str>, serde_json::Value> = HashMap::new();

    let _ = serde_json::to_value(doc.score).is_ok_and(
        |score| result_json.insert(Cow::from("score"), score).is_some()
    );

    let _ = serde_json::to_value(&doc.highlights).is_ok_and(
        |highlights| result_json.insert(Cow::from("highlights"), highlights).is_some()
    );

    let doc = &doc.tantivy_doc;
    for field_value in doc.field_values() {
        match field_to_name.get(&field_value.field) {
            Some(key) => {
                let _ = extract_text_from_owned_value(&field_value.value).is_some_and(
                    |value| serde_json::to_value(value).is_ok_and(
                        |value| result_json.insert(Cow::Borrowed(key), value).is_some())
                );
            }
            None => {}
        }
    }
    result_json
}