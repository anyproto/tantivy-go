use crate::tantivy_util::{Fragment, Highlight};
use tantivy::query::Query;
use tantivy::schema::Schema;
use tantivy::{Searcher, TantivyDocument, TantivyError};
use tantivy::snippet::SnippetGenerator;

pub fn find_highlights(
    with_highlights: bool,
    searcher: &Searcher,
    query: &Box<dyn Query>,
    doc: &TantivyDocument,
    schema: Schema,
) -> Result<Vec<Highlight>, TantivyError> {
    let mut highlights: Vec<Highlight> = vec![];
    if with_highlights {
        for (field_value, _) in doc.field_values() {
            let snippet_generator = SnippetGenerator::create(
                &searcher, query, field_value)?;
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
                field_name: schema.get_field_name(field_value).to_string(),
                fragment: Fragment {
                    t: snippet.fragment().to_owned(),
                    r: highlighted,
                },
            });
        }
    }
    Ok(highlights)
}