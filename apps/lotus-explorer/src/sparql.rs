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

// Single-entry "most recent query" CSV cache. Clicking CSV → JSON → TTL on the
// same result set would otherwise re-fetch the full dataset three times. The
// cache is cleared implicitly on the next distinct query.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static CSV_CACHE: std::cell::RefCell<Option<(u64, std::rc::Rc<String>)>>
        = const { std::cell::RefCell::new(None) };
}

/// Execute a SPARQL query, reusing the CSV body from the previous call when
/// the query text is identical (same FNV-1a fingerprint).
pub async fn execute_sparql_cached(sparql: &str) -> Result<std::rc::Rc<String>, FetchError> {
    #[cfg(target_arch = "wasm32")]
    {
        let key = fnv1a_one(sparql.as_bytes());
        if let Some(hit) = CSV_CACHE.with(|c| {
            c.borrow()
                .as_ref()
                .and_then(|(k, v)| (*k == key).then(|| v.clone()))
        }) {
            return Ok(hit);
        }
        let body = shared_execute(sparql, QLEVER_WIKIDATA).await?;
        let rc = std::rc::Rc::new(body);
        CSV_CACHE.with(|c| *c.borrow_mut() = Some((key, rc.clone())));
        Ok(rc)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let body = shared_execute(sparql, QLEVER_WIKIDATA).await?;
        Ok(std::rc::Rc::new(body))
    }
}

// ── Compound CSV parser ───────────────────────────────────────────────────────

/// Parse a full QLever CSV response into `CompoundEntry` rows.
pub fn parse_compounds_csv(csv_text: &str) -> Result<Vec<CompoundEntry>, FetchError> {
    let (rows, _stats, _capped) = parse_compounds_csv_capped(csv_text, usize::MAX)?;
    Ok(rows)
}

/// Parse aggregate count CSV with columns:
/// `n_entries`, `n_compounds`, `n_taxa`, `n_references`.
pub fn parse_counts_csv(csv_text: &str) -> Result<DatasetStats, FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

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

/// Fast single-pass parser over the QLever CSV stream.
///
/// * **Every** data row contributes to `DatasetStats` and to the exact
///   `n_entries` count, so metadata / download totals are always accurate.
/// * Only the first `max_rows` *distinct* `(compound, taxon, reference)`
///   triples are materialized into full `CompoundEntry` structs. Rows past
///   that cap only touch their three QID columns (for dedup + stat
///   fingerprinting) — no allocation for names, SMILES, titles, DOIs or
///   dates, which is where the old parser was spending its time on very
///   large result sets.
///
/// Returns `(display_rows, full_stats, was_capped)`.
pub fn parse_compounds_csv_capped(
    csv_text: &str,
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

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
            name: owned_or_empty(label),
            inchikey: non_empty(inchikey),
            smiles: coalesce(smiles_iso, smiles_con),
            mass: mass.parse::<f64>().ok(),
            formula: non_empty(formula),
            taxon_qid: taxon_qid.clone(),
            taxon_name: owned_or_empty(taxon_name),
            reference_qid: reference_qid.clone(),
            ref_title: non_empty(ref_title),
            ref_doi: clean_doi(ref_doi),
            pub_year: parse_year(ref_date),
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
fn byte_field_str<'a>(rec: &'a csv::ByteRecord, idx: Option<usize>) -> &'a str {
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
