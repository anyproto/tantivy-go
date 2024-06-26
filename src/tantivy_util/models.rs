
use serde::Serialize;
use tantivy::TantivyDocument;

#[derive(Clone)]
pub struct Document {
    pub tantivy_doc: TantivyDocument,
    pub highlights: Vec<Highlight>,
    pub score: usize,
}

#[derive(Clone, Serialize)]
pub struct Highlight {
    pub field_name: String,
    pub fragment: String,
    pub highlighted: Vec<(usize, usize)>,
}

pub struct SearchResult {
    pub documents: Vec<Document>,
    pub size: usize,
}