// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Pagination and dataset-statistics query builders.
//!
//! These operate on already-constructed query strings — they don't know about
//! the internal structure of the base query beyond finding the `SELECT`
//! keyword.

/// Generate a dataset statistics query from a base compound query.
///
/// **Metrics:**
/// - `n_entries`: total result triples (including duplicates)
/// - `n_entries_unique`: unique compound-taxon-reference combinations
/// - `n_compounds`: distinct compounds (Wikidata entities)
/// - `n_taxa`: distinct organisms
/// - `n_references`: distinct evidence sources
///
/// Uses `COUNT(DISTINCT …)` to compute cardinality without materializing
/// full result sets.  The base query's WHERE block is preserved, so all
/// filtering / search logic is retained.
#[must_use]
pub fn query_counts_from_base(base_query: &str) -> String {
    let Some(select_pos) = base_query.find("SELECT") else {
        return base_query.to_string();
    };
    let prefixes = &base_query[..select_pos];
    let inner_select = base_query[select_pos..].trim();

    format!(
        r#"{prefixes}
SELECT
  (COUNT(*) AS ?n_entries)
  (COUNT(DISTINCT CONCAT(
    STR(?compound), "\u001F", COALESCE(STR(?taxon), ""), "\u001F", COALESCE(STR(?ref_qid), "")
  )) AS ?n_entries_unique)
  (COUNT(DISTINCT ?compound) AS ?n_compounds)
  (COUNT(DISTINCT ?taxon) AS ?n_taxa)
  (COUNT(DISTINCT ?ref_qid) AS ?n_references)
WHERE {{
  {{
    {inner_select}
  }}
}}"#
    )
}

/// Append a `LIMIT` clause to a base query for pagination or sampling.
///
/// # Use Cases
///
/// - Pagination: fetch first N results, then apply `OFFSET` for next page
/// - Sampling: `LIMIT 100` for quick exploratory queries
/// - UI constraints: avoid overwhelming clients with massive result sets
#[must_use]
pub fn query_with_limit(base_query: &str, limit: usize) -> String {
    let trimmed = base_query.trim_end();
    format!("{trimmed}\nLIMIT {limit}")
}
