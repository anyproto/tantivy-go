use crate::tantivy_util::TantivyGoError;
use tantivy::schema::{Field, FieldType, Schema};

pub fn get_string_field_entry(schema: Schema, field: Field) -> Result<Field, TantivyGoError> {
    let field_type = schema.get_field_entry(field).field_type();
    Ok(match { field_type } {
        FieldType::Str(_) => field,
        &_ => {
            return Err(TantivyGoError("Only FieldType::Str has been supported yet".to_string()))
        }
    })
}
