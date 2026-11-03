// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, SourceSummary};
use crate::query::{build_lotus_query_qlever, build_lotus_query_wdqs, build_pubchem_query};
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
        let inchikey = binding_str(&binding, "inchikey");
        if inchikey.is_empty() {
            continue;
        }
        let connectivity = inchikey.split('-').next().unwrap_or(inchikey);
        let entry = summary.entry(connectivity.to_string()).or_default();

        // Get the related item QID (main compound, or discovered stereoisomer/tautomer/parent)
        let related_uri = binding_str(&binding, "related_item");

        // Get taxon if available - insert BEFORE logging so count is accurate
        let taxon_name = binding_str(&binding, "taxon_name");
        let has_taxon = !taxon_name.is_empty();
        if has_taxon {
            entry.taxa.insert(taxon_name.to_string());
        }

        if let Some(qid) = related_uri.strip_prefix("http://www.wikidata.org/entity/")
            && !qid.is_empty()
        {
            entry.compounds.insert(qid.to_string());
            if has_taxon {
                entry.compounds_with_taxa.insert(qid.to_string());
            }
        }
    }

    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_pubchem_hits(
    http: &ReqwestClient,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();

    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_query(chunk);

        for binding in sparql_bindings(http, QLEVER_PUBCHEM, &query).await? {
            let inchikey = binding_str(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            // Match on the 14-character skeleton hash only
            let key = inchikey.split('-').next().unwrap_or(inchikey);
            let cid = binding_str(&binding, "related_cid");

            let entry = summary.entry(key.to_string()).or_default();

            if !cid.is_empty() {
                entry.cids.insert(cid.to_string());
            }
        }
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
    let mut json: Value = serde_json::from_str(&response).map_err(|err| err.to_string())?;
    let bindings = json
        .as_object_mut()
        .and_then(|root| root.remove("results"))
        .and_then(|mut results| results.as_object_mut()?.remove("bindings"))
        .and_then(|bindings| match bindings {
            Value::Array(arr) => Some(arr),
            _ => None,
        })
        .ok_or_else(|| "SPARQL response missing bindings".to_string())?;
    Ok(bindings
        .into_iter()
        .filter_map(|v| match v {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
fn binding_str<'a>(binding: &'a serde_json::Map<String, Value>, key: &str) -> &'a str {
    binding
        .get(key)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
}
