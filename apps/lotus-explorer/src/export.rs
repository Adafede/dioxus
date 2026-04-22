//! Export formats for LOTUS results — CSV, NDJSON, Turtle (TTL), and a
//! Schema.org compliant metadata JSON-LD document.
//!
//! Mirrors the Python `CSVExportStrategy`, `JSONExportStrategy`,
//! `TTLExportStrategy`, and `LOTUSExplorer.create_metadata` so that files
//! produced by the Rust/Dioxus port are drop-in compatible with those from
//! the marimo notebook.

use crate::models::{CompoundEntry, DatasetStats, ElementState, SearchCriteria, SmilesSearchType};
use serde_json::{json, Map, Value};

// Wikidata colour palette (matches Python CONFIG).  Values are kept in Rust
// for any code paths that need the colours programmatically (e.g. future
// CLI reports); CSS rules use them via `--wd-*` custom properties.
#[allow(dead_code)] pub const WD_COLOR_COMPOUND:  &str = "#990000"; // red
#[allow(dead_code)] pub const WD_COLOR_TAXON:     &str = "#339966"; // green
#[allow(dead_code)] pub const WD_COLOR_REFERENCE: &str = "#006699"; // blue
#[allow(dead_code)] pub const WD_COLOR_HYPERLINK: &str = "#3377c4";

pub const APP_VERSION: &str = "0.1.0";
pub const APP_NAME: &str = "LOTUS Wikidata Explorer (Dioxus port)";
pub const APP_URL: &str =
    "https://github.com/Adafede/marimo/blob/main/apps/lotus_wikidata_explorer.py";
pub const QLEVER_ENDPOINT: &str = "https://qlever.dev/api/wikidata";

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
    let m = if mp < 10 { (mp + 3) as u32 } else { (mp - 9) as u32 };
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
            ("carbon",     criteria.c_min, criteria.c_max, crate::models::DEFAULT_C_MAX),
            ("hydrogen",   criteria.h_min, criteria.h_max, crate::models::DEFAULT_H_MAX),
            ("nitrogen",   criteria.n_min, criteria.n_max, crate::models::DEFAULT_N_MAX),
            ("oxygen",     criteria.o_min, criteria.o_max, crate::models::DEFAULT_O_MAX),
            ("phosphorus", criteria.p_min, criteria.p_max, crate::models::DEFAULT_P_MAX),
            ("sulfur",     criteria.s_min, criteria.s_max, crate::models::DEFAULT_S_MAX),
        ] {
            if min > 0 || max < default_max {
                mf.insert(name.into(), json!({ "min": min, "max": max }));
            }
        }
        let halogens: Vec<(&str, ElementState)> = vec![
            ("fluorine", criteria.f_state),
            ("chlorine", criteria.cl_state),
            ("bromine",  criteria.br_state),
            ("iodine",   criteria.i_state),
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
    pub stats: &'a DatasetStats,
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
            format!("LOTUS Data — {} search in {effective_taxon}", title_case(st)),
            format!(
                "Chemical compounds from {effective_taxon}. Retrieved via LOTUS \
                 Wikidata Explorer with {st} chemical search (SACHEM/IDSM)."
            ),
        )
    } else {
        (
            format!("LOTUS Data — {effective_taxon}"),
            format!("Chemical compounds from {effective_taxon}. Retrieved via LOTUS Wikidata Explorer."),
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
            c.get("search_type").cloned().unwrap_or(json!("substructure")),
        );
        sq.insert(
            "input_format".into(),
            Value::String(if multiline { "molfile".into() } else { "smiles".into() }),
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
    meta.insert("@context".into(), Value::String("https://schema.org/".into()));
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
            { "@type": "DataDownload", "encodingFormat": "application/json", "contentUrl": "data:application/json" },
            { "@type": "DataDownload", "encodingFormat": "text/turtle",      "contentUrl": "data:text/turtle" },
        ]),
    );
    meta.insert(
        "numberOfRecords".into(),
        Value::Number(inp.stats.n_entries.into()),
    );
    meta.insert(
        "variablesMeasured".into(),
        json!([
            "compound_name", "compound_smiles", "compound_inchikey", "compound_mass",
            "molecular_formula", "taxon_name", "reference_title", "reference_doi",
            "reference_date", "compound_qid", "taxon_qid", "reference_qid",
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

// ── NDJSON export ─────────────────────────────────────────────────────────────

pub fn build_ndjson(rows: &[CompoundEntry]) -> String {
    let mut out = String::with_capacity(rows.len() * 256);
    for e in rows {
        let v = json!({
            "compound_qid":     e.compound_qid,
            "compound_name":    e.name,
            "compound_inchikey":e.inchikey,
            "compound_smiles":  e.smiles,
            "compound_mass":    e.mass,
            "molecular_formula":e.formula,
            "taxon_qid":        e.taxon_qid,
            "taxon_name":       e.taxon_name,
            "reference_qid":    e.reference_qid,
            "reference_title":  e.ref_title,
            "reference_doi":    e.ref_doi,
            "reference_year":   e.pub_year,
            "statement_id":     e.statement_id(),
        });
        out.push_str(&serde_json::to_string(&v).unwrap_or_default());
        out.push('\n');
    }
    out
}

// ── Turtle (TTL) export ───────────────────────────────────────────────────────

/// Emit a compact, self-contained Turtle document with:
/// * a Schema.org `Dataset` header describing the result set and query,
/// * one block of `wdt:`-qualified triples per compound / taxon / reference,
/// * `wd:Qnnn p:P703 wds:… ; ps:P703 wd:Qmmm ; prov:wasDerivedFrom wd:Qrrr`
///   statements linking the three entities.
pub fn build_ttl(rows: &[CompoundEntry], meta: MetadataInputs<'_>) -> String {
    let mut out = String::with_capacity(rows.len() * 512 + 2048);
    out.push_str(concat!(
        "@prefix wd:      <http://www.wikidata.org/entity/> .\n",
        "@prefix wds:     <http://www.wikidata.org/entity/statement/> .\n",
        "@prefix wdt:     <http://www.wikidata.org/prop/direct/> .\n",
        "@prefix p:       <http://www.wikidata.org/prop/> .\n",
        "@prefix ps:      <http://www.wikidata.org/prop/statement/> .\n",
        "@prefix prov:    <http://www.w3.org/ns/prov#> .\n",
        "@prefix schema:  <http://schema.org/> .\n",
        "@prefix dcterms: <http://purl.org/dc/terms/> .\n",
        "@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .\n\n",
    ));

    let dataset_uri = format!("urn:hash:sha256:{}", meta.result_hash);
    out.push_str(&format!("<{dataset_uri}> a schema:Dataset ;\n"));
    out.push_str(&format!(
        "    schema:name \"LOTUS Wikidata Explorer query result\" ;\n"
    ));
    out.push_str(&format!(
        "    schema:version \"{APP_VERSION}\" ;\n"
    ));
    out.push_str(&format!(
        "    schema:dateCreated \"{}\"^^xsd:dateTime ;\n",
        now_iso8601()
    ));
    out.push_str("    schema:license <https://creativecommons.org/publicdomain/zero/1.0/> ;\n");
    out.push_str(&format!(
        "    schema:numberOfRecords \"{}\"^^xsd:integer ;\n",
        meta.stats.n_entries
    ));
    out.push_str(&format!(
        "    dcterms:identifier \"sha256:{}\" .\n\n",
        meta.result_hash
    ));

    // Dedup metadata triples per entity to keep the file compact.
    use std::collections::HashSet;
    let mut seen_c: HashSet<&str> = HashSet::new();
    let mut seen_t: HashSet<&str> = HashSet::new();
    let mut seen_r: HashSet<&str> = HashSet::new();

    for e in rows {
        if seen_c.insert(e.compound_qid.as_str()) {
            out.push_str(&format!("wd:{} ", e.compound_qid));
            let mut props: Vec<String> = Vec::new();
            if !e.name.trim().is_empty() {
                props.push(format!(
                    "    schema:name {}",
                    ttl_literal(&e.name)
                ));
            }
            if let Some(ik) = &e.inchikey {
                props.push(format!("    wdt:P235 {}", ttl_literal(ik)));
            }
            if let Some(sm) = &e.smiles {
                props.push(format!("    wdt:P233 {}", ttl_literal(sm)));
            }
            if let Some(m) = e.mass {
                props.push(format!("    wdt:P2067 \"{m}\"^^xsd:decimal"));
            }
            if let Some(f) = &e.formula {
                props.push(format!("    wdt:P274 {}", ttl_literal(f)));
            }
            if props.is_empty() {
                out.push_str("a schema:ChemicalSubstance .\n");
            } else {
                out.push_str("a schema:ChemicalSubstance ;\n");
                out.push_str(&props.join(" ;\n"));
                out.push_str(" .\n");
            }
        }

        if seen_t.insert(e.taxon_qid.as_str()) && !e.taxon_qid.is_empty() {
            out.push_str(&format!(
                "wd:{} a schema:Taxon ; wdt:P225 {} .\n",
                e.taxon_qid,
                ttl_literal(&e.taxon_name)
            ));
        }

        if seen_r.insert(e.reference_qid.as_str()) && !e.reference_qid.is_empty() {
            out.push_str(&format!("wd:{} a schema:ScholarlyArticle", e.reference_qid));
            if let Some(t) = &e.ref_title {
                out.push_str(&format!(" ; wdt:P1476 {}", ttl_literal(t)));
            }
            if let Some(d) = &e.ref_doi {
                out.push_str(&format!(" ; wdt:P356 {}", ttl_literal(d)));
            }
            if let Some(y) = e.pub_year {
                out.push_str(&format!(
                    " ; wdt:P577 \"{y:04}-01-01\"^^xsd:date"
                ));
            }
            out.push_str(" .\n");
        }

        // Statement link (p:/ps:/prov:).
        if let Some(stmt) = e.statement_id() {
            out.push_str(&format!(
                "wd:{c} p:P703 wds:{s} .\nwds:{s} ps:P703 wd:{t} ; prov:wasDerivedFrom wd:{r} .\n",
                c = e.compound_qid,
                s = stmt,
                t = e.taxon_qid,
                r = e.reference_qid,
            ));
        }
    }

    out
}

fn ttl_literal(s: &str) -> String {
    let escaped = s
        .replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
        .replace('\r', r"\r")
        .replace('\t', r"\t");
    format!("\"{escaped}\"")
}

// ── `data:` URLs for browser download ─────────────────────────────────────────

pub fn to_data_url(mime: &str, body: &str) -> String {
    format!("data:{mime};charset=utf-8,{}", urlencoding::encode(body))
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

/// Normalise a taxon string into a filesystem-safe slug, matching the
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

/// Build a data-download filename matching the Python convention:
/// `{YYYYMMDD}_lotus_{safe_taxon}[_{search_type}].{ext}`.
///
/// `search_type` should be `Some("substructure")` / `Some("similarity")`
/// when a chemical-structure search is active, and `None` otherwise.
pub fn generate_filename(taxon: &str, ext: &str, search_type: Option<&str>) -> String {
    let date = today_yyyymmdd();
    let safe = safe_taxon_slug(taxon);
    match search_type {
        Some(st) if !st.is_empty() => format!("{date}_lotus_{safe}_{st}.{ext}"),
        _ => format!("{date}_lotus_{safe}.{ext}"),
    }
}


