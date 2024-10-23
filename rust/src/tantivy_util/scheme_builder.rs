use tantivy::schema::{FAST, IndexRecordOption, SchemaBuilder, STORED, STRING, TEXT, TextFieldIndexing};

pub fn add_text_field(
    stored: bool,
    is_text: bool,
    is_fast: bool,
    builder: &mut SchemaBuilder,
    tokenizer_name: &str,
    field_name: &str,
    index_record_option: IndexRecordOption,
) {
    let mut text_options = if is_text { TEXT } else { STRING };
    text_options = if stored { text_options | STORED } else { text_options };
    text_options = if is_fast { text_options | FAST } else { text_options };
    text_options = text_options.set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(tokenizer_name)
            .set_index_option(index_record_option)
    );
    builder.add_text_field(field_name, text_options);
}

