use tantivy::schema::{
    IndexRecordOption, NumericOptions, SchemaBuilder, TextFieldIndexing, COERCE, FAST, INDEXED,
    STORED, STRING, TEXT,
};

pub fn add_text_field(
    stored: bool,
    is_text: bool,
    is_fast: bool,
    builder: &mut SchemaBuilder,
    tokenizer_name: &str,
    field_name: &str,
    index_record_option: IndexRecordOption,
) -> u32 {
    let mut text_options = if is_text { TEXT } else { STRING };
    text_options = if stored {
        text_options | STORED
    } else {
        text_options
    };
    text_options = if is_fast {
        text_options | FAST
    } else {
        text_options
    };
    text_options = text_options.set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(tokenizer_name)
            .set_index_option(index_record_option),
    );
    builder.add_text_field(field_name, text_options).field_id()
}

pub fn add_i64_field(
    stored: bool,
    indexed: bool,
    fieldnorms: bool, // This attribute only has an effect if indexed is true.
    fast: bool,
    builder: &mut SchemaBuilder,
    coerce: bool,
    field_name: &str,
) -> u32 {
    let mut i64_options = NumericOptions::default();
    if indexed {
        i64_options = i64_options | INDEXED;
        if fieldnorms {
            i64_options = i64_options.set_fieldnorm();
        }
    };
    if stored {
        i64_options = i64_options | STORED;
    };
    if fast {
        i64_options = i64_options | FAST;
    };
    if coerce {
        i64_options = i64_options | COERCE;
    };
    builder.add_i64_field(field_name, i64_options).field_id()
}
