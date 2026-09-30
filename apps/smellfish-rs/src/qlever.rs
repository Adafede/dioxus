// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, SourceSummary};
use crate::query::{build_lotus_query_qlever, build_lotus_query_wdqs, escape_sparql_literal};
use crate::sparql::{QLEVER_PUBCHEM, QLEVER_WIKIDATA, WDQS_WIKIDATA, run_query};
use futures::future::join;
use lotus_search::ResponseFormat;
use serde_json::Value;
use std::collections::HashMap;

// The browser's `fetch`, through the pooled client `lotus-search` builds for it.
// Constructed once per enrichment run and passed to every query, because a
// client per query is a fresh connection pool per query.
#[cfg(target_arch = "wasm32")]
use lotus_search::reqwest_client::ReqwestClient;

#[cfg(target_arch = "wasm32")]
const QUERY_CHUNK_SIZE: usize = 50;

#[cfg(target_arch = "wasm32")]
pub(crate) async fn enrich_sources(
    inchikeys: &[String],
    _smiles_list: &[String],
    mut set_status: impl FnMut(String),
) -> EnrichmentOutcome {
    // One client for the whole run, so the three endpoints' connections are
    // pooled rather than rebuilt per query.
    let http = match ReqwestClient::new() {
        Ok(http) => http,
        Err(err) => {
            return EnrichmentOutcome {
                enrichment: Enrichment {
                    lotus: HashMap::new(),
                    pubchem: HashMap::new(),
                },
                endpoints: vec![EndpointStatus {
                    name: "HTTP client".to_string(),
                    endpoint: String::new(),
                    reachable: false,
                    detail: err.to_string(),
                }],
                warnings: vec![format!("could not open an HTTP client: {err}")],
            };
        }
    };
    let mut warnings = Vec::new();

    // Probe qlever and pubchem endpoints (in parallel)
    let (qlever_probe, pubchem_probe) = join(
        probe_endpoint(&http, "LOTUS", QLEVER_WIKIDATA),
        probe_endpoint(&http, "PubChem", QLEVER_PUBCHEM),
    )
    .await;

    // Try WDQS first (most reliable for complex queries)
    set_status("Querying data sources…".to_string());

    let lotus_result = fetch_lotus_hits_by_inchikey(&http, WDQS_WIKIDATA, inchikeys).await;

    let lotus = match lotus_result {
        Ok(data) => data,
        Err(err) => {
            warnings.push(format!(
                "LOTUS WDQS failed, switched to Qlever fallback: {err}"
            ));
            // Fall back to qlever with optimized query
            match fetch_lotus_hits_by_inchikey(&http, QLEVER_WIKIDATA, inchikeys).await {
                Ok(data) => data,
                Err(err) => {
                    warnings.push(format!("LOTUS qlever also failed: {err}"));
                    HashMap::new()
                }
            }
        }
    };

    // Show WDQS as the primary endpoint since that's what we actually queried
    let lotus_probe = EndpointStatus {
        name: "LOTUS".to_string(),
        endpoint: WDQS_WIKIDATA.to_string(),
        reachable: !lotus.is_empty(), // Reachable if we got results
        detail: if lotus.is_empty() {
            "no results".to_string()
        } else {
            "online".to_string()
        },
    };

    // Use classical InChIKey lookup for PubChem
    let pubchem = if pubchem_probe.reachable {
        set_status("Querying PubChem (InChIKey lookup)…".to_string());
        match fetch_pubchem_hits(&http, inchikeys).await {
            Ok(data) => data,
            Err(err) => {
                warnings.push(format!("PubChem search failed: {err}"));
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
        endpoints: vec![lotus_probe, qlever_probe, pubchem_probe],
        warnings,
    }
}

#[cfg(target_arch = "wasm32")]
async fn probe_endpoint(http: &ReqwestClient, name: &str, endpoint: &str) -> EndpointStatus {
    match run_query(http, endpoint, "ASK {}", ResponseFormat::SparqlJson).await {
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
async fn fetch_lotus_hits_by_inchikey(
    http: &ReqwestClient,
    endpoint: &str,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    // Use WDQS query for WDQS, qlever-specific query for qlever
    let query = if endpoint == WDQS_WIKIDATA {
        build_lotus_query_wdqs(inchikeys)
    } else {
        build_lotus_query_qlever(inchikeys)
    };

    let bindings = sparql_bindings(http, endpoint, &query).await?;

    let mut summary: HashMap<String, SourceSummary> = HashMap::new();

    for binding in bindings {
        // Get the InChIKey we're querying for
        let inchikey = binding_value(&binding, "inchikey");
        if inchikey.is_empty() {
            continue;
        }
        let connectivity = inchikey.split('-').next().unwrap_or(&inchikey).to_string();

        let entry = summary.entry(connectivity.clone()).or_default();

        // Get the related item QID (main compound, or discovered stereoisomer/tautomer/parent)
        let related_uri = binding_value(&binding, "related_item");

        // Get taxon if available - insert BEFORE logging so count is accurate
        let taxon_name = binding_value(&binding, "taxon_name");
        let has_taxon = !taxon_name.is_empty();
        if has_taxon {
            entry.taxa.insert(taxon_name);
        }

        if let Some(qid) = related_uri.strip_prefix("http://www.wikidata.org/entity/")
            && !qid.is_empty()
        {
            entry.compounds.insert(qid.to_string());
            // If we have a taxon for this QID, also track it in compounds_with_taxa
            if has_taxon {
                entry.compounds_with_taxa.insert(qid.to_string());
            }
            // Log to console: taxon count next to QID (after taxon insertion)
            web_sys::console::log_1(
                &format!("LOTUS: QID:{} taxa:{}", qid, entry.taxa.len()).into(),
            );
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
        r"
PREFIX cheminf: <http://semanticscience.org/resource/>
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX vocab: <http://rdf.ncbi.nlm.nih.gov/pubchem/vocabulary#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?inchikey ?related_cid WHERE {{
  VALUES ?inchikey {{ {values} }}
  ?compound vocab:inchikey ?inchikey .
  {{
    ?compound dcterms:identifier ?related_cid .
  }}
  UNION
  {{
    ?compound cheminf:CHEMINF_000461 ?stereoisomer .
    ?stereoisomer dcterms:identifier ?related_cid .
  }}
  UNION
  {{
    ?compound cheminf:CHEMINF_000462 ?same_conn .
    ?same_conn dcterms:identifier ?related_cid .
  }}
}}
"
    )
}

#[cfg(target_arch = "wasm32")]
async fn fetch_pubchem_hits(
    http: &ReqwestClient,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    web_sys::console::log_1(&format!("PubChem: Querying {} InChIKeys", inchikeys.len()).into());

    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_query(chunk);
        web_sys::console::log_1(&format!("PubChem chunk query: {} InChIKeys", chunk.len()).into());

        for binding in sparql_bindings(http, QLEVER_PUBCHEM, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            // Match on the 14-character skeleton hash only
            let key = inchikey.split('-').next().unwrap_or(&inchikey).to_string();
            let cid = binding_value(&binding, "related_cid");

            if !cid.is_empty() {
                web_sys::console::log_1(&format!("  PubChem: {key} -> CID {cid}").into());
            }

            let entry = summary.entry(key.clone()).or_default();

            if !cid.is_empty() {
                entry.cids.insert(cid);
            }
        }
    }
    web_sys::console::log_1(
        &format!("PubChem FINAL: {} skeletons with hits", summary.len()).into(),
    );
    for key in summary.keys() {
        web_sys::console::log_1(&format!("  Key: {key}").into());
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn sparql_bindings(
    http: &ReqwestClient,
    endpoint: &str,
    query: &str,
) -> Result<Vec<serde_json::Map<String, Value>>, String> {
    let response = run_query(http, endpoint, query, ResponseFormat::SparqlJson)
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
