use base64::Engine;
use base64::engine::general_purpose;
use tantivy::query::Query;
use tantivy::{Searcher, SnippetGenerator, TantivyDocument, TantivyError};
use tantivy::schema::Schema;
use crate::tantivy_util::{Fragment, Highlight};

pub fn find_highlights(
    with_highlights: bool,
    searcher: &Searcher,
    query: &Box<dyn Query>,
    doc: &TantivyDocument,
    schema: Schema,
) -> Result<Vec<Highlight>, TantivyError> {
    let mut highlights: Vec<Highlight> = vec![];
    if with_highlights {
        for field_value in doc.field_values() {
            let snippet_generator = SnippetGenerator::create(
                &searcher, query, field_value.field)?;
            let snippet = snippet_generator.snippet_from_doc(doc);
            let highlighted: Vec<(usize, usize)> = snippet.highlighted()
                .to_owned()
                .iter()
                .filter_map(|highlight| {
                    if highlight.is_empty() { None } else { Some((highlight.start, highlight.end)) }
                }).collect();

            if highlighted.is_empty() {
                continue;
            }
            highlights.push(Highlight {
                field_name: schema.get_field_name(field_value.field).to_string(),
                fragment: Fragment {
                    t: general_purpose::STANDARD.encode(&snippet.fragment().to_owned()), //to comply with bleve temporarily
                    r: highlighted,
                },
            });
        }
    }
    Ok(highlights)
}

mod tests {
    use tantivy::tokenizer::*;
    use tantivy::schema::*;
    use tantivy::{Index, DocAddress, doc, DocId};
    use tantivy::collector::TopDocs;
    use tantivy::query::QueryParser;
    use tantivy::schema::document::DocumentDeserialize;

    #[test]
    fn test_ascii_folding_filter() {
        // Определяем схему
        let mut schema_builder = Schema::builder();
        let mut text_options = TEXT;
        text_options = text_options | STORED;
        text_options = text_options.set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("custom")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        );
        let text = schema_builder.add_text_field("text", text_options);
        let schema = schema_builder.build();

        // Создаем индекс
        let index = Index::create_in_ram(schema.clone());

        // Создаем кастомный токенайзер с AsciiFoldingFilter
        let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .build();

        index.tokenizers().register("custom", tokenizer);

        // Добавляем документы в индекс
        let mut index_writer = index.writer(50_000_000).unwrap();
        index_writer.add_document(doc!(text => "strasse"));
        index_writer.add_document(doc!(text => "straße"));
        index_writer.commit().unwrap();

        // Создаем QueryParser с кастомным токенайзером
        let query_parser = QueryParser::for_index(&index, vec![text]);

        // Выполняем поиск по "strasse"
        let searcher = index.reader().unwrap().searcher();
        let query = query_parser.parse_query("straße").unwrap();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        assert_eq!(top_docs.len(), 2);

        // Проверяем совпадение документов
        if let Some((_, doc_address)) = top_docs.get(0) {
            let first_doc : TantivyDocument = searcher.doc(*doc_address).unwrap();
            let first_text = first_doc.get_first(text).unwrap().as_str().unwrap();
            assert!(first_text == "strasse" || first_text == "straße");
        } else {
            panic!("First document not found");
        }

        if let Some((_, doc_address)) = top_docs.get(1) {
            let second_doc : TantivyDocument= searcher.doc(*doc_address).unwrap();
            let second_text = second_doc.get_first(text).unwrap().as_str().unwrap();
            assert!(second_text == "strasse" || second_text == "straße");
        } else {
            panic!("Second document not found");
        }
    }
}