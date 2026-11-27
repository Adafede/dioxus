// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Streaming SDF (Structure-Data File) parser.
//!
//! An SDF file is a sequence of records, each containing a molecular structure
//! (connection table) followed by zero or more property blocks in the form:
//!
//! ```text
//! > <PROPERTY_NAME>
//! property value can span multiple lines
//!
//! > <ANOTHER_PROPERTY>
//! another value
//!
//! $$$$   (record separator)
//! ```
//!
//! This parser focuses on extracting the property blocks, since the connection
//! tables are not needed for classification by chemical class.

use std::collections::HashMap;

/// One record extracted from an SDF file — a map of property name → value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Record {
    properties: HashMap<String, String>,
}

impl Record {
    #[must_use]
    fn get(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }
}

/// Iterator that yields [`Record`]s from an SDF text blob.
///
/// The iterator is lazy — it scans the text on each `next()` call without
/// pre-allocating a `Vec` of all records. This keeps memory bounded for
/// large files (the LMSD SDF is ~46 MB uncompressed).
#[derive(Debug)]
struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    const fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
}

/// Find the byte bounds of the next `$$$$` separator line in `text`.
///
/// Returns `(record_len, next_record_offset)` where `record_len` ends at the
/// start of `$$$$` and `next_record_offset` advances past the separator line
/// itself. When no `$$$$` separator appears, spans to `text.len()`.
fn find_record_span(text: &str) -> (usize, usize) {
    let mut search_from = 0usize;
    while let Some(rel) = text.get(search_from..).and_then(|s| s.find("$$$$")) {
        let match_start = search_from + rel;
        let line_start = text
            .get(..match_start)
            .and_then(|s| s.rfind('\n'))
            .map_or(0, |nl| nl + 1);
        let line_end = text
            .get(match_start..)
            .and_then(|s| s.find('\n'))
            .map_or(text.len(), |nl| match_start + nl + 1);
        if text
            .get(line_start..line_end)
            .is_some_and(|line| line.trim() == "$$$$")
        {
            return (line_start, line_end);
        }
        search_from = match_start + 4;
    }
    (text.len(), text.len())
}

impl Iterator for Parser<'_> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(rest) = self.text.get(self.pos..) {
            if rest.is_empty() {
                return None;
            }
            let (record_len, next_offset) = find_record_span(rest);
            let record_slice = rest.get(..record_len).unwrap_or(rest);
            self.pos += next_offset;

            if let Some(gt_offset) = record_slice.find('>')
                && let Some(prop_slice) = record_slice.get(gt_offset..)
                && let Some(record) = parse_record_properties(prop_slice)
            {
                return Some(record);
            }
        }
        None
    }
}

fn parse_property_header(text: &str, abs_gt: usize) -> Option<(String, usize)> {
    let after = text.get(abs_gt..)?;
    let line_end = after.find('\n').map_or(text.len(), |p| abs_gt + p);
    let header_line = text.get(abs_gt..line_end)?.trim_end_matches('\r');
    let after_gt = header_line.get(1..)?.trim_start_matches([' ', '\t']);
    let tag_tail = after_gt.strip_prefix('<')?;
    let name_end = tag_tail.find('>')?;
    Some((tag_tail.get(..name_end)?.to_string(), line_end))
}

fn read_property_value(text: &str, value_start: usize) -> (String, usize) {
    let mut value_lines: Vec<&str> = Vec::new();
    let mut line_start = value_start;
    while let Some(after) = text.get(line_start..) {
        if after.is_empty() {
            break;
        }
        let line_end = after.find('\n').map_or(text.len(), |p| line_start + p);
        let line = text
            .get(line_start..line_end)
            .unwrap_or("")
            .trim_end_matches('\r');

        if line.trim().is_empty() || line.starts_with('>') {
            break;
        }
        value_lines.push(line.trim_end());
        line_start = (line_end + 1).min(text.len());
    }
    (value_lines.join("\n"), line_start)
}

fn parse_record_properties(text: &str) -> Option<Record> {
    let mut pos = 0;
    let mut properties: HashMap<String, String> = HashMap::new();

    while let Some(rest) = text.get(pos..) {
        if rest.is_empty() {
            break;
        }
        let Some(gt_idx) = (if rest.starts_with('>') {
            Some(0)
        } else {
            rest.find("\n>").map(|idx| idx + 1)
        }) else {
            break;
        };

        let abs_gt = pos + gt_idx;
        let Some((name, line_end)) = parse_property_header(text, abs_gt) else {
            pos = abs_gt + 1;
            continue;
        };

        if line_end >= text.len() {
            break;
        }
        let (value, next_pos) = read_property_value(text, line_end + 1);
        properties.insert(name, value);
        pos = next_pos;
    }

    (!properties.is_empty()).then_some(Record { properties })
}

/// `LipidMaps` LMSD-specific column definitions and TSV conversion.
const COLUMNS: &[&str] = &[
    "LM_ID",
    "NAME",
    "SYSTEMATIC_NAME",
    "SYNONYMS",
    "CATEGORY",
    "MAIN_CLASS",
    "SUB_CLASS",
    "EXACT_MASS",
    "FORMULA",
    "INCHI_KEY",
    "INCHI",
    "SMILES",
    "PUBCHEM_CID",
    "CHEBI_ID",
    "KEGG_ID",
    "HMDB_ID",
    "SWISSLIPIDS_ID",
    "LIPIDBANK_ID",
    "PLANTFA_ID",
];

fn push_sanitized_tsv_field(out: &mut String, val: &str) {
    if !val.bytes().any(|b| matches!(b, b'\r' | b'\n' | b'\t')) {
        out.push_str(val);
        return;
    }
    let mut rest = val;
    while let Some(pos) = rest.find(['\r', '\n', '\t']) {
        let (prefix, tail) = rest.split_at(pos);
        out.push_str(prefix);
        if let Some(after_crlf) = tail.strip_prefix("\r\n") {
            out.push_str("; ");
            rest = after_crlf;
        } else if let Some(after_line) = tail.strip_prefix(['\r', '\n']) {
            out.push_str("; ");
            rest = after_line;
        } else if let Some(after_tab) = tail.strip_prefix('\t') {
            out.push(' ');
            rest = after_tab;
        } else {
            break;
        }
    }
    out.push_str(rest);
}

/// Read an SDF text blob and emit a TSV string with the LMSD columns.
///
/// Only records that have at least a non-empty `SMILES` and `LM_ID` are
/// included. Empty property values become empty TSV fields. Missing
/// properties also become empty fields (the column still appears).
/// Multi-line property values have newlines replaced with `"; "` to
/// preserve TSV row boundaries.
///
/// Returns `(tsv_text, record_count)`.
#[must_use]
pub(super) fn to_lmsd_tsv(sdf_text: &str) -> (String, usize) {
    let mut out = String::with_capacity(sdf_text.len() / 4);
    for (i, col) in COLUMNS.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        out.push_str(col);
    }
    out.push('\n');

    let mut count = 0;
    for record in Parser::new(sdf_text) {
        let smiles = record.get("SMILES").unwrap_or("");
        let lm_id = record.get("LM_ID").unwrap_or("");
        if smiles.is_empty() || lm_id.is_empty() {
            continue;
        }
        for (i, col) in COLUMNS.iter().enumerate() {
            if i > 0 {
                out.push('\t');
            }
            push_sanitized_tsv_field(&mut out, record.get(col).unwrap_or(""));
        }
        out.push('\n');
        count += 1;
    }

    (out, count)
}

#[cfg(test)]
mod tests;
