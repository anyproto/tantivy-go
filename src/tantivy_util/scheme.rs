use tantivy::schema::{Field, FieldType, Schema};

pub fn get_string_field_entry <'a>(schema: Schema, field: Field) -> Result<Field, &'a str> {
    Ok(match { schema.get_field_entry(field).field_type() } {
        FieldType::Str(_) => field,
        &_ => return Err("wrong field type")
    })
}