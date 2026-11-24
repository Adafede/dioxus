// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Batched LOTUS (WDQS with `QLever` fallback) and `PubChem` SPARQL enrichment.

use crate::error::SmellfishError;
use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, InChIKey14, SourceSummary};
use crate::query::{build_lotus_query_qlever, build_lotus_query_wdqs, build_pubchem_sparql};
use crate::sparql::{QLEVER_PUBCHEM, QLEVER_WIKIDATA, WDQS_WIKIDATA, run_query};
use futures::future::join;
use lotus_search::{Http, ResponseFormat};
use serde_json::{Map, Value};
use std::collections::HashMap;

const QUERY_CHUNK_SIZE: usize = 50;

/// Queries LOTUS (`WDQS` with `QLever` fallback) and `PubChem` for the given `InChIKey`s.
pub async fn enrich_sources(
    http: &impl Http,
    inchikeys: &[String],
    mut set_status: impl FnMut(&str),
) -> EnrichmentOutcome {
    let mut warnings = Vec::new();
    let (qlever_probe, pubchem_probe) = join(
        probe_endpoint(http, "QLever Wikidata", QLEVER_WIKIDATA),
        probe_endpoint(http, "PubChem", QLEVER_PUBCHEM),
    )
    .await;

    set_status("Querying data sources…");
    let (lotus, lotus_probe) = fetch_lotus_with_fallback(http, inchikeys, &mut warnings).await;

    let pubchem = if pubchem_probe.reachable {
        set_status("Querying PubChem (InChIKey lookup)…");
        match fetch_pubchem_hits(http, inchikeys).await {
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

async fn probe_endpoint(http: &impl Http, name: &str, endpoint: &str) -> EndpointStatus {
    let (reachable, detail) =
        match run_query(http, endpoint, "ASK {}", ResponseFormat::SparqlJson).await {
            Ok(_) => (true, "reachable".to_string()),
            Err(err) => (false, err.to_string()),
        };
    EndpointStatus {
        name: name.to_string(),
        endpoint: endpoint.to_string(),
        reachable,
        detail,
    }
}

fn lotus_status(endpoint: &str, reachable: bool, detail: impl Into<String>) -> EndpointStatus {
    EndpointStatus {
        name: "LOTUS".to_string(),
        endpoint: endpoint.to_string(),
        reachable,
        detail: detail.into(),
    }
}

async fn fetch_lotus_with_fallback(
    http: &impl Http,
    inchikeys: &[String],
    warnings: &mut Vec<String>,
) -> (HashMap<String, SourceSummary>, EndpointStatus) {
    match fetch_lotus_hits(http, WDQS_WIKIDATA, inchikeys).await {
        Ok(data) => {
            let detail = if data.is_empty() {
                "online (0 matches)"
            } else {
                "online"
            };
            (data, lotus_status(WDQS_WIKIDATA, true, detail))
        }
        Err(wdqs_err) => {
            warnings.push(format!(
                "LOTUS WDQS failed, switched to Qlever fallback: {wdqs_err}"
            ));
            match fetch_lotus_hits(http, QLEVER_WIKIDATA, inchikeys).await {
                Ok(data) => {
                    let detail = if data.is_empty() {
                        "online via QLever fallback (0 matches)"
                    } else {
                        "online (QLever fallback)"
                    };
                    (data, lotus_status(QLEVER_WIKIDATA, true, detail))
                }
                Err(qlever_err) => {
                    warnings.push(format!("LOTUS qlever also failed: {qlever_err}"));
                    (
                        HashMap::new(),
                        lotus_status(
                            WDQS_WIKIDATA,
                            false,
                            format!("WDQS: {wdqs_err}; QLever: {qlever_err}"),
                        ),
                    )
                }
            }
        }
    }
}

async fn fetch_lotus_hits(
    http: &impl Http,
    endpoint: &str,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, SmellfishError> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = if endpoint == WDQS_WIKIDATA {
            build_lotus_query_wdqs(chunk)
        } else {
            build_lotus_query_qlever(chunk)
        };
        for binding in sparql_bindings(http, endpoint, &query).await? {
            let Some(connectivity) = InChIKey14::from_inchikey(binding_str(&binding, "inchikey"))
            else {
                continue;
            };
            let entry = summary
                .entry(connectivity.as_str().to_string())
                .or_default();
            let related_uri = binding_str(&binding, "related_item");
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
    }
    Ok(summary)
}

async fn fetch_pubchem_hits(
    http: &impl Http,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, SmellfishError> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_sparql(chunk);
        for binding in sparql_bindings(http, QLEVER_PUBCHEM, &query).await? {
            let Some(key) = InChIKey14::from_inchikey(binding_str(&binding, "inchikey")) else {
                continue;
            };
            let cid = binding_str(&binding, "related_cid");
            let entry = summary.entry(key.as_str().to_string()).or_default();
            if !cid.is_empty() {
                entry.cids.insert(cid.to_string());
            }
        }
    }
    Ok(summary)
}

async fn sparql_bindings(
    http: &impl Http,
    endpoint: &str,
    query: &str,
) -> Result<Vec<Map<String, Value>>, SmellfishError> {
    let response = run_query(http, endpoint, query, ResponseFormat::SparqlJson).await?;
    let mut json: Value =
        serde_json::from_str(&response).map_err(SmellfishError::InvalidSparqlJson)?;
    let bindings = json
        .as_object_mut()
        .and_then(|root| root.remove("results"))
        .and_then(|mut results| results.as_object_mut()?.remove("bindings"))
        .and_then(|bindings| match bindings {
            Value::Array(arr) => Some(arr),
            _ => None,
        })
        .ok_or(SmellfishError::MissingSparqlBindings)?;
    Ok(bindings
        .into_iter()
        .filter_map(|v| match v {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .collect())
}

fn binding_str<'a>(binding: &'a Map<String, Value>, key: &str) -> &'a str {
    binding
        .get(key)
        .and_then(|value| value.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
}
