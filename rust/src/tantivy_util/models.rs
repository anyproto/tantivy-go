use serde::Serialize;
use tantivy::{Index, IndexReader, IndexWriter, TantivyDocument};


pub struct TantivyContext {
    pub index: Index,
    pub writer: IndexWriter,
    reader: IndexReader,
}

impl TantivyContext {
    pub fn new(index: Index, writer: IndexWriter, reader: IndexReader) -> TantivyContext {
        TantivyContext {
            index,
            writer,
            reader
        }
    }

    pub fn reader(&mut self) -> &IndexReader {
        let _ = self.reader.reload();
        return &self.reader;
    }
}

#[derive(Clone)]
pub struct Document {
    pub tantivy_doc: TantivyDocument,
    pub highlights: Vec<Highlight>,
    pub score: f32,
}

#[derive(Clone, Serialize)]
pub struct Highlight {
    pub field_name: String,
    pub fragment: Fragment,
}

#[derive(Clone, Serialize)]
pub struct Fragment {
    pub t: String, //to comply with bleve temporarily
    pub r: Vec<(usize, usize)>, //to comply with bleve temporarily
}

pub struct SearchResult {
    pub documents: Vec<Document>,
    pub size: usize,
}