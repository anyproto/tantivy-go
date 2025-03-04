use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
use crate::tantivy_util::{extract_terms, TantivyGoError};
use tantivy::query::Occur::{Must, Should};
use tantivy::query::{
    BooleanQuery, BoostQuery, Occur, PhrasePrefixQuery, PhraseQuery, Query, TermQuery,
};
use tantivy::schema::{IndexRecordOption, Schema};
use tantivy::{Index, Score};

fn convert_to_tantivy(
    index: &Index,
    parsed: FinalQuery,
    schema: &Schema,
) -> Result<Box<dyn Query>, TantivyGoError> {
    if parsed.fields.is_empty() || parsed.texts.is_empty() {
        return Err(TantivyGoError(
            "Fields or texts cannot be empty".to_string(),
        ));
    }

    // Recursive function to convert `QueryElement` to Tantivy's queries
    fn element_to_query(
        index: &Index,
        element: &QueryElement,
        schema: &Schema,
        texts: &[String],
        fields: &[String],
    ) -> Result<(Occur, Box<dyn Query>), TantivyGoError> {
        let occur = modifier_to_occur(&element.modifier);

        let process_field_and_text =
            |field_index: usize, text_index: usize| -> Result<(_, _), TantivyGoError> {
                let field = fields
                    .get(field_index)
                    .ok_or(TantivyGoError("Invalid field index".to_string()))?;
                let text = texts
                    .get(text_index)
                    .ok_or(TantivyGoError("Invalid text index".to_string()))?;
                let field = schema
                    .get_field(field)
                    .or(Err(TantivyGoError("Invalid field name".to_string())))?;
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
                    if terms.len() == 1 {
                        Ok(try_boost(
                            occur,
                            *boost,
                            Box::new(TermQuery::new(
                                terms[0].1.clone(),
                                IndexRecordOption::WithFreqsAndPositions,
                            )),
                        ))
                    } else {
                        Ok(try_boost(
                            occur,
                            *boost,
                            Box::new(PhraseQuery::new_with_offset(terms)),
                        ))
                    }
                }

                GoQuery::PhrasePrefixQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(PhrasePrefixQuery::new_with_offset(terms)),
                    ))
                }

                GoQuery::TermPrefixQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(PhrasePrefixQuery::new(vec![terms[0].1.clone()])),
                    ))
                }

                GoQuery::TermQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(TermQuery::new(
                            terms[0].1.clone(),
                            IndexRecordOption::WithFreqs,
                        )),
                    ))
                }

                GoQuery::EveryTermQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    let mut post_terms = vec![];
                    for (_, term) in terms.iter().enumerate() {
                        let result = try_boost(
                            Must,
                            1.0,
                            Box::new(TermQuery::new(term.1.clone(), IndexRecordOption::WithFreqs)),
                        );
                        post_terms.push(result);
                    }

                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(BooleanQuery::new(post_terms)),
                    ))
                }

                GoQuery::OneOfTermQuery {
                    field_index,
                    text_index,
                    boost,
                } => {
                    let (field, text) = process_field_and_text(*field_index, *text_index)?;
                    let terms = extract_terms(&index, field, text)?;
                    let mut post_terms = vec![];
                    for (i, term) in terms.iter().enumerate() {
                        let result = try_boost(
                            Should,
                            1.0 - 0.5f32 * (i + 1) as f32 / terms.len() as f32,
                            Box::new(TermQuery::new(term.1.clone(), IndexRecordOption::WithFreqs)),
                        );
                        post_terms.push(result);
                    }

                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(BooleanQuery::new(post_terms)),
                    ))
                }

                GoQuery::BoolQuery { subqueries, boost } => {
                    let mut sub_queries = vec![];
                    for subquery in subqueries {
                        sub_queries.push(element_to_query(index, subquery, schema, texts, fields)?);
                    }
                    Ok(try_boost(
                        occur,
                        *boost,
                        Box::new(BooleanQuery::new(sub_queries)),
                    ))
                }
            }
        } else {
            Err(TantivyGoError("Query is None in QueryElement".to_string()))
        }
    }

    fn try_boost(occur: Occur, boost: f32, query: Box<dyn Query>) -> (Occur, Box<dyn Query>) {
        if boost == 1.0 {
            (occur, query)
        } else {
            (occur, Box::new(BoostQuery::new(query, boost as Score)))
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
) -> Result<Box<dyn Query>, TantivyGoError> {
    match serde_json::from_str(json) {
        Ok(parsed) => convert_to_tantivy(index, parsed, schema),
        Err(e) => Err(TantivyGoError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::convert::convert_to_tantivy;
    use crate::queries::models::BoolQuery;
    use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
    use std::fs;
    use tantivy::query::BooleanQuery;
    use tantivy::query::PhraseQuery as TPhraseQuery;
    use tantivy::query::TermQuery as TTermQuery;
    use tantivy::query::{BoostQuery, Occur as TO};
    use tantivy::query::{PhrasePrefixQuery as TPhrasePrefixQuery, Query};
    use tantivy::schema::{Field, IndexRecordOption, Schema, TextFieldIndexing, STORED, TEXT};
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer};
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
                "sample three words",
                "one",
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
                        query: Some(GoQuery::TermPrefixQuery {
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
                        query: Some(GoQuery::TermPrefixQuery {
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
                                        subqueries: Vec::from([
                                            QueryElement {
                                                query: Some(GoQuery::PhraseQuery {
                                                    field_index: 7,
                                                    text_index: 5,
                                                    boost: 0.8,
                                                }),
                                                modifier: QueryModifier::Must,
                                            },
                                            QueryElement {
                                                query: Some(GoQuery::EveryTermQuery {
                                                    field_index: 0,
                                                    text_index: 6,
                                                    boost: 0.4,
                                                }),
                                                modifier: QueryModifier::MustNot,
                                            },
                                        ]),
                                        boost: 0.3f32,
                                    }),
                                    modifier: QueryModifier::Should,
                                },
                            ]),
                            boost: 1f32,
                        }),
                        modifier: QueryModifier::Must,
                    },
                    QueryElement {
                        query: Some(GoQuery::OneOfTermQuery {
                            field_index: 1,
                            text_index: 6,
                            boost: 1f32,
                        }),
                        modifier: QueryModifier::Must,
                    },
                    QueryElement {
                        query: Some(GoQuery::PhraseQuery {
                            field_index: 1,
                            text_index: 7,
                            boost: 1f32,
                        }),
                        modifier: QueryModifier::Must,
                    },
                    QueryElement {
                        query: Some(GoQuery::TermQuery {
                            field_index: 1,
                            text_index: 7,
                            boost: 1f32,
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

        assert_eq!(parsed, expected);
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
                        Box::new(BoostQuery::new(
                            Box::new(BooleanQuery::new(vec![
                                (
                                    TO::Must,
                                    boost_query(
                                        phrase_query(comments, vec!["not", "single", "term"]),
                                        0.8,
                                    ),
                                ),
                                (
                                    TO::MustNot,
                                    Box::new(BoostQuery::new(
                                        Box::new(BooleanQuery::new(vec![
                                            (
                                                TO::Must,
                                                Box::new(TTermQuery::new(
                                                    Term::from_field_text(body1, "sample"),
                                                    IndexRecordOption::WithFreqs,
                                                )),
                                            ),
                                            (
                                                TO::Must,
                                                Box::new(TTermQuery::new(
                                                    Term::from_field_text(body1, "three"),
                                                    IndexRecordOption::WithFreqs,
                                                )),
                                            ),
                                            (
                                                TO::Must,
                                                Box::new(TTermQuery::new(
                                                    Term::from_field_text(body1, "words"),
                                                    IndexRecordOption::WithFreqs,
                                                )),
                                            ),
                                        ])),
                                        0.4f32,
                                    )),
                                ),
                            ])),
                            0.3f32,
                        )),
                    ),
                ])),
            ),
            (
                TO::Must,
                Box::new(BooleanQuery::new(vec![
                    (
                        TO::Should,
                        Box::new(BoostQuery::new(
                            Box::new(TTermQuery::new(
                                Term::from_field_text(body2, "sample"),
                                IndexRecordOption::WithFreqs,
                            )),
                            0.8333333f32,
                        )),
                    ),
                    (
                        TO::Should,
                        Box::new(BoostQuery::new(
                            Box::new(TTermQuery::new(
                                Term::from_field_text(body2, "three"),
                                IndexRecordOption::WithFreqs,
                            )),
                            0.6666666f32,
                        )),
                    ),
                    (
                        TO::Should,
                        Box::new(BoostQuery::new(
                            Box::new(TTermQuery::new(
                                Term::from_field_text(body2, "words"),
                                IndexRecordOption::WithFreqs,
                            )),
                            0.5f32,
                        )),
                    ),
                ])),
            ),
            (
                TO::Must,
                Box::new(TTermQuery::new(
                    Term::from_field_text(body2, "one"),
                    IndexRecordOption::WithFreqs,
                )),
            ),
            (
                TO::Must,
                Box::new(TTermQuery::new(
                    Term::from_field_text(body2, "one"),
                    IndexRecordOption::WithFreqs,
                )),
            ),
        ]);

        assert_eq!(format!("{parsed:#?}"), format!("{expected:#?}"));
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
