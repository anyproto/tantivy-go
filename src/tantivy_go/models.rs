use tantivy::TantivyDocument;

#[derive(Clone)]
pub struct Document {
    pub tantivy_doc: TantivyDocument,
    pub score: usize,
}

pub struct SearchResult {
    pub documents: Vec<Document>,
    pub size: usize,
}