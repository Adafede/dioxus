use crate::model::{EndpointStatus, Enrichment, EnrichmentOutcome, SourceSummary};
use futures::future::join;
use serde_json::Value;
use shared::sparql::{ResponseFormat, execute_sparql_with_format};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
pub const LOTUS_ENDPOINT: &str = "https://qlever.dev/api/wikidata";
#[cfg(target_arch = "wasm32")]
pub const PUBCHEM_ENDPOINT: &str = "https://qlever.cs.uni-freiburg.de/api/pubchem";
#[cfg(target_arch = "wasm32")]
const QUERY_CHUNK_SIZE: usize = 40;

#[cfg(target_arch = "wasm32")]
pub async fn enrich_sources(inchikeys: &[String]) -> EnrichmentOutcome {
    let (lotus_probe, pubchem_probe) = join(
        probe_endpoint("LOTUS", LOTUS_ENDPOINT),
        probe_endpoint("PubChem", PUBCHEM_ENDPOINT),
    )
    .await;

    let mut warnings = Vec::new();
    let lotus = if lotus_probe.reachable {
        match fetch_lotus_hits(inchikeys).await {
            Ok(data) => data,
            Err(err) => {
                warnings.push(format!("LOTUS exact lookup failed: {err}"));
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

    let pubchem = if pubchem_probe.reachable {
        match fetch_pubchem_hits(inchikeys).await {
            Ok(data) => data,
            Err(err) => {
                warnings.push(format!("PubChem exact lookup failed: {err}"));
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
async fn fetch_lotus_hits(inchikeys: &[String]) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_lotus_query(chunk);
        for binding in sparql_bindings(LOTUS_ENDPOINT, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            let entry = summary.entry(inchikey).or_default();
            let compound = binding_value(&binding, "compoundLabel");
            let taxon = binding_value(&binding, "taxonLabel");
            if !compound.is_empty() {
                entry.compounds.insert(compound);
            }
            if !taxon.is_empty() {
                entry.taxa.insert(taxon);
            }
        }
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_pubchem_hits(
    inchikeys: &[String],
) -> Result<HashMap<String, SourceSummary>, String> {
    let mut summary: HashMap<String, SourceSummary> = HashMap::new();
    for chunk in inchikeys.chunks(QUERY_CHUNK_SIZE) {
        let query = build_pubchem_query(chunk);
        for binding in sparql_bindings(PUBCHEM_ENDPOINT, &query).await? {
            let inchikey = binding_value(&binding, "inchikey");
            if inchikey.is_empty() {
                continue;
            }
            let entry = summary.entry(inchikey).or_default();
            let cid = binding_value(&binding, "cid");
            let label = binding_value(&binding, "label");
            let iupac = binding_value(&binding, "iupac");
            let taxon = binding_value(&binding, "taxonLabel");
            if !cid.is_empty() {
                entry.cids.insert(cid);
            }
            if !label.is_empty() {
                entry.names.insert(label);
            }
            if !iupac.is_empty() {
                entry.names.insert(iupac);
            }
            if !taxon.is_empty() {
                entry.taxa.insert(taxon);
            }
        }
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
fn build_lotus_query(chunk: &[String]) -> String {
    let values = chunk
        .iter()
        .map(|value| format!("\"{}\"", escape_sparql_literal(value)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"
PREFIX wd: <http://www.wikidata.org/entity/>
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX p: <http://www.wikidata.org/prop/>
PREFIX ps: <http://www.wikidata.org/prop/statement/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?inchikey ?compound ?compoundLabel ?taxonLabel WHERE {{
  VALUES ?inchikey {{ {values} }}
  ?compound wdt:P235 ?inchikey ;
            p:P703 ?statement .
  ?statement ps:P703 ?taxon .
  OPTIONAL {{ ?compound rdfs:label ?compoundLabel FILTER(LANG(?compoundLabel) = "en") }}
  OPTIONAL {{ ?taxon rdfs:label ?taxonLabel FILTER(LANG(?taxonLabel) = "en") }}
}}
"#
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
fn escape_sparql_literal(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', r#"\""#)
}
