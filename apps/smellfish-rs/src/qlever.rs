use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, SourceSummary};
use futures::future::join;
use lotus::transport::{ResponseFormat, execute_sparql_with_format};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
pub const LOTUS_ENDPOINT: &str = "https://qlever.dev/api/wikidata";
#[cfg(target_arch = "wasm32")]
pub const WDQS_ENDPOINT: &str = "https://query.wikidata.org/sparql";
#[cfg(target_arch = "wasm32")]
pub const PUBCHEM_ENDPOINT: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";
#[cfg(target_arch = "wasm32")]
const QUERY_CHUNK_SIZE: usize = 50;

#[cfg(target_arch = "wasm32")]
pub async fn enrich_sources(
    inchikeys: &[String],
    _smiles_list: &[String],
    mut set_status: impl FnMut(String),
) -> EnrichmentOutcome {
    let mut warnings = Vec::new();

    // Probe qlever and pubchem endpoints (in parallel)
    let (qlever_probe, pubchem_probe) = join(
        probe_endpoint("LOTUS", LOTUS_ENDPOINT),
        probe_endpoint("PubChem", PUBCHEM_ENDPOINT),
    )
    .await;

    // Try WDQS first (most reliable for complex queries)
    set_status("Querying data sources…".to_string());

    let lotus_result = fetch_lotus_hits_by_inchikey(WDQS_ENDPOINT, inchikeys).await;

    let lotus = match lotus_result {
        Ok(data) => data,
        Err(err) => {
            warnings.push(format!(
                "LOTUS WDQS failed, switched to Qlever fallback: {err}"
            ));
            // Fall back to qlever with optimized query
            match fetch_lotus_hits_by_inchikey(LOTUS_ENDPOINT, inchikeys).await {
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
        endpoint: WDQS_ENDPOINT.to_string(),
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
        match fetch_pubchem_hits(inchikeys).await {
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
async fn fetch_lotus_hits_by_inchikey(
    endpoint: &str,
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    // Use WDQS query for WDQS, qlever-specific query for qlever
    let query = if endpoint == WDQS_ENDPOINT {
        build_lotus_query_wdqs(inchikeys)
    } else {
        build_lotus_query_qlever(inchikeys)
    };

    let bindings = sparql_bindings(endpoint, &query).await?;

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

        if let Some(qid) = related_uri.strip_prefix("http://www.wikidata.org/entity/") {
            if !qid.is_empty() {
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
    }

    Ok(summary)
}

/// Build Wikidata LOTUS query for WDQS (standard nested SELECT approach).
fn build_lotus_query_wdqs(inchikeys: &[String]) -> String {
    let values = inchikeys
        .iter()
        .filter(|s| !s.is_empty())
        .map(|v| format!("\"{}\"", escape_sparql_literal(v)))
        .collect::<Vec<_>>()
        .join(" ");

    if values.is_empty() {
        return "SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {} # empty".to_string();
    }

    // Nested SELECT structure for cleaner query
    format!(
        r#"PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {{
  {{
    SELECT DISTINCT ?inchikey ?connectivity ?related_item WHERE {{
      VALUES ?inchikey {{ {values} }}
      BIND(SUBSTR(?inchikey, 1 , 14 ) AS ?connectivity)
      ?item wdt:P235 ?inchikey;
        (wdt:P3364|wdt:P6185|(wdt:P279*)|^(wdt:P279+)) ?related_item.
      OPTIONAL {{ ?related_item wdt:P235 ?related_inchikey. }}
      OPTIONAL {{
        ?item wdt:P6185 ?related_item.
        BIND("true"^^xsd:boolean AS ?is_tautomer)
      }}
      FILTER(((?item = ?related_item) || (BOUND(?is_tautomer))) || (STRSTARTS(?related_inchikey, ?connectivity)))
    }}
  }}
  OPTIONAL {{ ?related_item (wdt:P703/wdt:P225) ?taxon_name. }}
}}"#
    )
}

/// Build Wikidata LOTUS query for qlever (optimized union structure).
fn build_lotus_query_qlever(inchikeys: &[String]) -> String {
    let values = inchikeys
        .iter()
        .filter(|s| !s.is_empty())
        .map(|v| format!("\"{}\"", escape_sparql_literal(v)))
        .collect::<Vec<_>>()
        .join(" ");

    if values.is_empty() {
        return "SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {} # empty".to_string();
    }

    // Union-based query for qlever optimization
    format!(
        r#"PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {{
  {{
    SELECT DISTINCT ?inchikey ?connectivity ?related_item WHERE {{
      VALUES ?inchikey {{ {values} }}
      BIND(SUBSTR(?inchikey, 1, 14) AS ?connectivity)
      ?item wdt:P235 ?inchikey .

      {{
        ?item (wdt:P3364|wdt:P6185|(wdt:P279*)|^(wdt:P279+)) ?related_item .
        OPTIONAL {{ ?related_item wdt:P235 ?related_inchikey . }}
        OPTIONAL {{
          ?item wdt:P6185 ?related_item .
          BIND("true"^^xsd:boolean AS ?is_tautomer)
        }}
        FILTER(((?item = ?related_item) || (BOUND(?is_tautomer))) || (STRSTARTS(?related_inchikey, ?connectivity)))
      }}
      UNION
      {{
        # Native QLever compressed dictionary prefix lookup
        ?related_item wdt:P235 ?prefix_inchikey .
        FILTER(STRSTARTS(?prefix_inchikey, ?connectivity))
      }}
    }}
  }}
  OPTIONAL {{ ?related_item (wdt:P703/wdt:P225) ?taxon_name . }}
}}"#
    )
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
            let cid = binding_value(&binding, "related_cid");

            if !cid.is_empty() {
                web_sys::console::log_1(&format!("  PubChem: {} -> CID {}", key, cid).into());
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
