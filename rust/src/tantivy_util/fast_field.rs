use std::collections::HashMap;
use tantivy::columnar::StrColumn;
use tantivy::schema::{Field, Schema};
use tantivy::{DocAddress, Searcher};

use crate::tantivy_util::TantivyGoError;

/// Reads fast field values for doc addresses, grouped by segment for efficiency.
pub fn read_fast_field_values(
    searcher: &Searcher,
    schema: &Schema,
    field: Field,
    doc_addresses: &[DocAddress],
) -> Result<Vec<Option<String>>, TantivyGoError> {
    if doc_addresses.is_empty() {
        return Ok(vec![]);
    }

    let field_name = schema.get_field_name(field);

    let mut segment_groups: HashMap<u32, Vec<(usize, u32)>> = HashMap::new();
    for (idx, addr) in doc_addresses.iter().enumerate() {
        segment_groups
            .entry(addr.segment_ord)
            .or_default()
            .push((idx, addr.doc_id));
    }

    let mut results: Vec<Option<String>> = vec![None; doc_addresses.len()];
    let mut buffer = String::new();

    for (segment_ord, docs) in segment_groups {
        let segment_reader = searcher.segment_reader(segment_ord);
        let fast_fields = segment_reader.fast_fields();

        let str_column: StrColumn = fast_fields
            .str(field_name)
            .map_err(|e| TantivyGoError::from_err(
                &format!("Failed to get fast field '{}'", field_name),
                &e.to_string(),
            ))?
            .ok_or_else(|| TantivyGoError(
                format!("Fast field '{}' not found in segment {}", field_name, segment_ord),
            ))?;

        for (result_idx, doc_id) in docs {
            buffer.clear();
            if let Some(ord) = str_column.term_ords(doc_id).next() {
                if str_column.ord_to_str(ord, &mut buffer).is_ok() && !buffer.is_empty() {
                    results[result_idx] = Some(buffer.clone());
                }
            }
        }
    }

    Ok(results)
}
