//! Export metadata helpers for LOTUS results.

use crate::models::{ElementState, SearchCriteria, SmilesSearchType};
use serde_json::{Map, Value, json};

pub const APP_VERSION: &str = "0.1.0";
pub const APP_NAME: &str = "LOTUS Wikidata Explorer (Dioxus port)";
pub const APP_URL: &str = "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explorer";
pub const QLEVER_ENDPOINT: &str = "https://qlever.dev/api/wikidata";

fn export_search_type_suffix(criteria: &SearchCriteria) -> Option<&'static str> {
    if criteria.smiles.trim().is_empty() {
        None
    } else {
        Some(match criteria.smiles_search_type {
            SmilesSearchType::Substructure => "substructure",
            SmilesSearchType::Similarity => "similarity",
        })
    }
}

// ── Utility: ISO-8601 "now" ───────────────────────────────────────────────────

pub fn now_iso8601() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let s: String = js_sys::Date::new_0().to_iso_string().into();
        s
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        // Crude ISO formatter — good enough for metadata; not chrono-accurate for leap years
        // but fine for the metadata header.
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let (y, m, d, hh, mm, ss) = epoch_to_ymdhms(secs);
        format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn epoch_to_ymdhms(secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    // Small, dependency-free algorithm (Howard Hinnant, public domain).
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hh = (rem / 3600) as u32;
    let mm = ((rem % 3600) / 60) as u32;
    let ss = (rem % 60) as u32;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 {
        (mp + 3) as u32
    } else {
        (mp - 9) as u32
    };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d, hh, mm, ss)
}

// ── Filters → JSON (mirrors Python `SearchCriteria.to_filters_dict`) ──────────

pub fn criteria_to_filters_value(criteria: &SearchCriteria) -> Value {
    let mut filters = Map::new();

    if !criteria.smiles.trim().is_empty() {
        let mut cs = Map::new();
        cs.insert("smiles".into(), Value::String(criteria.smiles.clone()));
        cs.insert(
            "search_type".into(),
            Value::String(match criteria.smiles_search_type {
                SmilesSearchType::Substructure => "substructure".into(),
                SmilesSearchType::Similarity => "similarity".into(),
            }),
        );
        if criteria.smiles_search_type == SmilesSearchType::Similarity {
            cs.insert(
                "similarity_threshold".into(),
                json!(criteria.smiles_threshold),
            );
        }
        filters.insert("chemical_structure".into(), Value::Object(cs));
    }

    if criteria.has_mass_filter() {
        filters.insert(
            "mass".into(),
            json!({ "min": criteria.mass_min, "max": criteria.mass_max }),
        );
    }

    if criteria.has_year_filter() {
        filters.insert(
            "publication_year".into(),
            json!({ "start": criteria.year_min, "end": criteria.year_max }),
        );
    }

    if criteria.has_formula_filter() {
        let mut mf = Map::new();
        let exact = criteria.formula_exact.trim();
        if !exact.is_empty() {
            mf.insert("exact_formula".into(), Value::String(exact.into()));
        }
        for (name, min, max, default_max) in [
            (
                "carbon",
                criteria.c_min,
                criteria.c_max,
                crate::models::DEFAULT_C_MAX,
            ),
            (
                "hydrogen",
                criteria.h_min,
                criteria.h_max,
                crate::models::DEFAULT_H_MAX,
            ),
            (
                "nitrogen",
                criteria.n_min,
                criteria.n_max,
                crate::models::DEFAULT_N_MAX,
            ),
            (
                "oxygen",
                criteria.o_min,
                criteria.o_max,
                crate::models::DEFAULT_O_MAX,
            ),
            (
                "phosphorus",
                criteria.p_min,
                criteria.p_max,
                crate::models::DEFAULT_P_MAX,
            ),
            (
                "sulfur",
                criteria.s_min,
                criteria.s_max,
                crate::models::DEFAULT_S_MAX,
            ),
        ] {
            if min > 0 || max < default_max {
                mf.insert(name.into(), json!({ "min": min, "max": max }));
            }
        }
        let halogens: Vec<(&str, ElementState)> = vec![
            ("fluorine", criteria.f_state),
            ("chlorine", criteria.cl_state),
            ("bromine", criteria.br_state),
            ("iodine", criteria.i_state),
        ];
        let mut hal = Map::new();
        for (name, state) in halogens {
            if state != ElementState::Allowed {
                hal.insert(name.into(), Value::String(state.as_str().into()));
            }
        }
        if !hal.is_empty() {
            mf.insert("halogens".into(), Value::Object(hal));
        }
        if !mf.is_empty() {
            filters.insert("molecular_formula".into(), Value::Object(mf));
        }
    }

    Value::Object(filters)
}

// ── Metadata (Schema.org Dataset, JSON-LD) ───────────────────────────────────

pub struct MetadataInputs<'a> {
    pub criteria: &'a SearchCriteria,
    pub qid: Option<&'a str>,
    pub number_of_records_override: Option<usize>,
    pub query_hash: &'a str,
    pub result_hash: &'a str,
}

pub fn build_metadata_json(inp: MetadataInputs<'_>) -> String {
    let filters = criteria_to_filters_value(inp.criteria);

    let effective_taxon = match (inp.qid, inp.criteria.taxon.trim()) {
        (Some("*"), _) | (None, "") => "all taxa".to_string(),
        (_, t) if !t.is_empty() => t.to_string(),
        (Some(q), _) => q.to_string(),
        _ => "all taxa".to_string(),
    };

    let chem = filters.get("chemical_structure").cloned();
    let (dataset_name, description) = if let Some(c) = chem.as_ref() {
        let st = c
            .get("search_type")
            .and_then(|v| v.as_str())
            .unwrap_or("substructure");
        (
            format!(
                "LOTUS Data — {} search in {effective_taxon}",
                title_case(st)
            ),
            format!(
                "Chemical compounds from {effective_taxon}. Retrieved via LOTUS \
                 Wikidata Explorer with {st} chemical search (SACHEM/IDSM)."
            ),
        )
    } else {
        (
            format!("LOTUS Data — {effective_taxon}"),
            format!(
                "Chemical compounds from {effective_taxon}. Retrieved via LOTUS Wikidata Explorer."
            ),
        )
    };

    let mut providers = vec![
        json!({ "@type": "Organization", "name": "LOTUS Initiative",
                "url": "https://www.wikidata.org/wiki/Q104225190" }),
        json!({ "@type": "Organization", "name": "Wikidata",
                "url": "http://www.wikidata.org/" }),
    ];
    if chem.is_some() {
        providers.push(json!({ "@type": "Organization", "name": "IDSM",
                               "url": "https://idsm.elixir-czech.cz/" }));
    }

    let mut search_params = Map::new();
    search_params.insert("taxon".into(), Value::String(effective_taxon));
    search_params.insert(
        "taxon_qid".into(),
        match inp.qid {
            Some(q) if q != "*" => Value::String(q.to_string()),
            _ => Value::Null,
        },
    );
    if let Some(c) = chem.as_ref() {
        let smiles_str = c
            .get("smiles")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let multiline = smiles_str.contains('\n') || smiles_str.contains('\r');
        let mut sq = Map::new();
        sq.insert("param_key".into(), "structure".into());
        sq.insert("legacy_param_key".into(), "smiles".into());
        sq.insert(
            "search_type".into(),
            c.get("search_type")
                .cloned()
                .unwrap_or(json!("substructure")),
        );
        sq.insert(
            "input_format".into(),
            Value::String(if multiline {
                "molfile".into()
            } else {
                "smiles".into()
            }),
        );
        if let Some(t) = c.get("similarity_threshold").cloned() {
            sq.insert("similarity_threshold".into(), t);
        }
        if multiline {
            sq.insert(
                "query_preview".into(),
                Value::String(smiles_str.chars().take(500).collect()),
            );
            sq.insert(
                "query_length".into(),
                Value::Number(smiles_str.len().into()),
            );
        } else {
            sq.insert("query_text".into(), Value::String(smiles_str));
        }
        search_params.insert("structure_query".into(), Value::Object(sq));
    }
    if let Some(obj) = filters.as_object() {
        if !obj.is_empty() {
            search_params.insert("filters".into(), filters.clone());
        }
    }

    let mut meta = Map::new();
    meta.insert(
        "@context".into(),
        Value::String("https://schema.org/".into()),
    );
    meta.insert("@type".into(), Value::String("Dataset".into()));
    meta.insert("name".into(), Value::String(dataset_name));
    meta.insert("description".into(), Value::String(description));
    meta.insert("version".into(), Value::String(APP_VERSION.into()));
    meta.insert("dateCreated".into(), Value::String(now_iso8601()));
    meta.insert(
        "license".into(),
        Value::String("https://creativecommons.org/publicdomain/zero/1.0/".into()),
    );
    meta.insert(
        "creator".into(),
        json!({
            "@type": "SoftwareApplication",
            "name": APP_NAME,
            "version": APP_VERSION,
            "url": APP_URL,
        }),
    );
    meta.insert("provider".into(), Value::Array(providers));
    meta.insert(
        "citation".into(),
        json!([{
            "@type": "ScholarlyArticle",
            "name": "LOTUS initiative",
            "identifier": "https://doi.org/10.7554/eLife.70780",
        }]),
    );
    meta.insert(
        "distribution".into(),
        json!([
            { "@type": "DataDownload", "encodingFormat": "text/csv",         "contentUrl": "data:text/csv" },
            { "@type": "DataDownload", "encodingFormat": "application/sparql-results+json", "contentUrl": "data:application/sparql-results+json" },
            { "@type": "DataDownload", "encodingFormat": "text/turtle",      "contentUrl": "data:text/turtle" },
        ]),
    );
    if let Some(n_records) = inp.number_of_records_override {
        meta.insert("numberOfRecords".into(), Value::Number(n_records.into()));
    }
    meta.insert(
        "variablesMeasured".into(),
        json!([
            "compound_name",
            "compound_smiles",
            "compound_inchikey",
            "compound_mass",
            "molecular_formula",
            "taxon_name",
            "reference_title",
            "reference_doi",
            "reference_date",
            "compound_qid",
            "taxon_qid",
            "reference_qid",
        ]),
    );
    meta.insert("search_parameters".into(), Value::Object(search_params));
    if chem.is_some() {
        meta.insert(
            "chemical_search_service".into(),
            json!({
                "name": "SACHEM",
                "provider": "IDSM",
                "endpoint": "https://idsm.elixir-czech.cz/sparql/endpoint/",
            }),
        );
    }
    meta.insert(
        "sparql_endpoint".into(),
        json!({
            "url": QLEVER_ENDPOINT,
            "name": "QLever Wikidata",
            "description": "Fast SPARQL endpoint for Wikidata",
        }),
    );
    meta.insert(
        "provenance".into(),
        json!({
            "query_hash":  { "algorithm": "SHA-256", "value": inp.query_hash },
            "result_hash": { "algorithm": "SHA-256", "value": inp.result_hash },
            "dataset_uri": format!("urn:hash:sha256:{}", inp.result_hash),
        }),
    );

    serde_json::to_string_pretty(&Value::Object(meta)).unwrap_or_default()
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

// ── Download filenames (mirrors Python `generate_filename`) ──────────────────

/// Compact `YYYYMMDD` date string derived from [`now_iso8601`].
pub fn today_yyyymmdd() -> String {
    // `now_iso8601()` returns `YYYY-MM-DDThh:mm:ssZ`; strip non-digits and
    // keep the first 8 digits (the calendar date).
    now_iso8601()
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(8)
        .collect()
}

/// Normalize a taxon string into a filesystem-safe slug, matching the
/// Python notebook's `generate_filename` logic:
///
/// * empty / whitespace → `any_taxon`
/// * `"*"`              → `all_taxa`
/// * otherwise: spaces / slashes / colons → `_`, `*` → `star`,
///   other shell-unfriendly characters (`?"<>|`) are stripped.
pub fn safe_taxon_slug(taxon: &str) -> String {
    let t = taxon.trim();
    if t.is_empty() {
        return "any_taxon".to_string();
    }
    if t == "*" {
        return "all_taxa".to_string();
    }
    let mut out = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            ' ' | '/' | '\\' | ':' => out.push('_'),
            '*' => out.push_str("star"),
            '?' | '"' | '<' | '>' | '|' => {}
            _ => out.push(c),
        }
    }
    out
}

/// Build a data-download filename matching the app convention:
/// `{YYYYMMDD}_lotus_{safe_taxon}[_{search_type}][_filtered].{ext}`.
pub fn generate_filename(criteria: &SearchCriteria, ext: &str) -> String {
    let date = today_yyyymmdd();
    let safe = safe_taxon_slug(&criteria.taxon);
    let mut stem = format!("{date}_lotus_{safe}");
    if let Some(st) = export_search_type_suffix(criteria) {
        stem.push('_');
        stem.push_str(st);
    }
    if criteria.has_effective_filters() {
        stem.push_str("_filtered");
    }
    format!("{stem}.{ext}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_filename_marks_filtered_taxon_queries() {
        let criteria = SearchCriteria::default();
        let name = generate_filename(&criteria, "csv");
        assert!(name.ends_with("_filtered.csv"));
    }

    #[test]
    fn export_filename_for_full_dataset_has_no_filtered_suffix() {
        let criteria = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        let name = generate_filename(&criteria, "csv");
        assert!(!name.contains("_filtered."));
        assert!(name.ends_with("_all_taxa.csv"));
    }

    #[test]
    fn export_filename_keeps_search_type_before_filtered_suffix() {
        let mut criteria = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        criteria.smiles = "c1ccccc1".into();
        criteria.smiles_search_type = SmilesSearchType::Similarity;
        let name = generate_filename(&criteria, "rdf");
        assert!(name.ends_with("_similarity_filtered.rdf"));
    }
}
