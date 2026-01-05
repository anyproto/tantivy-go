use crate::queries::{FinalQuery, GoQuery, QueryElement, QueryModifier};
use crate::tantivy_util::{extract_terms, TantivyGoError};
use tantivy::query::Occur::{Must, Should};
use tantivy::query::{
    AllQuery as TAllQuery, BooleanQuery, BoostQuery, Occur, PhrasePrefixQuery, PhraseQuery, Query,
    TermQuery,
};
use tantivy::schema::{IndexRecordOption, Schema};
use tantivy::{Index, Score};

fn contains_all_query(subqueries: &[QueryElement]) -> bool {
    subqueries.iter().any(|elem| match &elem.query {
        Some(GoQuery::AllQuery { .. }) => true,
        Some(GoQuery::BoolQuery { subqueries, .. }) => contains_all_query(subqueries),
        _ => false,
    })
}

pub fn convert_to_tantivy(
    index: &Index,
    parsed: FinalQuery,
    schema: &Schema,
) -> Result<Box<dyn Query>, TantivyGoError> {
    let has_all_query = contains_all_query(&parsed.query.subqueries);
    if !has_all_query && (parsed.fields.is_empty() || parsed.texts.is_empty()) {
        return Err(TantivyGoError(
            "Fields or texts cannot be empty".to_string(),
        ));
    }

    /// Convert a QueryElement into an optional Tantivy Query.
    /// Returns Ok(None) if the clause yields zero terms and is not Must.
    /// Returns a special "match nothing" query if the clause is Must and yields zero terms.
    fn element_to_query(
        index: &Index,
        element: &QueryElement,
        schema: &Schema,
        texts: &[String],
        fields: &[String],
    ) -> Result<Option<(Occur, Box<dyn Query>)>, TantivyGoError> {
        let occur = match element.modifier {
            QueryModifier::Must => Occur::Must,
            QueryModifier::Should => Occur::Should,
            QueryModifier::MustNot => Occur::MustNot,
        };

        let get_field_and_text = |fi: usize, ti: usize| -> Result<(_, _), TantivyGoError> {
            let f_name = fields.get(fi)
                .ok_or_else(|| TantivyGoError("Invalid field index".into()))?;
            let txt = texts.get(ti)
                .ok_or_else(|| TantivyGoError("Invalid text index".into()))?;
            let f = schema.get_field(f_name)
                .map_err(|_e| TantivyGoError("Invalid field name".into()))?;
            Ok((f, txt.as_str()))
        };

        // Wrap extract_terms: handle zero-term errors differently for Must vs Should/MustNot
        let get_terms = |f, txt| -> Result<Vec<(usize, tantivy::Term)>, TantivyGoError> {
            match extract_terms(index, f, txt) {
                Ok(v) => Ok(v),
                Err(ref e) if e.0.contains("Zero terms were extracted") => Ok(Vec::new()),
                Err(e) => Err(e),
            }
        };

        // Create a query that matches no documents - for empty Must clauses
        let create_impossible_query = || -> Box<dyn Query> {
            // Create a term that won't match any document
            let term = tantivy::Term::from_field_text(schema.get_field("_id").unwrap_or_else(|_| {
                // If no _id field exists, use the first field from the schema
                if let Some(field) = schema.fields().next() {
                    field.0
                } else {
                    // This should never happen as schemas should have at least one field
                    panic!("Schema has no fields");
                }
            }), "__impossible_term_that_wont_match_anything__");
            
            Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs))
        };

        if let Some(go_q) = &element.query {
            let built = match go_q {
                GoQuery::PhraseQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        // For empty Must clauses, return a query that matches nothing
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None); // Discard empty Should/MustNot clauses
                        }
                    } else if terms.len() == 1 {
                        Some(try_boost(occur, *boost, Box::new(TermQuery::new(
                            terms[0].1.clone(),
                            IndexRecordOption::WithFreqsAndPositions,
                        ))))
                    } else {
                        Some(try_boost(occur, *boost, Box::new(PhraseQuery::new_with_offset(terms))))
                    }
                }
                GoQuery::PhrasePrefixQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        Some(try_boost(occur, *boost, Box::new(PhrasePrefixQuery::new_with_offset(terms))))
                    }
                }
                GoQuery::TermQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        Some(try_boost(occur, *boost, Box::new(TermQuery::new(
                            terms[0].1.clone(), IndexRecordOption::WithFreqs,
                        ))))
                    }
                }
                GoQuery::TermPrefixQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        Some(try_boost(occur, *boost,
                            Box::new(PhrasePrefixQuery::new(vec![terms[0].1.clone()]))))
                    }
                }
                GoQuery::EveryTermQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        let mut subs = Vec::new();
                        for (_pos, term) in terms {
                            subs.push(try_boost(Must, 1.0,
                                Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs))));
                        }
                        Some(try_boost(occur, *boost, Box::new(BooleanQuery::new(subs))))
                    }
                }
                GoQuery::OneOfTermQuery { field_index, text_index, boost } => {
                    let (f, txt) = get_field_and_text(*field_index, *text_index)?;
                    let terms = get_terms(f, txt)?;
                    if terms.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        let mut subs = Vec::new();
                        let len = terms.len() as f32;
                        for (i, (_pos, term)) in terms.into_iter().enumerate() {
                            let weight = 1.0 - 0.5 * ((i + 1) as f32 / len);
                            subs.push(try_boost(Should, weight,
                                Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs))));
                        }
                        Some(try_boost(occur, *boost, Box::new(BooleanQuery::new(subs))))
                    }
                }
                GoQuery::AllQuery { boost } => {
                    Some(try_boost(occur, *boost, Box::new(TAllQuery)))
                }
                GoQuery::BoolQuery { subqueries, boost } => {
                    let mut child = Vec::new();
                    for sq in subqueries {
                        if let Some(q) = element_to_query(index, sq, schema, texts, fields)? {
                            child.push(q);
                        }
                    }
                    if child.is_empty() {
                        if element.modifier == QueryModifier::Must {
                            Some(try_boost(occur, *boost, create_impossible_query()))
                        } else {
                            return Ok(None);
                        }
                    } else {
                        Some(try_boost(occur, *boost, Box::new(BooleanQuery::new(child))))
                    }
                }
            };
            return Ok(built);
        }

        Err(TantivyGoError("Query is None in QueryElement".into()))
    }

    // wrap boost
    fn try_boost(occur: Occur, boost: f32, q: Box<dyn Query>) -> (Occur, Box<dyn Query>) {
        if (boost - 1.0).abs() < std::f32::EPSILON {
            (occur, q)
        } else {
            (occur, Box::new(BoostQuery::new(q, boost as Score)))
        }
    }

    // === Top-level ===
    let mut top = Vec::new();
    for elem in &parsed.query.subqueries {
        if let Some(q) = element_to_query(index, elem, schema, &parsed.texts, &parsed.fields)? {
            top.push(q);
        }
    }
    if top.is_empty() {
        return Err(TantivyGoError("No usable clauses in query".into()));
    }
    Ok(Box::new(BooleanQuery::from(top)))
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
    fn test_convert_drops_empty_text_clause() {
        // Build schema with one field using "simple" tokenizer
        let mut schema_b = Schema::builder();
        let mut opts = TEXT | STORED;
        opts = opts.set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("simple")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        let f1 = schema_b.add_text_field("f1", opts.clone());
        let f2 = schema_b.add_text_field("f2", opts.clone());
        let f3 = schema_b.add_text_field("f3", opts.clone());
        let f4 = schema_b.add_text_field("f4", opts);
        let schema = schema_b.build();

        // Create index and register tokenizer
        let index = Index::create_in_ram(schema.clone());
        index.tokenizers().register(
            "simple",
            TextAnalyzer::builder(SimpleTokenizer::default()).build(),
        );

        // Construct FinalQuery with four clauses: three empty with different modifiers, one valid
        let fq = FinalQuery {
            texts: vec!["".into(), "".into(), "".into(), "hello".into()],
            fields: vec!["f1".into(), "f2".into(), "f3".into(), "f4".into()],
            query: BoolQuery { subqueries: vec![
                QueryElement { query: Some(GoQuery::PhraseQuery { field_index: 0, text_index: 0, boost: 1.0 }), modifier: QueryModifier::Must },
                QueryElement { query: Some(GoQuery::PhraseQuery { field_index: 1, text_index: 1, boost: 1.0 }), modifier: QueryModifier::MustNot },
                QueryElement { query: Some(GoQuery::PhraseQuery { field_index: 2, text_index: 2, boost: 1.0 }), modifier: QueryModifier::Should },
                QueryElement { query: Some(GoQuery::PhraseQuery { field_index: 3, text_index: 3, boost: 1.0 }), modifier: QueryModifier::Must },
            ]},
        };

        // Convert: should succeed and keep empty Must clauses as impossible queries, dropping Should/MustNot
        let q = convert_to_tantivy(&index, fq, &schema).expect("conversion failed");
        let dq = q.as_any().downcast_ref::<BooleanQuery>()
            .expect("expected BooleanQuery");
        let subs = dq.clauses();
        
        // We should have 2 clauses: one impossible query for the empty Must clause, and one for "hello"
        assert_eq!(subs.len(), 2);
        
        // Both should have Must occur
        assert_eq!(subs[0].0, TO::Must);
        assert_eq!(subs[1].0, TO::Must);
        
        let debug = format!("{:?}", dq);
        assert!(debug.contains("hello"));
        // The impossible query term should also be in the debug output
        assert!(debug.contains("__impossible_term_that_wont_match_anything__"));
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