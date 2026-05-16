// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;

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

/// Returns (Option<QID>, Vec<creation_QS_lines>).
/// If the taxon exists, returns (Some(qid), []). Otherwise returns (None, <minimal CREATE QS>).
pub(super) async fn resolve_or_create_taxon(
    name: &str,
) -> Result<(Option<String>, Vec<String>), CurationError> {
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
