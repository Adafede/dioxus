//! SPARQL execution and CSV parsing for the LOTUS explorer.
//!
//! Delegates URL construction and low-level helpers to the `shared` crate.

use shared::sparql::{
    FetchError, QLEVER_WIKIDATA, clean_doi, coalesce, col_idx, execute_sparql as shared_execute,
    extract_qid, field, non_empty, parse_year,
};

use crate::models::{CompoundEntry, DatasetStats, TaxonMatch};
use std::collections::HashSet;
use std::num::Wrapping;

fn owned_or_empty(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        s.to_owned()
    }
}

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

fn fnv1a_extend(mut h: Wrapping<u64>, bytes: &[u8]) -> Wrapping<u64> {
    const FNV_PRIME: Wrapping<u64> = Wrapping(1099511628211);
    for b in bytes {
        h ^= Wrapping(*b as u64);
        h *= FNV_PRIME;
    }
    h
}

fn entry_key_fingerprint(compound_qid: &str, taxon_qid: &str, reference_qid: &str) -> u64 {
    // Fast 64-bit fingerprint for dedup keys.
    let mut h = Wrapping(14695981039346656037u64); // FNV offset basis
    h = fnv1a_extend(h, compound_qid.as_bytes());
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, taxon_qid.as_bytes());
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, reference_qid.as_bytes());
    h.0
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
#[allow(dead_code)]
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
    let c_compound = col_idx(&headers, "compound");
    let c_label = col_idx(&headers, "compoundLabel");
    let c_inchikey = col_idx(&headers, "compound_inchikey");
    let c_smiles_iso = col_idx(&headers, "compound_smiles_iso");
    let c_smiles_con = col_idx(&headers, "compound_smiles_conn");
    let c_mass = col_idx(&headers, "compound_mass");
    let c_formula = col_idx(&headers, "compound_formula");
    let c_taxon = col_idx(&headers, "taxon");
    let c_taxon_name = col_idx(&headers, "taxon_name");
    let c_ref_qid = col_idx(&headers, "ref_qid");
    let c_ref_title = col_idx(&headers, "ref_title");
    let c_ref_doi = col_idx(&headers, "ref_doi");
    let c_ref_date = col_idx(&headers, "ref_date");
    let c_statement = col_idx(&headers, "statement");

    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(1024);
    let mut seen: HashSet<u64> = HashSet::with_capacity(2048);

    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;

        let compound_qid = parse_entity_id(field(&rec, c_compound));
        if compound_qid.is_empty() {
            continue;
        }

        let taxon_qid = parse_entity_id(field(&rec, c_taxon));
        let reference_qid = parse_entity_id(field(&rec, c_ref_qid));

        // Deduplicate: same compound × taxon × reference = one row
        let key = entry_key_fingerprint(&compound_qid, &taxon_qid, &reference_qid);
        if !seen.insert(key) {
            continue;
        }

        entries.push(CompoundEntry {
            compound_qid,
            name: owned_or_empty(field(&rec, c_label)),
            inchikey: non_empty(field(&rec, c_inchikey)),
            // Prefer isomeric SMILES, fall back to connectivity SMILES (same as Python)
            smiles: coalesce(field(&rec, c_smiles_iso), field(&rec, c_smiles_con)),
            mass: field(&rec, c_mass).parse::<f64>().ok(),
            formula: non_empty(field(&rec, c_formula)),
            taxon_qid,
            taxon_name: owned_or_empty(field(&rec, c_taxon_name)),
            reference_qid,
            ref_title: non_empty(field(&rec, c_ref_title)),
            ref_doi: clean_doi(field(&rec, c_ref_doi)),
            pub_year: parse_year(field(&rec, c_ref_date)),
            statement: non_empty(field(&rec, c_statement)),
        });
    }

    Ok(entries)
}

/// Parse QLever CSV output into `CompoundEntry` rows, materializing at most
/// `max_rows` unique entries.
///
/// Returns `(rows, total_distinct, was_capped)`.
pub fn parse_compounds_csv_capped(
    csv_text: &str,
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();

    let c_compound = col_idx(&headers, "compound");
    let c_label = col_idx(&headers, "compoundLabel");
    let c_inchikey = col_idx(&headers, "compound_inchikey");
    let c_smiles_iso = col_idx(&headers, "compound_smiles_iso");
    let c_smiles_con = col_idx(&headers, "compound_smiles_conn");
    let c_mass = col_idx(&headers, "compound_mass");
    let c_formula = col_idx(&headers, "compound_formula");
    let c_taxon = col_idx(&headers, "taxon");
    let c_taxon_name = col_idx(&headers, "taxon_name");
    let c_ref_qid = col_idx(&headers, "ref_qid");
    let c_ref_title = col_idx(&headers, "ref_title");
    let c_ref_doi = col_idx(&headers, "ref_doi");
    let c_ref_date = col_idx(&headers, "ref_date");
    let c_statement = col_idx(&headers, "statement");

    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(max_rows.min(2048));
    let mut seen: HashSet<u64> = HashSet::with_capacity(max_rows.saturating_mul(4));
    let mut total_distinct = 0usize;
    let mut compounds: HashSet<String> = HashSet::new();
    let mut taxa: HashSet<String> = HashSet::new();
    let mut references: HashSet<String> = HashSet::new();

    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;

        let compound_qid = parse_entity_id(field(&rec, c_compound));
        if compound_qid.is_empty() {
            continue;
        }

        let taxon_qid = parse_entity_id(field(&rec, c_taxon));
        let reference_qid = parse_entity_id(field(&rec, c_ref_qid));

        let key = entry_key_fingerprint(&compound_qid, &taxon_qid, &reference_qid);
        if !seen.insert(key) {
            continue;
        }

        total_distinct += 1;
        compounds.insert(compound_qid.clone());
        if !taxon_qid.is_empty() {
            taxa.insert(taxon_qid.clone());
        }
        if !reference_qid.is_empty() {
            references.insert(reference_qid.clone());
        }

        if entries.len() >= max_rows {
            // Keep scanning for exact total_distinct, but skip extra row materialization.
            continue;
        }

        entries.push(CompoundEntry {
            compound_qid,
            name: owned_or_empty(field(&rec, c_label)),
            inchikey: non_empty(field(&rec, c_inchikey)),
            smiles: coalesce(field(&rec, c_smiles_iso), field(&rec, c_smiles_con)),
            mass: field(&rec, c_mass).parse::<f64>().ok(),
            formula: non_empty(field(&rec, c_formula)),
            taxon_qid,
            taxon_name: owned_or_empty(field(&rec, c_taxon_name)),
            reference_qid,
            ref_title: non_empty(field(&rec, c_ref_title)),
            ref_doi: clean_doi(field(&rec, c_ref_doi)),
            pub_year: parse_year(field(&rec, c_ref_date)),
            statement: non_empty(field(&rec, c_statement)),
        });
    }

    let stats = DatasetStats {
        n_compounds: compounds.len(),
        n_taxa: taxa.len(),
        n_references: references.len(),
        n_entries: total_distinct,
    };
    let was_capped = total_distinct > max_rows;
    Ok((entries, stats, was_capped))
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
    let c_name = col_idx(&headers, "taxon_name");

    let mut matches = Vec::new();
    for result in rdr.records() {
        let rec = result.map_err(|e| FetchError::Parse(e.to_string()))?;
        let qid = parse_entity_id(field(&rec, c_taxon));
        let name = field(&rec, c_name).to_string();
        if !qid.is_empty() && !name.is_empty() {
            matches.push(TaxonMatch { qid, name });
        }
    }
    Ok(matches)
}

// ...no separate count-query parser needed; totals are computed in-stream.
