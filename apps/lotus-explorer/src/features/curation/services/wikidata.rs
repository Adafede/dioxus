// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;
use std::collections::{HashMap, HashSet};

pub async fn fetch_wikidata_compound_by_inchikey(
    inchikey: &str,
) -> Result<Option<WikidataCompound>, CurationError> {
    let query = format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?compound ?canonical ?iso ?inchi ?formula ?mass WHERE {{\n  \
           ?compound wdt:P235 \"{}\" .\n  \
           OPTIONAL {{ ?compound wdt:P233 ?canonical . }}\n  \
           OPTIONAL {{ ?compound wdt:P2017 ?iso . }}\n  \
           OPTIONAL {{ ?compound wdt:P234 ?inchi . }}\n  \
           OPTIONAL {{ ?compound wdt:P274 ?formula . }}\n  \
           OPTIONAL {{ ?compound wdt:P2067 ?mass . }}\n\
         }} LIMIT 1",
        escape_sparql_string(inchikey)
    );
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;

    let json =
        serde_json::from_str::<Value>(&raw).map_err(|e| CurationError::Parse(e.to_string()))?;
    let Some(first) = json
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
    else {
        return Ok(None);
    };

    let qid = first
        .get("compound")
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .and_then(extract_qid_from_uri)
        .ok_or_else(|| CurationError::Parse("missing compound qid".to_string()))?;

    Ok(Some(WikidataCompound {
        qid: qid.to_string(),
        canonical_smiles: binding_value(first, "canonical"),
        isomeric_smiles: binding_value(first, "iso"),
        inchi: binding_value(first, "inchi"),
        formula: binding_value(first, "formula"),
        mass: binding_value(first, "mass").and_then(|v| v.parse::<f64>().ok()),
    }))
}

/// Returns (Option<QID>, Vec<creation_QS_lines>).
/// If the taxon exists, returns (Some(qid), []). Otherwise returns (None, <minimal CREATE QS>).
pub async fn resolve_or_create_taxon(
    name: &str,
    pre_resolved_qid: Option<&str>,
) -> Result<(Option<String>, Vec<String>), CurationError> {
    if let Some(qid) = pre_resolved_qid {
        return Ok((Some(qid.to_string()), Vec::new()));
    }

    if let Some(qid) = resolve_taxon_qid(name).await? {
        return Ok((Some(qid), Vec::new()));
    }
    let qs = vec![
        "## -- Step: create missing taxon --".to_string(),
        "CREATE".to_string(),
        format!("LAST|Len|\"{}\"", escape_qs_string(name)),
        format!("LAST|P31|{WD_TAXON_QID}"),
        format!("LAST|P225|\"{}\"", escape_qs_string(name)),
    ];
    Ok((None, qs))
}

pub(super) fn normalize_taxon_lookup(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

fn canonicalize_taxon_label(name: &str) -> String {
    let mut words = name.split_whitespace();
    let Some(first) = words.next() else {
        return String::new();
    };

    let first = {
        let mut chars = first.chars();
        match chars.next() {
            Some(ch) => {
                let mut rebuilt = String::new();
                rebuilt.push(ch.to_ascii_uppercase());
                rebuilt.extend(chars.map(|c| c.to_ascii_lowercase()));
                rebuilt
            }
            None => String::new(),
        }
    };

    let mut rebuilt = String::new();
    rebuilt.push_str(&first);
    for word in words {
        rebuilt.push(' ');
        rebuilt.push_str(&word.to_ascii_lowercase());
    }
    rebuilt
}

fn taxon_name_candidates(name: &str) -> Vec<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let canonical = canonicalize_taxon_label(trimmed);
    if canonical == trimmed {
        vec![trimmed.to_string()]
    } else {
        vec![trimmed.to_string(), canonical]
    }
}

fn build_single_taxon_lookup_query(name: &str) -> Option<String> {
    let values = taxon_name_candidates(name)
        .into_iter()
        .map(|candidate| format!("\"{}\"", escape_sparql_string(&candidate)))
        .collect::<Vec<_>>();

    if values.is_empty() {
        return None;
    }

    Some(format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?taxon WHERE {{\n  \
           VALUES ?taxonName {{ {} }}\n  \
           ?taxon wdt:P225 ?taxonName ;\n        \
                  wdt:P31 wd:Q16521 .\n\
         }} LIMIT 1",
        values.join(" ")
    ))
}

fn build_reference_lookup_query(dois: &[String]) -> String {
    let values = dois
        .iter()
        .map(|doi| {
            format!(
                "(\"{}\" \"{}\")",
                escape_sparql_string(doi),
                escape_sparql_string(doi)
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?lookup ?ref WHERE {{\n  \
           VALUES (?lookup ?doi) {{\n    \
             {values}\n  \
           }}\n  \
           ?ref wdt:P356 ?doi .\n\
         }}"
    )
}

pub async fn resolve_reference_qids_batch<'a>(
    dois: impl IntoIterator<Item = &'a str>,
) -> Result<HashMap<String, String>, CurationError> {
    let mut seen = HashSet::new();
    let mut normalized_dois = Vec::new();

    for doi in dois {
        let Some(normalized) = normalize_doi(doi) else {
            continue;
        };
        if seen.insert(normalized.clone()) {
            normalized_dois.push(normalized);
        }
    }

    if normalized_dois.is_empty() {
        return Ok(HashMap::new());
    }

    let query = build_reference_lookup_query(&normalized_dois);
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    let json =
        serde_json::from_str::<Value>(&raw).map_err(|e| CurationError::Parse(e.to_string()))?;

    let mut resolved = HashMap::new();
    for binding in json
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(lookup) = binding_value(binding, "lookup") else {
            continue;
        };
        let Some(qid) = binding
            .get("ref")
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
            .and_then(extract_qid_from_uri)
        else {
            continue;
        };
        resolved.insert(lookup, qid.to_string());
    }

    Ok(resolved)
}

fn build_taxon_lookup_query(lookups: &[(String, String)]) -> String {
    let mut values = String::new();
    for (i, (lookup, taxon_name)) in lookups.iter().enumerate() {
        if i > 0 {
            values.push_str("\n    ");
        }
        values.push('(');
        values.push('"');
        values.push_str(&escape_sparql_string(lookup));
        values.push_str("\" \"");
        values.push_str(&escape_sparql_string(taxon_name));
        values.push_str("\")");
    }

    format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?lookup ?taxon WHERE {{\n  \
           VALUES (?lookup ?taxonName) {{\n    \
             {values}\n  \
           }}\n  \
           ?taxon wdt:P225 ?taxonName ;\n        \
                  wdt:P31 wd:Q16521 .\n\
         }}"
    )
}

pub async fn resolve_taxon_qids_batch<'a>(
    names: impl IntoIterator<Item = &'a str>,
) -> Result<HashMap<String, String>, CurationError> {
    let mut lookups = Vec::new();
    let mut seen = HashSet::new();

    for name in names {
        let trimmed = name.trim();
        let Some(lookup) = normalize_taxon_lookup(trimmed) else {
            continue;
        };
        if seen.insert(lookup.clone()) {
            lookups.push((lookup, trimmed.to_string()));
        }
    }

    if lookups.is_empty() {
        return Ok(HashMap::new());
    }

    let query = build_taxon_lookup_query(&lookups);
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    let json =
        serde_json::from_str::<Value>(&raw).map_err(|e| CurationError::Parse(e.to_string()))?;

    let mut resolved = HashMap::new();
    for binding in json
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(lookup) = binding_value(binding, "lookup") else {
            continue;
        };
        let Some(qid) = binding
            .get("taxon")
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
            .and_then(extract_qid_from_uri)
        else {
            continue;
        };

        resolved.insert(lookup, qid.to_string());
    }

    Ok(resolved)
}

pub(super) async fn resolve_taxon_qid(name: &str) -> Result<Option<String>, CurationError> {
    let Some(query) = build_single_taxon_lookup_query(name) else {
        return Ok(None);
    };
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    extract_first_qid(&raw, "taxon")
}

pub async fn resolve_reference_qid(doi: &str) -> Result<Option<String>, CurationError> {
    let query = format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?ref WHERE {{ ?ref wdt:P356 \"{}\" . }} LIMIT 1",
        escape_sparql_string(&doi.to_ascii_uppercase())
    );
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    extract_first_qid(&raw, "ref")
}

pub async fn compound_has_taxon_with_ref(
    compound_qid: &str,
    taxon_qid: &str,
    ref_qid: &str,
) -> Result<bool, CurationError> {
    let query = format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         ASK {{\n  \
           wd:{compound_qid} p:P703 ?stmt .\n  \
           ?stmt ps:P703 wd:{taxon_qid} ;\n        \
                 prov:wasDerivedFrom ?refnode .\n  \
           ?refnode pr:P248 wd:{ref_qid} .\n\
         }}"
    );
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    let parsed =
        serde_json::from_str::<Value>(&raw).map_err(|e| CurationError::Parse(e.to_string()))?;
    Ok(parsed
        .get("boolean")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

pub async fn compound_has_taxon(
    compound_qid: &str,
    taxon_qid: &str,
) -> Result<bool, CurationError> {
    let query = format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         ASK {{ wd:{compound_qid} wdt:{WD_OCCURS_IN_TAXON_PROP} wd:{taxon_qid} . }}"
    );
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    let parsed =
        serde_json::from_str::<Value>(&raw).map_err(|e| CurationError::Parse(e.to_string()))?;
    Ok(parsed
        .get("boolean")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_taxon_lookup_trims_and_lowercases() {
        assert_eq!(
            normalize_taxon_lookup("  Gentiana lutea  "),
            Some("gentiana lutea".to_string())
        );
        assert_eq!(normalize_taxon_lookup("   \n"), None);
    }

    #[test]
    fn build_taxon_lookup_query_uses_values_pairs_and_taxon_type_constraint() {
        let query = build_taxon_lookup_query(&[
            ("voacanga africana".into(), "Voacanga africana".into()),
            ("gentiana lutea".into(), "Gentiana lutea".into()),
        ]);

        assert!(query.contains("VALUES (?lookup ?taxonName)"));
        assert!(query.contains("wdt:P225 ?taxonName"));
        assert!(query.contains("wdt:P31 wd:Q16521"));
        assert!(query.contains("\"voacanga africana\" \"Voacanga africana\""));
    }

    #[test]
    fn build_single_taxon_lookup_query_uses_values_without_lcase_filter() {
        let query = build_single_taxon_lookup_query("ficticia imaginaria").expect("query");

        assert!(query.contains("VALUES ?taxonName"));
        assert!(query.contains("\"ficticia imaginaria\" \"Ficticia imaginaria\""));
        assert!(query.contains("wdt:P31 wd:Q16521"));
        assert!(!query.contains("LCASE"));
        assert!(!query.contains("FILTER"));
    }

    #[test]
    fn build_reference_lookup_query_uses_values_pairs() {
        let query = build_reference_lookup_query(&["10.1000/ABC".into(), "10.2000/XYZ".into()]);

        assert!(query.contains("VALUES (?lookup ?doi)"));
        assert!(query.contains("\"10.1000/ABC\" \"10.1000/ABC\""));
        assert!(query.contains("?ref wdt:P356 ?doi"));
    }
}
