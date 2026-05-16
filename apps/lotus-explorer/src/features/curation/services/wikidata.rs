// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;
use std::collections::HashMap;

const LOOKUP_BATCH_SIZE: usize = 128;

pub(super) async fn fetch_wikidata_compound_by_inchikey(
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

pub(super) async fn resolve_taxon_qid(name: &str) -> Result<Option<String>, CurationError> {
    let query = format!(
        "{CURATION_SPARQL_PREFIXES}\n\
         SELECT ?taxon WHERE {{\n  \
           ?taxon wdt:P225 ?taxonName .\n  \
           FILTER(LCASE(STR(?taxonName)) = LCASE(\"{}\"))\n\
         }} LIMIT 1",
        escape_sparql_string(name)
    );
    let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    extract_first_qid(&raw, "taxon")
}

pub(super) async fn resolve_taxon_qids_batch(
    names: &[String],
) -> Result<HashMap<String, String>, CurationError> {
    if names.is_empty() {
        return Ok(HashMap::new());
    }
    if names.len() == 1 {
        let mut resolved = HashMap::new();
        if let Some(qid) = resolve_taxon_qid(&names[0]).await? {
            resolved.insert(names[0].clone(), qid);
        }
        return Ok(resolved);
    }

    let mut resolved = HashMap::new();
    for chunk in names.chunks(LOOKUP_BATCH_SIZE) {
        let values = chunk
            .iter()
            .map(|name| format!("\"{}\"", escape_sparql_string(name)))
            .collect::<Vec<_>>()
            .join(" ");
        let query = format!(
            "{CURATION_SPARQL_PREFIXES}\n\
             SELECT ?lookup ?taxon WHERE {{\n  \
               VALUES ?lookup {{ {values} }}\n  \
               ?taxon wdt:P225 ?taxonName .\n  \
               FILTER(LCASE(STR(?taxonName)) = ?lookup)\n\
             }}"
        );
        let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
            .await
            .map_err(|e| CurationError::Http(e.to_string()))?;
        resolved.extend(extract_qid_map(&raw, "lookup", "taxon")?);
    }
    Ok(resolved)
}

pub(super) async fn resolve_reference_qid(doi: &str) -> Result<Option<String>, CurationError> {
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

pub(super) async fn resolve_reference_qids_batch(
    dois: &[String],
) -> Result<HashMap<String, String>, CurationError> {
    if dois.is_empty() {
        return Ok(HashMap::new());
    }
    if dois.len() == 1 {
        let mut resolved = HashMap::new();
        if let Some(qid) = resolve_reference_qid(&dois[0]).await? {
            resolved.insert(dois[0].clone(), qid);
        }
        return Ok(resolved);
    }

    let mut resolved = HashMap::new();
    for chunk in dois.chunks(LOOKUP_BATCH_SIZE) {
        let values = chunk
            .iter()
            .map(|doi| format!("\"{}\"", escape_sparql_string(doi)))
            .collect::<Vec<_>>()
            .join(" ");
        let query = format!(
            "{CURATION_SPARQL_PREFIXES}\n\
             SELECT ?lookup ?ref WHERE {{\n  \
               VALUES ?lookup {{ {values} }}\n  \
               ?ref wdt:P356 ?refDoi .\n  \
               FILTER(UCASE(STR(?refDoi)) = ?lookup)\n\
             }}"
        );
        let raw = execute_sparql_format(&query, SparqlResponseFormat::SparqlJson)
            .await
            .map_err(|e| CurationError::Http(e.to_string()))?;
        resolved.extend(extract_qid_map(&raw, "lookup", "ref")?);
    }
    Ok(resolved)
}

pub(super) async fn compound_has_taxon_with_ref(
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

pub(super) async fn compound_has_taxon(
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

fn extract_qid_map(
    raw_json: &str,
    key_var: &str,
    qid_var: &str,
) -> Result<HashMap<String, String>, CurationError> {
    let json =
        serde_json::from_str::<Value>(raw_json).map_err(|e| CurationError::Parse(e.to_string()))?;
    let mut out = HashMap::new();
    if let Some(bindings) = json
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
    {
        for binding in bindings {
            let Some(lookup) = binding
                .get(key_var)
                .and_then(|v| v.get("value"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let Some(qid) = binding
                .get(qid_var)
                .and_then(|v| v.get("value"))
                .and_then(Value::as_str)
                .and_then(extract_qid_from_uri)
            else {
                continue;
            };
            out.entry(lookup.to_string())
                .or_insert_with(|| qid.to_string());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_qid_map_reads_lookup_to_qid_bindings() {
        let raw = r#"{
          "results": {
            "bindings": [
              {
                "lookup": { "type": "literal", "value": "gentiana lutea" },
                "taxon": { "type": "uri", "value": "http://www.wikidata.org/entity/Q123" }
              },
              {
                "lookup": { "type": "literal", "value": "10.1000/ABC" },
                "ref": { "type": "uri", "value": "http://www.wikidata.org/entity/Q456" }
              }
            ]
          }
        }"#;

        let taxa = extract_qid_map(raw, "lookup", "taxon").expect("taxa map");
        assert_eq!(taxa.get("gentiana lutea"), Some(&"Q123".to_string()));

        let refs = extract_qid_map(raw, "lookup", "ref").expect("ref map");
        assert_eq!(refs.get("10.1000/ABC"), Some(&"Q456".to_string()));
    }
}
