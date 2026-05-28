// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use super::models::{CompoundEntry, DatasetStats, TaxonMatch};
#[cfg(not(target_arch = "wasm32"))]
use crate::sparql::execute_sparql_tempfile as shared_execute_tempfile;
use crate::sparql::{
    FetchError, QLEVER_WIKIDATA, SparqlResponseFormat, col_idx, execute_sparql as shared_execute,
    execute_sparql_body as shared_execute_body, execute_sparql_bytes as shared_execute_bytes,
    execute_sparql_with_format as shared_execute_with_format, extract_qid,
    fetch_export_url_bytes as shared_fetch_export_url_bytes, field, parse_year,
};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::num::Wrapping;
use std::sync::Arc;

const WIKIDATA_STATEMENT_PREFIX: &str = "http://www.wikidata.org/entity/statement/";

/// Column indices for the compound CSV format used by all LOTUS SPARQL queries.
struct CompoundColumns {
    compound: Option<usize>,
    label: Option<usize>,
    inchikey: Option<usize>,
    smiles_iso: Option<usize>,
    smiles_con: Option<usize>,
    mass: Option<usize>,
    formula: Option<usize>,
    taxon: Option<usize>,
    taxon_name: Option<usize>,
    ref_qid: Option<usize>,
    ref_title: Option<usize>,
    ref_doi: Option<usize>,
    ref_date: Option<usize>,
    statement: Option<usize>,
}

impl CompoundColumns {
    fn detect(headers: &csv::ByteRecord) -> Self {
        let find =
            |name: &str| -> Option<usize> { headers.iter().position(|h| h == name.as_bytes()) };
        Self {
            compound: find("compound"),
            label: find("compoundLabel"),
            inchikey: find("compound_inchikey"),
            smiles_iso: find("compound_smiles_iso"),
            smiles_con: find("compound_smiles_conn"),
            mass: find("compound_mass"),
            formula: find("compound_formula"),
            taxon: find("taxon"),
            taxon_name: find("taxon_name"),
            ref_qid: find("ref_qid"),
            ref_title: find("ref_title"),
            ref_doi: find("ref_doi"),
            ref_date: find("ref_date"),
            statement: find("statement"),
        }
    }
}

/// String interners for all `CompoundEntry` fields.
struct CompoundInterners {
    qid: StrInterner,
    label: StrInterner,
    taxon_name: StrInterner,
    ref_title: StrInterner,
    doi: StrInterner,
    inchikey: StrInterner,
    smiles: StrInterner,
    formula: StrInterner,
    statement: StrInterner,
}

impl CompoundInterners {
    fn new(cap: usize) -> Self {
        Self {
            qid: StrInterner::with_capacity(cap),
            label: StrInterner::with_capacity(cap),
            taxon_name: StrInterner::with_capacity(64),
            ref_title: StrInterner::with_capacity(128),
            doi: StrInterner::with_capacity(cap / 2),
            inchikey: StrInterner::with_capacity(cap),
            smiles: StrInterner::with_capacity(cap * 2),
            formula: StrInterner::with_capacity(cap),
            statement: StrInterner::with_capacity(cap),
        }
    }

    fn build_entry(
        &mut self,
        cols: &CompoundColumns,
        rec: &csv::ByteRecord,
        compound_qid: &str,
        taxon_qid: &str,
        reference_qid: &str,
    ) -> CompoundEntry {
        let label = byte_field_str(rec, cols.label);
        let inchikey = byte_field_str(rec, cols.inchikey);
        let smiles_iso = byte_field_str(rec, cols.smiles_iso);
        let smiles_con = byte_field_str(rec, cols.smiles_con);
        let mass_str = byte_field_str(rec, cols.mass);
        let formula = byte_field_str(rec, cols.formula);
        let taxon_name = byte_field_str(rec, cols.taxon_name);
        let ref_title = byte_field_str(rec, cols.ref_title);
        let ref_doi = byte_field_str(rec, cols.ref_doi);
        let ref_date = byte_field_str(rec, cols.ref_date);
        let statement = byte_field_str(rec, cols.statement);

        CompoundEntry {
            compound_qid: self.qid.intern_or_empty(compound_qid),
            name: self.label.intern_or_empty(label),
            inchikey: self.inchikey.intern_optional(inchikey),
            smiles: self.smiles.intern_optional(if smiles_iso.is_empty() {
                smiles_con
            } else {
                smiles_iso
            }),
            mass: mass_str.parse::<f64>().ok(),
            formula: self.formula.intern_optional(formula),
            taxon_qid: self.qid.intern_or_empty(taxon_qid),
            taxon_name: self.taxon_name.intern_or_empty(taxon_name),
            reference_qid: self.qid.intern_or_empty(reference_qid),
            ref_title: self.ref_title.intern_optional(ref_title),
            ref_doi: normalize_doi_value(ref_doi).and_then(|d| self.doi.intern_optional(d)),
            pub_year: parse_year(ref_date).and_then(|y| i16::try_from(y).ok()),
            statement: normalize_statement_value(statement)
                .and_then(|s| self.statement.intern_optional(s)),
        }
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
    const FNV_PRIME: Wrapping<u64> = Wrapping(1_099_511_628_211);
    for b in bytes {
        h ^= Wrapping(u64::from(*b));
        h *= FNV_PRIME;
    }
    h
}

#[inline]
fn fnv1a_one(bytes: &[u8]) -> u64 {
    fnv1a_extend(Wrapping(14_695_981_039_346_656_037_u64), bytes).0
}

fn entry_key_fingerprint(compound_qid: &[u8], taxon_qid: &[u8], reference_qid: &[u8]) -> u64 {
    let mut h = Wrapping(14_695_981_039_346_656_037_u64);
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

    if let Some(rest) = lexical.strip_prefix('Q')
        && !rest.is_empty()
        && rest.bytes().all(|b| b.is_ascii_digit())
    {
        return lexical.to_string();
    }

    if !lexical.is_empty() && lexical.bytes().all(|b| b.is_ascii_digit()) {
        return format!("Q{lexical}");
    }

    String::new()
}

#[inline]
fn normalize_statement_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .strip_prefix(WIKIDATA_STATEMENT_PREFIX)
            .unwrap_or(trimmed),
    )
}

#[inline]
fn normalize_doi_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.split("doi.org/").last().unwrap_or(trimmed).trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub async fn execute_sparql(sparql: &str) -> Result<String, FetchError> {
    shared_execute(sparql, QLEVER_WIKIDATA).await
}

pub async fn execute_sparql_bytes(sparql: &str) -> Result<Vec<u8>, FetchError> {
    shared_execute_bytes(sparql, QLEVER_WIKIDATA).await
}

pub async fn execute_sparql_body(sparql: &str) -> Result<crate::sparql::ResponseBody, FetchError> {
    shared_execute_body(sparql, QLEVER_WIKIDATA).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn execute_sparql_tempfile(sparql: &str) -> Result<tempfile::NamedTempFile, FetchError> {
    shared_execute_tempfile(sparql, QLEVER_WIKIDATA).await
}

pub async fn execute_sparql_format(
    sparql: &str,
    format: SparqlResponseFormat,
) -> Result<String, FetchError> {
    shared_execute_with_format(sparql, QLEVER_WIKIDATA, format).await
}

pub async fn fetch_export_url_format(
    url: &str,
    format: SparqlResponseFormat,
) -> Result<String, FetchError> {
    let bytes = shared_fetch_export_url_bytes(url, format).await?;
    String::from_utf8(bytes).map_err(|e| FetchError::Parse(e.to_string()))
}

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
    let cols = CompoundColumns::detect(&headers);

    let initial_cap = max_rows.min(1024);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    let mut interners = CompoundInterners::new(initial_cap);
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
        if let Some(i) = cols.compound
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut compound_qid, b);
        }
        if compound_qid.is_empty() {
            continue;
        }
        taxon_qid.clear();
        if let Some(i) = cols.taxon
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut taxon_qid, b);
        }
        reference_qid.clear();
        if let Some(i) = cols.ref_qid
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut reference_qid, b);
        }

        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        entries.push(interners.build_entry(&cols, &rec, &compound_qid, &taxon_qid, &reference_qid));
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
    let c_entries_unique = col_idx(&headers, "n_entries_unique");
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

    let n_entries = parse_num(c_entries);
    let n_entries_unique = parse_num(c_entries_unique);

    Ok(DatasetStats {
        n_entries,
        n_entries_unique: if n_entries_unique == 0 {
            n_entries
        } else {
            n_entries_unique
        },
        n_compounds: parse_num(c_compounds),
        n_taxa: parse_num(c_taxa),
        n_references: parse_num(c_refs),
    })
}

pub fn parse_compounds_csv_capped_bytes(
    csv_bytes: &[u8],
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    parse_compounds_csv_capped_reader(csv_bytes, max_rows)
}

pub fn parse_compounds_csv_capped_reader<R: Read>(
    reader: R,
    max_rows: usize,
) -> Result<(Vec<CompoundEntry>, DatasetStats, bool), FetchError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let headers = rdr
        .byte_headers()
        .map_err(|e| FetchError::Parse(e.to_string()))?
        .clone();
    let cols = CompoundColumns::detect(&headers);

    let initial_cap = max_rows.min(2048);
    let mut entries: Vec<CompoundEntry> = Vec::with_capacity(initial_cap);
    let mut seen: HashSet<u64> = HashSet::with_capacity(initial_cap.saturating_mul(2));
    let mut compound_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut taxon_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut ref_fps: HashSet<u64> = HashSet::with_capacity(initial_cap);
    let mut total_raw = 0usize;
    let mut total_distinct = 0usize;
    let mut interners = CompoundInterners::new(initial_cap);

    let mut compound_qid = String::new();
    let mut taxon_qid = String::new();
    let mut reference_qid = String::new();

    let mut rec = csv::ByteRecord::new();
    while rdr
        .read_byte_record(&mut rec)
        .map_err(|e| FetchError::Parse(e.to_string()))?
    {
        compound_qid.clear();
        if let Some(i) = cols.compound
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut compound_qid, b);
        }
        if compound_qid.is_empty() {
            continue;
        }
        total_raw += 1;
        taxon_qid.clear();
        if let Some(i) = cols.taxon
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut taxon_qid, b);
        }
        reference_qid.clear();
        if let Some(i) = cols.ref_qid
            && let Some(b) = rec.get(i)
        {
            fill_qid(&mut reference_qid, b);
        }

        let key = entry_key_fingerprint(
            compound_qid.as_bytes(),
            taxon_qid.as_bytes(),
            reference_qid.as_bytes(),
        );
        if !seen.insert(key) {
            continue;
        }

        total_distinct += 1;
        compound_fps.insert(fnv1a_one(compound_qid.as_bytes()));
        if !taxon_qid.is_empty() {
            taxon_fps.insert(fnv1a_one(taxon_qid.as_bytes()));
        }
        if !reference_qid.is_empty() {
            ref_fps.insert(fnv1a_one(reference_qid.as_bytes()));
        }

        if entries.len() < max_rows {
            entries.push(interners.build_entry(
                &cols,
                &rec,
                &compound_qid,
                &taxon_qid,
                &reference_qid,
            ));
        }
    }

    let stats = DatasetStats {
        n_compounds: compound_fps.len(),
        n_taxa: taxon_fps.len(),
        n_references: ref_fps.len(),
        n_entries: total_raw,
        n_entries_unique: total_distinct,
    };
    let was_capped = total_distinct > entries.len();
    Ok((entries, stats, was_capped))
}

#[inline]
fn byte_field_str(rec: &csv::ByteRecord, idx: Option<usize>) -> &str {
    idx.and_then(|i| rec.get(i))
        .map_or("", |bytes| std::str::from_utf8(bytes).unwrap_or("").trim())
}

fn fill_qid(out: &mut String, bytes: &[u8]) {
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };
    if s.is_empty() {
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_taxon_csv_handles_uri_and_numeric_qids() {
        let csv = b"taxon,taxon_name\nhttp://www.wikidata.org/entity/Q123,Alpha\n\"456\"^^<http://www.w3.org/2001/XMLSchema#integer>,Beta\n";
        let parsed = parse_taxon_csv_bytes(csv).expect("taxon parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].qid, "Q123");
        assert_eq!(parsed[0].name, "Alpha");
        assert_eq!(parsed[1].qid, "Q456");
        assert_eq!(parsed[1].name, "Beta");
    }

    #[test]
    fn parse_counts_csv_prefers_unique_when_available() {
        let csv = b"n_entries,n_entries_unique,n_compounds,n_taxa,n_references\n10,7,3,2,4\n";
        let stats = parse_counts_csv_bytes(csv).expect("count parse");
        assert_eq!(stats.n_entries, 10);
        assert_eq!(stats.n_entries_unique, 7);
        assert_eq!(stats.n_compounds, 3);
        assert_eq!(stats.n_taxa, 2);
        assert_eq!(stats.n_references, 4);
    }

    #[test]
    fn parse_compounds_display_dedups_by_entry_triple() {
        let csv = b"compound,compoundLabel,compound_inchikey,compound_smiles_conn,compound_mass,compound_formula,taxon,taxon_name,ref_qid,ref_title,ref_doi,ref_date,statement\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ2,cmpd2,IK2,CC,111.1,C2H4,Q11,TaxonB,Q101,TitleB,10.1/b,2021-01-01,http://www.wikidata.org/entity/statement/S2\n";
        let rows = parse_compounds_csv_display_bytes(csv, 50).expect("display parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].compound_qid.as_ref(), "Q1");
        assert_eq!(rows[0].taxon_qid.as_ref(), "Q10");
        assert_eq!(rows[0].statement.as_deref(), Some("S1"));
        assert_eq!(rows[1].compound_qid.as_ref(), "Q2");
        assert_eq!(rows[1].statement.as_deref(), Some("S2"));
    }

    #[test]
    fn parse_compounds_capped_reports_cap_and_stats() {
        let csv = b"compound,compoundLabel,compound_inchikey,compound_smiles_conn,compound_mass,compound_formula,taxon,taxon_name,ref_qid,ref_title,ref_doi,ref_date,statement\nQ1,cmpd,IK1,C,123.4,C1H2,Q10,TaxonA,Q100,TitleA,10.1/a,2022-01-01,http://www.wikidata.org/entity/statement/S1\nQ2,cmpd2,IK2,CC,111.1,C2H4,Q11,TaxonB,Q101,TitleB,10.1/b,2021-01-01,http://www.wikidata.org/entity/statement/S2\nQ3,cmpd3,IK3,CCC,99.1,C3H6,Q12,TaxonC,Q102,TitleC,10.1/c,2020-01-01,http://www.wikidata.org/entity/statement/S3\n";
        let (rows, stats, capped) = parse_compounds_csv_capped_bytes(csv, 2).expect("capped parse");
        assert_eq!(rows.len(), 2);
        assert!(capped);
        assert_eq!(stats.n_entries, 3);
        assert_eq!(stats.n_entries_unique, 3);
        assert_eq!(stats.n_compounds, 3);
    }

    #[test]
    fn parse_compounds_capped_reader_matches_bytes_path() {
        let csv = b"compound,compoundLabel,taxon,ref_qid\nQ1,cmpd,Q10,Q100\nQ2,cmpd2,Q11,Q101\nQ3,cmpd3,Q12,Q102\n";
        let (rows_bytes, stats_bytes, capped_bytes) =
            parse_compounds_csv_capped_bytes(csv, 2).expect("bytes parse");
        let (rows_reader, stats_reader, capped_reader) =
            parse_compounds_csv_capped_reader(std::io::Cursor::new(csv), 2).expect("reader parse");

        assert_eq!(rows_reader.len(), rows_bytes.len());
        assert_eq!(stats_reader, stats_bytes);
        assert_eq!(capped_reader, capped_bytes);
    }
}
