use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, SourceSummary};
use futures::future::join;
use serde_json::Value;
use shared::lotus::models::SmilesSearchType;
use shared::lotus::queries;
use shared::sparql::{ResponseFormat, execute_sparql_with_format};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
pub const LOTUS_ENDPOINT: &str = "https://qlever.dev/api/wikidata";
#[cfg(target_arch = "wasm32")]
pub const PUBCHEM_ENDPOINT: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";
#[cfg(target_arch = "wasm32")]
const QUERY_CHUNK_SIZE: usize = 40;

#[cfg(target_arch = "wasm32")]
pub async fn enrich_sources(
    inchikeys: &[String],
    smiles_list: &[String],
    set_status: impl FnMut(String),
) -> EnrichmentOutcome {
    let mut warnings = Vec::new();
    let mut set_status = set_status;
    let (lotus_probe, pubchem_probe) = join(
        probe_endpoint("LOTUS", LOTUS_ENDPOINT),
        probe_endpoint("PubChem", PUBCHEM_ENDPOINT),
    )
    .await;

    // Use SMILES-based similarity search for LOTUS
    let lotus = if lotus_probe.reachable {
        set_status("Querying LOTUS (SMILES similarity search)…".to_string());
        match fetch_lotus_hits_by_smiles(smiles_list).await {
            Ok(data) => data,
            Err(err) => {
                warnings.push(format!("LOTUS SMILES search failed: {err}"));
                HashMap::new()
            }
        }
    } else {
        warnings.push(format!(
            "LOTUS endpoint unavailable: {}",
            lotus_probe.detail
        ));
        HashMap::new()
    };

    // Use classical InChIKey lookup for PubChem
    let pubchem = if pubchem_probe.reachable {
        set_status("Querying PubChem (InChIKey lookup)…".to_string());
        match fetch_pubchem_hits(inchikeys).await {
            Ok(data) => data,
            Err(err) => {
                warnings.push(format!("PubChem InChIKey search failed: {err}"));
                HashMap::new()
            }
        }
    } else {
        warnings.push(format!(
            "PubChem endpoint unavailable: {}",
            pubchem_probe.detail
        ));
        HashMap::new()
    };

    EnrichmentOutcome {
        enrichment: Enrichment { lotus, pubchem },
        endpoints: vec![lotus_probe, pubchem_probe],
        warnings,
    }
}

#[cfg(target_arch = "wasm32")]
async fn probe_endpoint(name: &str, endpoint: &str) -> EndpointStatus {
    match execute_sparql_with_format("ASK {}", endpoint, ResponseFormat::SparqlJson).await {
        Ok(_) => EndpointStatus {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            reachable: true,
            detail: "reachable".to_string(),
        },
        Err(err) => EndpointStatus {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            reachable: false,
            detail: err.to_string(),
        },
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_lotus_hits_by_smiles(
    smiles_list: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    let threshold = 1.0; // Will be converted to 0.95 by query_sachem_batch

    for chunk in smiles_list.chunks(QUERY_CHUNK_SIZE) {
        // Convert chunk to &[&str] for query_sachem_batch
        let chunk_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();

        let sparql_query =
            queries::query_sachem_batch(&chunk_refs, SmilesSearchType::Similarity, threshold, None);

        match sparql_bindings(LOTUS_ENDPOINT, &sparql_query).await {
            Ok(bindings) => {
                for binding in bindings {
                    // Use the returned InChIKey 14-char skeleton directly as the key
                    let returned_inchikey = binding_value(&binding, "compound_inchikey");
                    if returned_inchikey.is_empty() {
                        continue;
                    }

                    // Extract 14-char skeleton from returned InChIKey
                    let inchikey_skeleton = returned_inchikey
                        .split('-')
                        .next()
                        .unwrap_or(&returned_inchikey)
                        .to_string();

                    let entry = summary.entry(inchikey_skeleton).or_default();

                    let compound_uri = binding_value(&binding, "c");
                    let qid = if let Some(qid) =
                        compound_uri.strip_prefix("http://www.wikidata.org/entity/")
                    {
                        if !qid.is_empty() {
                            entry.compounds.insert(qid.to_string());
                            Some(qid.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let taxon_label = binding_value(&binding, "taxon_name");
                    if !taxon_label.is_empty() {
                        entry.taxa.insert(taxon_label);
                        // Mark this compound as having taxon info
                        if let Some(ref qid) = qid {
                            entry.compounds_with_taxa.insert(qid.clone());
                        }
                    }
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
fn build_pubchem_query(chunk: &[String]) -> String {
    let values = chunk
        .iter()
        .map(|value| format!("\"{}\"", escape_sparql_literal(value)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX vocab: <http://rdf.ncbi.nlm.nih.gov/pubchem/vocabulary#>
PREFIX skos: <http://www.w3.org/2004/02/skos/core#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?inchikey ?cid ?label ?iupac ?taxonLabel WHERE {{
  VALUES ?inchikey {{ {values} }}
  ?compound dcterms:identifier ?cid ;
            vocab:inchikey ?inchikey .
  OPTIONAL {{ ?compound skos:prefLabel ?label }}
  OPTIONAL {{ ?compound vocab:preferred_iupac_name ?iupac }}
  OPTIONAL {{
    ?compound <http://purl.obolibrary.org/obo/RO_0002162> ?taxon .
    OPTIONAL {{ ?taxon rdfs:label ?taxonLabel FILTER(LANG(?taxonLabel) = "en") }}
  }}
}}
"#
    )
}

#[cfg(target_arch = "wasm32")]
async fn fetch_pubchem_hits(
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    web_sys::console::log_1(&format!("PubChem: Querying {} InChIKeys", inchikeys.len()).into());

    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_query(chunk);
        web_sys::console::log_1(&format!("PubChem chunk query: {} InChIKeys", chunk.len()).into());

        for binding in sparql_bindings(PUBCHEM_ENDPOINT, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            // Match on the 14-character skeleton hash only
            let key = inchikey.split('-').next().unwrap_or(&inchikey).to_string();
            let cid = binding_value(&binding, "cid");

            if !cid.is_empty() {
                web_sys::console::log_1(&format!("  PubChem: {} -> CID {}", key, cid).into());
            }

            let entry = summary.entry(key.clone()).or_default();
            let cid = binding_value(&binding, "cid");

            if !cid.is_empty() {
                web_sys::console::log_1(&format!("  PubChem: {} -> CID {}", key, cid).into());
                entry.cids.insert(cid);
            }
            let taxon = binding_value(&binding, "taxonLabel");
            if !taxon.is_empty() {
                entry.taxa.insert(taxon);
            }
        }
    }
    web_sys::console::log_1(
        &format!("PubChem FINAL: {} skeletons with hits", summary.len()).into(),
    );
    for key in summary.keys() {
        web_sys::console::log_1(&format!("  Key: {}", key).into());
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn sparql_bindings(
    endpoint: &str,
    query: &str,
) -> Result<Vec<serde_json::Map<String, Value>>, String> {
    let response = execute_sparql_with_format(query, endpoint, ResponseFormat::SparqlJson)
        .await
        .map_err(|err| err.to_string())?;
    let json: Value = serde_json::from_str(&response).map_err(|err| err.to_string())?;
    let bindings = json
        .get("results")
        .and_then(|value| value.get("bindings"))
        .and_then(Value::as_array)
        .ok_or_else(|| "SPARQL response missing bindings".to_string())?;
    Ok(bindings
        .iter()
        .filter_map(Value::as_object)
        .cloned()
        .collect())
}

#[cfg(target_arch = "wasm32")]
fn binding_value(binding: &serde_json::Map<String, Value>, key: &str) -> String {
    binding
        .get(key)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(target_arch = "wasm32")]
fn escape_sparql_literal(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', r#"\""#)
}
