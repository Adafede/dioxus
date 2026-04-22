//! SPARQL execution and CSV parsing for the LOTUS explorer.
//!
//! Delegates URL construction and low-level helpers to the `shared` crate.

use shared::sparql::{
    clean_doi, coalesce, col_idx, execute_sparql as shared_execute, extract_qid, field, non_empty,
    parse_year, FetchError, QLEVER_WIKIDATA,
};

use crate::models::{CompoundEntry, TaxonMatch};
use std::collections::HashSet;

fn parse_entity_id(value: &str) -> String {
    let qid = extract_qid(value);
    if !qid.is_empty() {
        return qid;
    }

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Handle typed literals, e.g. "134630"^^<http://www.w3.org/2001/XMLSchema#integer>
    let lexical = trimmed
        .split("^^")
        .next()
        .unwrap_or(trimmed)
        .trim()
        .trim_matches('"');

    if let Some(rest) = lexical.strip_prefix('Q') {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return lexical.to_string();
        }
    }

    if !lexical.is_empty() && lexical.chars().all(|c| c.is_ascii_digit()) {
        return format!("Q{lexical}");
    }

    String::new()
}


// ── SPARQL execution ──────────────────────────────────────────────────────────

/// Execute a SPARQL query against the QLever Wikidata endpoint.
pub async fn execute_sparql(sparql: &str) -> Result<String, FetchError> {
    shared_execute(sparql, QLEVER_WIKIDATA).await
}

// ── Compound CSV parser ───────────────────────────────────────────────────────

/// Parse QLever CSV output into `CompoundEntry` rows.
///
/// Column names must match exactly what the queries in `queries.rs` project:
/// `compound`, `compoundLabel`, `compound_inchikey`, `compound_smiles_iso`,
/// `compound_smiles_conn`, `compound_mass`, `compound_formula`,
/// `taxon`, `taxon_name`, `ref_qid`, `ref_title`, `ref_doi`, `ref_date`, `statement`.
pub fn parse_compounds_csv(csv_text: &str) -> Result<Vec<CompoundEntry>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();

    // Column indices — safe to pre-compute once
    let c_compound   = col_idx(&headers, "compound");
    let c_label      = col_idx(&headers, "compoundLabel");
    let c_inchikey   = col_idx(&headers, "compound_inchikey");
    let c_smiles_iso = col_idx(&headers, "compound_smiles_iso");
    let c_smiles_con = col_idx(&headers, "compound_smiles_conn");
    let c_mass       = col_idx(&headers, "compound_mass");
    let c_formula    = col_idx(&headers, "compound_formula");
    let c_taxon      = col_idx(&headers, "taxon");
    let c_taxon_name = col_idx(&headers, "taxon_name");
    let c_ref_qid    = col_idx(&headers, "ref_qid");
    let c_ref_title  = col_idx(&headers, "ref_title");
    let c_ref_doi    = col_idx(&headers, "ref_doi");
    let c_ref_date   = col_idx(&headers, "ref_date");
    let c_statement  = col_idx(&headers, "statement");

    let mut entries: Vec<CompoundEntry> = Vec::new();
    let mut seen: HashSet<(String, String, String)> = HashSet::new();

    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;

        let compound_qid = parse_entity_id(field(&rec, c_compound));
        if compound_qid.is_empty() {
            continue;
        }

        let taxon_qid     = parse_entity_id(field(&rec, c_taxon));
        let reference_qid = parse_entity_id(field(&rec, c_ref_qid));

        // Deduplicate: same compound × taxon × reference = one row
        let key = (compound_qid.clone(), taxon_qid.clone(), reference_qid.clone());
        if !seen.insert(key) {
            continue;
        }

        entries.push(CompoundEntry {
            compound_qid,
            name:          field(&rec, c_label).to_string(),
            inchikey:      non_empty(field(&rec, c_inchikey)),
            // Prefer isomeric SMILES, fall back to connectivity SMILES (same as Python)
            smiles:        coalesce(field(&rec, c_smiles_iso), field(&rec, c_smiles_con)),
            mass:          field(&rec, c_mass).parse::<f64>().ok(),
            formula:       non_empty(field(&rec, c_formula)),
            taxon_qid,
            taxon_name:    field(&rec, c_taxon_name).to_string(),
            reference_qid,
            ref_title:     non_empty(field(&rec, c_ref_title)),
            ref_doi:       clean_doi(field(&rec, c_ref_doi)),
            pub_year:      parse_year(field(&rec, c_ref_date)),
            statement:     non_empty(field(&rec, c_statement)),
        });
    }

    Ok(entries)
}

// ── Taxon CSV parser ──────────────────────────────────────────────────────────

/// Parse taxon search CSV into `TaxonMatch` rows.
/// Columns: `taxon` (full URI or bare QID), `taxon_name`.
pub fn parse_taxon_csv(csv_text: &str) -> Result<Vec<TaxonMatch>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();

    let c_taxon = col_idx(&headers, "taxon");
    let c_name  = col_idx(&headers, "taxon_name");

    let mut matches = Vec::new();
    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;
        let qid  = parse_entity_id(field(&rec, c_taxon));
        let name = field(&rec, c_name).to_string();
        if !qid.is_empty() && !name.is_empty() {
            matches.push(TaxonMatch { qid, name });
        }
    }
    Ok(matches)
}
