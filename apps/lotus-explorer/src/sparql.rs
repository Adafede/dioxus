//! SPARQL execution and CSV parsing for the LOTUS explorer.
//!
//! Delegates URL construction and low-level helpers to the `shared` crate.

use shared::sparql::{
    FetchError, QLEVER_WIKIDATA, SparqlResponseFormat, clean_doi, coalesce, col_idx,
    execute_sparql as shared_execute, execute_sparql_bytes as shared_execute_bytes,
    execute_sparql_with_format as shared_execute_with_format, extract_qid, field, non_empty,
    parse_year,
};

use crate::models::{CompoundEntry, DatasetStats, TaxonMatch};
use std::collections::{HashMap, HashSet};
use std::num::Wrapping;
use std::sync::Arc;

#[inline]
fn arc_or_empty(s: &str) -> Arc<str> {
    if s.is_empty() {
        Arc::<str>::from("")
    } else {
        Arc::<str>::from(s)
    }
}

#[inline]
fn arc_non_empty(s: &str) -> Option<Arc<str>> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(Arc::<str>::from(t))
    }
}

#[derive(Default)]
struct StrInterner {
    map: HashMap<Box<str>, Arc<str>>,
}

impl StrInterner {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    fn intern_or_empty(&mut self, value: &str) -> Arc<str> {
        let v = value.trim();
        if v.is_empty() {
            return Arc::<str>::from("");
        }
        if let Some(existing) = self.map.get(v) {
            return existing.clone();
        }
        let arc = Arc::<str>::from(v);
        self.map.insert(v.to_owned().into_boxed_str(), arc.clone());
        arc
    }

    fn intern_optional(&mut self, value: &str) -> Option<Arc<str>> {
        let v = value.trim();
        if v.is_empty() {
            None
        } else {
            Some(self.intern_or_empty(v))
        }
    }
}

#[inline]
fn fnv1a_extend(mut h: Wrapping<u64>, bytes: &[u8]) -> Wrapping<u64> {
    const FNV_PRIME: Wrapping<u64> = Wrapping(1099511628211);
    for b in bytes {
        h ^= Wrapping(*b as u64);
        h *= FNV_PRIME;
    }
    h
}

#[inline]
fn fnv1a_one(bytes: &[u8]) -> u64 {
    fnv1a_extend(Wrapping(14695981039346656037u64), bytes).0
}

fn entry_key_fingerprint(compound_qid: &[u8], taxon_qid: &[u8], reference_qid: &[u8]) -> u64 {
    let mut h = Wrapping(14695981039346656037u64);
    h = fnv1a_extend(h, compound_qid);
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, taxon_qid);
    h = fnv1a_extend(h, &[0x1f]);
    h = fnv1a_extend(h, reference_qid);
    h.0
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

    let lexical = trimmed
        .split("^^")
        .next()
        .unwrap_or(trimmed)
        .trim()
        .trim_matches('"');

    if let Some(rest) = lexical.strip_prefix('Q') {
        if !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()) {
            return lexical.to_string();
        }
    }

    if !lexical.is_empty() && lexical.bytes().all(|b| b.is_ascii_digit()) {
        return format!("Q{lexical}");
    }

    String::new()
}

// ── SPARQL execution ──────────────────────────────────────────────────────────

/// Execute a SPARQL query against the QLever Wikidata endpoint.
///
/// QLever honors `Accept-Encoding: gzip`; browsers add that header
/// automatically for `fetch`, so on wasm the CSV body is transparently
/// gzip-compressed over the wire (typically 5–10× smaller than the
/// uncompressed payload) and transparently decompressed before it reaches
/// this code. No extra work required on our side.
pub async fn execute_sparql(sparql: &str) -> Result<String, FetchError> {
    shared_execute(sparql, QLEVER_WIKIDATA).await
}

pub async fn execute_sparql_bytes(sparql: &str) -> Result<Vec<u8>, FetchError> {
    shared_execute_bytes(sparql, QLEVER_WIKIDATA).await
}

pub async fn execute_sparql_format(
    sparql: &str,
    format: SparqlResponseFormat,
) -> Result<String, FetchError> {
    shared_execute_with_format(sparql, QLEVER_WIKIDATA, format).await
}

// ── Compound CSV parser ───────────────────────────────────────────────────────

pub fn parse_compounds_csv_display_bytes(
    csv_bytes: &[u8],
    max_rows: usize,
) -> Result<Vec<CompoundEntry>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .byte_headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let find = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name.as_bytes()) };

    let c_compound = find("compound");
    let c_label = find("compoundLabel");
    let c_inchikey = find("compound_inchikey");
    let c_smiles_iso = find("compound_smiles_iso");
    let c_smiles_con = find("compound_smiles_conn");
    let c_mass = find("compound_mass");
    let c_formula = find("compound_formula");
    let c_taxon = find("taxon");
    let c_taxon_name = find("taxon_name");
    let c_ref_qid = find("ref_qid");
    let c_ref_title = find("ref_title");
    let c_ref_doi = find("ref_doi");
    let c_ref_date = find("ref_date");
    let c_statement = find("statement");

    let initial_cap = max_rows.min(1024);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    let mut taxon_name_intern = StrInterner::with_capacity(64);
    let mut ref_title_intern = StrInterner::with_capacity(128);
    let mut compound_qid = String::new();
    let mut taxon_qid = String::new();
    let mut reference_qid = String::new();

    let mut rec = csv::ByteRecord::new();
    while entries.len() < max_rows
        && rdr
            .read_byte_record(&mut rec)
            .map_err(|e| FetchError::Parse(e.to_string()))?
    {
        compound_qid.clear();
        if let Some(i) = c_compound {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut compound_qid, b);
            }
        }
        if compound_qid.is_empty() {
            continue;
        }
        taxon_qid.clear();
        if let Some(i) = c_taxon {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut taxon_qid, b);
            }
        }
        reference_qid.clear();
        if let Some(i) = c_ref_qid {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut reference_qid, b);
            }
        }

        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        let label = byte_field_str(&rec, c_label);
        let inchikey = byte_field_str(&rec, c_inchikey);
        let smiles_iso = byte_field_str(&rec, c_smiles_iso);
        let smiles_con = byte_field_str(&rec, c_smiles_con);
        let mass = byte_field_str(&rec, c_mass);
        let formula = byte_field_str(&rec, c_formula);
        let taxon_name = byte_field_str(&rec, c_taxon_name);
        let ref_title = byte_field_str(&rec, c_ref_title);
        let ref_doi = byte_field_str(&rec, c_ref_doi);
        let ref_date = byte_field_str(&rec, c_ref_date);
        let statement = byte_field_str(&rec, c_statement);

        entries.push(CompoundEntry {
            compound_qid: compound_qid.clone(),
            name: arc_or_empty(label),
            inchikey: non_empty(inchikey),
            smiles: coalesce(smiles_iso, smiles_con),
            mass: mass.parse::<f64>().ok(),
            formula: arc_non_empty(formula),
            taxon_qid: taxon_qid.clone(),
            taxon_name: taxon_name_intern.intern_or_empty(taxon_name),
            reference_qid: reference_qid.clone(),
            ref_title: ref_title_intern.intern_optional(ref_title),
            ref_doi: clean_doi(ref_doi),
            pub_year: parse_year(ref_date).and_then(|y| i16::try_from(y).ok()),
            statement: non_empty(statement),
        });
    }

    Ok(entries)
}

pub fn parse_counts_csv_bytes(csv_bytes: &[u8]) -> Result<DatasetStats, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let c_entries = col_idx(&headers, "n_entries");
    let c_compounds = col_idx(&headers, "n_compounds");
    let c_taxa = col_idx(&headers, "n_taxa");
    let c_refs = col_idx(&headers, "n_references");

    let mut records = rdr.records();
    let rec = match records.next() {
        Some(Ok(r)) => r,
        Some(Err(e)) => return Err(FetchError::Parse(e.to_string())),
        None => return Err(FetchError::Parse("Missing count row".to_string())),
    };

    let parse_num =
        |idx: Option<usize>| -> usize { field(&rec, idx).parse::<usize>().unwrap_or(0) };

    Ok(DatasetStats {
        n_entries: parse_num(c_entries),
        n_compounds: parse_num(c_compounds),
        n_taxa: parse_num(c_taxa),
        n_references: parse_num(c_refs),
    })
}

pub fn parse_compounds_csv_capped_bytes(
    csv_bytes: &[u8],
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = rdr
        .byte_headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();

    let find = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name.as_bytes()) };

    let c_compound = find("compound");
    let c_label = find("compoundLabel");
    let c_inchikey = find("compound_inchikey");
    let c_smiles_iso = find("compound_smiles_iso");
    let c_smiles_con = find("compound_smiles_conn");
    let c_mass = find("compound_mass");
    let c_formula = find("compound_formula");
    let c_taxon = find("taxon");
    let c_taxon_name = find("taxon_name");
    let c_ref_qid = find("ref_qid");
    let c_ref_title = find("ref_title");
    let c_ref_doi = find("ref_doi");
    let c_ref_date = find("ref_date");
    let c_statement = find("statement");

    let initial_cap = max_rows.min(2048);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    // Dedup-by-triple using 64-bit FNV fingerprints — no String allocations.
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    // Per-entity fingerprint sets for accurate stats without storing strings.
    let mut compound_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut taxon_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut ref_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut total_distinct = 0usize;
    let mut taxon_name_intern = StrInterner::with_capacity(64);
    let mut ref_title_intern = StrInterner::with_capacity(128);

    // Scratch buffers reused every row — avoids three `String` allocations per
    // overflow row (the hot path for huge taxa).
    let mut compound_qid = String::new();
    let mut taxon_qid = String::new();
    let mut reference_qid = String::new();

    let mut rec = csv::ByteRecord::new();
    while rdr
        .read_byte_record(&mut rec)
        .map_err(|e| FetchError::Parse(e.to_string()))?
    {
        compound_qid.clear();
        if let Some(i) = c_compound {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut compound_qid, b);
            }
        }
        if compound_qid.is_empty() {
            continue;
        }
        taxon_qid.clear();
        if let Some(i) = c_taxon {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut taxon_qid, b);
            }
        }
        reference_qid.clear();
        if let Some(i) = c_ref_qid {
            if let Some(b) = rec.get(i) {
                fill_qid(&mut reference_qid, b);
            }
        }

        // Dedup by (compound, taxon, ref) triple.
        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        // ── Stats (always, every row) ──
        total_distinct += 1;
        compound_fps.insert(fnv1a_one(compound_qid.as_bytes()));
        if !taxon_qid.is_empty() {
            taxon_fps.insert(fnv1a_one(taxon_qid.as_bytes()));
        }
        if !reference_qid.is_empty() {
            ref_fps.insert(fnv1a_one(reference_qid.as_bytes()));
        }

        // ── Past the display cap? Skip heavy string work. ──
        if entries.len() >= max_rows {
            continue;
        }

        // Materialize the full entry — only touches the remaining fields for
        // rows that will actually be rendered.
        let label = byte_field_str(&rec, c_label);
        let inchikey = byte_field_str(&rec, c_inchikey);
        let smiles_iso = byte_field_str(&rec, c_smiles_iso);
        let smiles_con = byte_field_str(&rec, c_smiles_con);
        let mass = byte_field_str(&rec, c_mass);
        let formula = byte_field_str(&rec, c_formula);
        let taxon_name = byte_field_str(&rec, c_taxon_name);
        let ref_title = byte_field_str(&rec, c_ref_title);
        let ref_doi = byte_field_str(&rec, c_ref_doi);
        let ref_date = byte_field_str(&rec, c_ref_date);
        let statement = byte_field_str(&rec, c_statement);

        entries.push(CompoundEntry {
            compound_qid: compound_qid.clone(),
            name: arc_or_empty(label),
            inchikey: non_empty(inchikey),
            smiles: coalesce(smiles_iso, smiles_con),
            mass: mass.parse::<f64>().ok(),
            formula: arc_non_empty(formula),
            taxon_qid: taxon_qid.clone(),
            taxon_name: taxon_name_intern.intern_or_empty(taxon_name),
            reference_qid: reference_qid.clone(),
            ref_title: ref_title_intern.intern_optional(ref_title),
            ref_doi: clean_doi(ref_doi),
            pub_year: parse_year(ref_date).and_then(|y| i16::try_from(y).ok()),
            statement: non_empty(statement),
        });
    }

    let stats = DatasetStats {
        n_compounds: compound_fps.len(),
        n_taxa: taxon_fps.len(),
        n_references: ref_fps.len(),
        n_entries: total_distinct,
    };
    let was_capped = total_distinct > entries.len();
    Ok((entries, stats, was_capped))
}

/// Decode a byte field as a trimmed UTF-8 string slice. QLever always emits
/// UTF-8; on malformed input we fall back to `""`.
#[inline]
fn byte_field_str(rec: &csv::ByteRecord, idx: Option<usize>) -> &str {
    match idx.and_then(|i| rec.get(i)) {
        Some(bytes) => std::str::from_utf8(bytes).unwrap_or("").trim(),
        None => "",
    }
}

/// Parse a Wikidata entity column into `out` without extra allocations,
/// handling all three shapes QLever can emit:
///  1. Full URI: `http://www.wikidata.org/entity/Q12345`
///  2. Bare QID: `Q12345`
///  3. Typed integer literal: `"134630"^^<…#integer>` (we prefix `Q`).
fn fill_qid(out: &mut String, bytes: &[u8]) {
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };
    if s.is_empty() {
        return;
    }

    // Full URI — only accept when the suffix really is Q<digits>.
    if let Some(idx) = s.rfind("wikidata.org/entity/") {
        let rest = &s[idx + "wikidata.org/entity/".len()..];
        if rest.len() >= 2
            && rest.as_bytes()[0] == b'Q'
            && rest.bytes().skip(1).all(|b| b.is_ascii_digit())
        {
            out.push_str(rest);
            return;
        }
    }

    // Typed-literal lexical form, e.g. `"134630"^^<...#integer>`.
    let lexical = s.split("^^").next().unwrap_or(s).trim().trim_matches('"');

    if lexical.is_empty() {
        return;
    }

    if lexical.as_bytes().first() == Some(&b'Q')
        && lexical.len() >= 2
        && lexical[1..].bytes().all(|b| b.is_ascii_digit())
    {
        out.push_str(lexical);
        return;
    }

    if lexical.bytes().all(|b| b.is_ascii_digit()) {
        out.push('Q');
        out.push_str(lexical);
    }
}

// ── Taxon CSV parser ──────────────────────────────────────────────────────────

pub fn parse_taxon_csv_bytes(csv_bytes: &[u8]) -> Result<Vec<TaxonMatch>, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

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
