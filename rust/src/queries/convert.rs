use crate::queries::models::BoolQuery;
use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
use crate::tantivy_util::extract_terms;
use std::error::Error;
use std::{env, fmt, fs};
use tantivy::query::{BooleanQuery, BoostQuery, Occur, PhrasePrefixQuery, PhraseQuery, Query};
use tantivy::schema::Schema;
use tantivy::{Index, Score};

fn convert_to_tantivy(
    index: &Index,
    parsed: FinalQuery,
    schema: &Schema,
) -> Result<Box<dyn Query>, Box<dyn Error>> {
    if parsed.fields.is_empty() || parsed.texts.is_empty() {
        return Err("Fields or texts cannot be empty".into());
    }

    // Recursive function to convert `QueryElement` to Tantivy's queries
    fn element_to_query(
        index: &Index,
        element: &QueryElement,
        schema: &Schema,
        texts: &[String],
        fields: &[String],
    ) -> Result<(Occur, Box<dyn Query>), Box<dyn Error>> {
        let occur = modifier_to_occur(&element.modifier);

        let process_field_and_text =
            |field_index: usize, text_index: usize| -> Result<(_, _), Box<dyn Error>> {
                let field = fields.get(field_index).ok_or("Invalid field index")?;
                let text = texts.get(text_index).ok_or("Invalid text index")?;
                let field = schema.get_field(field).or(Err("Invalid field name"))?;
                Ok((field, text))
            };

        if let Some(go_query) = &element.query {
            match go_query {
                GoQuery::PhraseQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    if terms.len() <= 1 {
                        return Err(Box::<dyn Error>::from("Phrase must have more than one term"));
                    }
                    try_boost(occur, *boost, Box::new(PhraseQuery::new(terms)))
                }

                GoQuery::PhrasePrefixQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    try_boost(occur, *boost, Box::new(PhrasePrefixQuery::new(terms)))
                }

                GoQuery::SingleTermPrefixQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    try_boost(
                        occur,
                        *boost,
                        Box::new(PhrasePrefixQuery::new(vec![terms[0].clone()])),
                    )
                }

                GoQuery::BoolQuery { subqueries } => {
                    let mut sub_queries = vec![];
                    for subquery in subqueries {
                        sub_queries.push(element_to_query(index, subquery, schema, texts, fields)?);
                    }
                    let bool_query = BooleanQuery::from(sub_queries);
                    Ok((occur, Box::new(bool_query)))
                }

                _ => Err("Unsupported GoQuery variant".into()),
            }
        } else {
            Err("Query is None in QueryElement".into())
        }
    }

    fn try_boost(
        occur: Occur,
        boost: f32,
        query: Box<dyn Query>,
    ) -> Result<(Occur, Box<dyn Query>), Box<dyn Error>> {
        if boost == 1.0 {
            Ok((occur, query))
        } else {
            Ok((occur, Box::new(BoostQuery::new(query, boost as Score))))
        }
    }

    let mut sub_queries = vec![];
    for subquery in &parsed.query.subqueries {
        sub_queries.push(element_to_query(
            index,
            subquery,
            schema,
            &parsed.texts,
            &parsed.fields,
        )?);
    }

    let bool_query = BooleanQuery::from(sub_queries);
    Ok(Box::new(bool_query))
}

// Convert your `QueryModifier` to Tantivy's `Occur`
fn modifier_to_occur(modifier: &QueryModifier) -> Occur {
    match modifier {
        QueryModifier::Must => Occur::Must,
        QueryModifier::Should => Occur::Should,
        QueryModifier::MustNot => Occur::MustNot,
    }
}

pub fn parse_query_from_json(
    index: &Index,
    schema: &Schema,
    json: &str,
) -> Result<Box<dyn Query>, Box<dyn Error>> {
    let parsed = serde_json::from_str(json)?;
    convert_to_tantivy(index, parsed, schema)
}

mod for_tests {
    use crate::queries::GoQuery::BoolQuery;
    use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
}

#[cfg(test)]
mod tests {
    use crate::queries::convert::convert_to_tantivy;
    use crate::queries::models::BoolQuery;
    use crate::queries::QueryModifier::Must;
    use crate::queries::QueryType::PhraseQuery;
    use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
    use std::fs;
    use tantivy::query::PhraseQuery as TPhraseQuery;
    use tantivy::query::{BooleanQuery, PhrasePrefixQuery};
    use tantivy::query::{BoostQuery, Occur as TO};
    use tantivy::query::{PhrasePrefixQuery as TPhrasePrefixQuery, Query};
    use tantivy::schema::{Field, IndexRecordOption, Schema, TextFieldIndexing, STORED, TEXT};
    use tantivy::tokenizer::{
        AsciiFoldingFilter, Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer,
        TextAnalyzer,
    };
    use tantivy::{Index, Term};

    fn expected_query() -> FinalQuery {
        FinalQuery {
            texts: vec![
                "some words",
                "term",
                "another term",
                "term2",
                "term3",
                "not single term",
            ]
            .into_iter()
            .map(|t| t.to_string())
            .collect(),
            fields: vec![
                "body1", "body2", "body3", "title1", "title2", "title3", "summary", "comments",
            ]
            .into_iter()
            .map(|t| t.to_string())
            .collect(),
            query: BoolQuery {
                subqueries: Vec::from([
                    QueryElement {
                        query: Some(GoQuery::PhraseQuery {
                            field_index: 0,
                            text_index: 0,
                            boost: 1.0,
                        }),
                        modifier: QueryModifier::Must,
                    },
                    QueryElement {
                        query: Some(GoQuery::PhrasePrefixQuery {
                            field_index: 1,
                            text_index: 1,
                            boost: 1.0,
                        }),
                        modifier: QueryModifier::Should,
                    },
                    QueryElement {
                        query: Some(GoQuery::SingleTermPrefixQuery {
                            field_index: 2,
                            text_index: 1,
                            boost: 1.0,
                        }),
                        modifier: QueryModifier::MustNot,
                    },
                    QueryElement {
                        query: Some(GoQuery::PhraseQuery {
                            field_index: 3,
                            text_index: 2,
                            boost: 0.1,
                        }),
                        modifier: QueryModifier::Must,
                    },
                    QueryElement {
                        query: Some(GoQuery::PhrasePrefixQuery {
                            field_index: 4,
                            text_index: 3,
                            boost: 0.1,
                        }),
                        modifier: QueryModifier::Should,
                    },
                    QueryElement {
                        query: Some(GoQuery::SingleTermPrefixQuery {
                            field_index: 5,
                            text_index: 3,
                            boost: 0.1,
                        }),
                        modifier: QueryModifier::MustNot,
                    },
                    QueryElement {
                        query: Some(GoQuery::BoolQuery {
                            subqueries: Vec::from([
                                QueryElement {
                                    query: Some(GoQuery::PhrasePrefixQuery {
                                        field_index: 6,
                                        text_index: 4,
                                        boost: 1.0,
                                    }),
                                    modifier: QueryModifier::Should,
                                },
                                QueryElement {
                                    query: Some(GoQuery::BoolQuery {
                                        subqueries: Vec::from([QueryElement {
                                            query: Some(GoQuery::PhraseQuery {
                                                field_index: 7,
                                                text_index: 5,
                                                boost: 0.8,
                                            }),
                                            modifier: QueryModifier::Must,
                                        }]),
                                    }),
                                    modifier: QueryModifier::Should,
                                },
                            ]),
                        }),
                        modifier: QueryModifier::Must,
                    },
                ]),
            },
        }
    }

    #[test]
    fn test_file_reading() {
        let file_path = "../test_jsons/data.json";
        let contents = fs::read_to_string(file_path).expect("Failed to read file");

        let expected: FinalQuery = expected_query();
        let parsed: FinalQuery = serde_json::from_str(&contents).expect("Json was not parsed");

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_convert() {
        let given_query: FinalQuery = expected_query();
        let text_analyzer_simple = TextAnalyzer::builder(SimpleTokenizer::default()).build();

        let mut text_options = TEXT;
        text_options = text_options | STORED;
        text_options = text_options.set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("simple")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );

        let mut schema_builder = Schema::builder();
        let body1 = schema_builder.add_text_field("body1", text_options.clone()); // 1
        let body2 = schema_builder.add_text_field("body2", text_options.clone()); // 2
        let body3 = schema_builder.add_text_field("body3", text_options.clone()); // 3
        let title1 = schema_builder.add_text_field("title1", text_options.clone()); // 4
        let title2 = schema_builder.add_text_field("title2", text_options.clone()); // 5
        let title3 = schema_builder.add_text_field("title3", text_options.clone()); // 6
        let summary = schema_builder.add_text_field("summary", text_options.clone()); // 7
        let comments = schema_builder.add_text_field("comments", text_options); // 8
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        index.tokenizers().register("simple", text_analyzer_simple);

        let parsed = convert_to_tantivy(&index, given_query, &schema).expect("can't convert");
        let expected = BooleanQuery::new(vec![
            (TO::Must, phrase_query(body1, vec!["some", "words"])),
            (TO::Should, phrase_prefix_query(body2, vec!["term"])),
            (TO::MustNot, phrase_prefix_query(body3, vec!["term"])),
            (
                TO::Must,
                boost_query(phrase_query(title1, vec!["another", "term"]), 0.1),
            ),
            (
                TO::Should,
                boost_query(phrase_prefix_query(title2, vec!["term2"]), 0.1),
            ),
            (
                TO::MustNot,
                boost_query(phrase_prefix_query(title3, vec!["term2"]), 0.1),
            ),
            (
                TO::Must,
                Box::new(BooleanQuery::new(vec![
                    (TO::Should, phrase_prefix_query(summary, vec!["term3"])),
                    (
                        TO::Should,
                        Box::new(BooleanQuery::new(vec![(
                            TO::Must,
                            boost_query(phrase_query(comments, vec!["not", "single", "term"]), 0.8),
                        )])),
                    ),
                ])),
            ),
        ]);

        assert_eq!(format!("{expected:#?}"), format!("{parsed:#?}"));
    }

    fn make_terms(field: Field, words: Vec<&str>) -> Vec<Term> {
        words
            .into_iter()
            .map(|w| Term::from_field_text(field, w))
            .collect()
    }

    fn phrase_query(field: Field, words: Vec<&str>) -> Box<TPhraseQuery> {
        Box::new(TPhraseQuery::new(make_terms(field, words)))
    }

    fn phrase_prefix_query(field: Field, words: Vec<&str>) -> Box<TPhrasePrefixQuery> {
        Box::new(TPhrasePrefixQuery::new(make_terms(field, words)))
    }

    fn boost_query(query: Box<dyn Query>, boost: f32) -> Box<BoostQuery> {
        Box::new(BoostQuery::new(query, boost))
    }
}
