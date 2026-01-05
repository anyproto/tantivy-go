use crate::queries::models::{GoQuery, QueryElement, QueryModifier};
use crate::queries::QueryType;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl QueryType {
    fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(QueryType::BoolQuery),
            1 => Some(QueryType::PhraseQuery),
            2 => Some(QueryType::PhrasePrefixQuery),
            3 => Some(QueryType::TermPrefixQuery),
            4 => Some(QueryType::TermQuery),
            5 => Some(QueryType::EveryTermQuery),
            6 => Some(QueryType::OneOfTermQuery),
            7 => Some(QueryType::AllQuery),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for QueryType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryTypeVisitor;

        impl<'de> Visitor<'de> for QueryTypeVisitor {
            type Value = QueryType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number representing the QueryType")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                QueryType::from_u64(value)
                    .ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(value), &self))
            }
        }

        deserializer.deserialize_u64(QueryTypeVisitor)
    }
}

impl QueryModifier {
    fn from_u64(val: u64) -> Option<Self> {
        match val {
            0 => Some(QueryModifier::Must),
            1 => Some(QueryModifier::Should),
            2 => Some(QueryModifier::MustNot),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for QueryModifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryModifierVisitor;

        impl<'de> Visitor<'de> for QueryModifierVisitor {
            type Value = QueryModifier;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number representing the QueryType")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                QueryModifier::from_u64(value)
                    .ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(value), &self))
            }
        }

        deserializer.deserialize_u64(QueryModifierVisitor)
    }
}

fn extract_query_data <'a, D : Deserializer<'a>>(
    map: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, D::Error> {
    map.get("query")
        .and_then(|q| q.as_object())
        .ok_or_else(|| de::Error::missing_field("query"))
}

impl<'de> Deserialize<'de> for QueryElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        let modifier = map
            .get("query_modifier")
            .ok_or_else(|| serde::de::Error::missing_field("query_modifier"))?
            .as_u64()
            .and_then(QueryModifier::from_u64)
            .ok_or_else(|| serde::de::Error::custom("Invalid query_modifier"))?;

        let query_type = map
            .get("query_type")
            .ok_or_else(|| serde::de::Error::missing_field("query_type"))?
            .as_u64()
            .and_then(QueryType::from_u64)
            .ok_or_else(|| serde::de::Error::custom("Invalid query_type"))?;

        fn extract_query_indices_and_boost(
            query_data: &serde_json::Map<String, serde_json::Value>,
        ) -> (usize, usize, f32) {
            let field_index = query_data
                .get("field_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let text_index = query_data
                .get("text_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let boost = query_data
                .get("boost")
                .and_then(|v| v.as_f64().map(|f| f as f32))
                .unwrap_or(1.0);

            (field_index, text_index, boost)
        }

        let query = match query_type {
            QueryType::BoolQuery => {
                let subqueries = map
                    .get("query")
                    .and_then(|q| q.get("subqueries"))
                    .ok_or_else(|| serde::de::Error::missing_field("subqueries"))?;
                let boost = map
                    .get("query")
                    .and_then(|q| q.get("boost"))
                    .and_then(|v| v.as_f64().map(|f| f as f32))
                    .ok_or_else(|| serde::de::Error::missing_field("boost"))?;
                Some(GoQuery::BoolQuery {
                    subqueries: serde_json::from_value(subqueries.clone())
                        .map_err(serde::de::Error::custom)?,
                    boost,
                })
            }
            QueryType::PhraseQuery | QueryType::PhrasePrefixQuery | QueryType::TermPrefixQuery
            | QueryType::TermQuery | QueryType::EveryTermQuery | QueryType::OneOfTermQuery => {
                let query_data = extract_query_data::<D>(&map)?;
                let (field_index, text_index, boost) = extract_query_indices_and_boost(query_data);

                Some(match query_type {
                    QueryType::PhraseQuery => GoQuery::PhraseQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    QueryType::PhrasePrefixQuery => GoQuery::PhrasePrefixQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    QueryType::TermPrefixQuery => GoQuery::TermPrefixQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    QueryType::TermQuery => GoQuery::TermQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    QueryType::EveryTermQuery => GoQuery::EveryTermQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    QueryType::OneOfTermQuery => GoQuery::OneOfTermQuery {
                        field_index,
                        text_index,
                        boost,
                    },
                    _ => return Err(de::Error::custom("Unknown query type")),
                })
            }
            QueryType::AllQuery => {
                let query_data = extract_query_data::<D>(&map)?;
                let boost = query_data
                    .get("boost")
                    .and_then(|v| v.as_f64().map(|f| f as f32))
                    .unwrap_or(1.0);
                Some(GoQuery::AllQuery { boost })
            }
        };

        Ok(QueryElement { query, modifier })
    }
}