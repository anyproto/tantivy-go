use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    BoolQuery,
    PhraseQuery,
    PhrasePrefixQuery,
    TermPrefixQuery,
    TermQuery,
    EveryTermQuery,
    OneOfTermQuery,
    AllQuery,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryModifier {
    Must,
    Should,
    MustNot,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GoQuery {
    BoolQuery {
        subqueries: Vec<QueryElement>,
        boost: f32,
    },
    PhraseQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    PhrasePrefixQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    TermPrefixQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    TermQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    EveryTermQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    OneOfTermQuery {
        field_index: usize,
        text_index: usize,
        boost: f32,
    },
    AllQuery {
        boost: f32,
    },
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct QueryElement {
    pub query: Option<GoQuery>,
    pub modifier: QueryModifier,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BoolQuery {
    pub subqueries: Vec<QueryElement>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FinalQuery {
    pub texts: Vec<String>,
    pub fields: Vec<String>,
    pub query: BoolQuery,
}
