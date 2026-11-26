// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

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

fn find_record_end(text: &str) -> Option<usize> {
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        if line.trim() == "$$$$" {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

impl Iterator for Parser<'_> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.text.len() {
                return None;
            }

            // Find the next ">" (property block opener).
            let gap = self.text[self.pos..].find('>')?;
            let abs = self.pos + gap;

            // If "$$$$" (record separator line) appears before the next ">",
            // skip past the separator.
            let before_gt = &self.text[self.pos..abs];
            if before_gt.lines().any(|l| l.trim() == "$$$$") {
                self.pos = abs;
                continue;
            }

            if let Some(record) = parse_one_record(&self.text[abs..]) {
                self.pos = abs + record.consumed;
                return Some(record.record);
            }

            // No property block found here — advance past this '>'.
            self.pos = abs + 1;
        }
    }
}

struct ParsedRecord {
    record: Record,
    consumed: usize,
}

fn parse_property_header(text: &str, abs_gt: usize) -> Option<(String, usize)> {
    let line_end = text[abs_gt..].find('\n').map_or(text.len(), |p| abs_gt + p);
    let header_line = text[abs_gt..line_end].trim_end_matches('\r');
    let after_gt = header_line[1..].trim_start_matches([' ', '\t']);
    let tag_tail = after_gt.strip_prefix('<')?;
    let name_end = tag_tail.find('>')?;
    Some((tag_tail[..name_end].to_string(), line_end))
}

fn read_property_value(text: &str, value_start: usize) -> (String, usize) {
    let mut value_lines: Vec<&str> = Vec::new();
    let mut line_start = value_start;
    while line_start < text.len() {
        let line_end = text[line_start..]
            .find('\n')
            .map_or(text.len(), |p| line_start + p);
        let line = text[line_start..line_end].trim_end_matches('\r');

        if line.trim().is_empty() || line.starts_with('>') || line.trim() == "$$$$" {
            break;
        }
        value_lines.push(line.trim_end());
        line_start = (line_end + 1).min(text.len());
    }
    (value_lines.join("\n"), line_start)
}

fn parse_one_record(text: &str) -> Option<ParsedRecord> {
    let mut pos = 0;
    let mut properties: HashMap<String, String> = HashMap::new();

    while pos < text.len() {
        let rest = &text[pos..];
        let dollar_pos = find_record_end(rest);
        let Some(gt_idx) = (if rest.starts_with('>') {
            Some(0)
        } else {
            rest.find("\n>").map(|idx| idx + 1)
        }) else {
            break;
        };

        if dollar_pos.is_some_and(|d| d < gt_idx) {
            break;
        }

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

    (!properties.is_empty()).then_some(ParsedRecord {
        record: Record { properties },
        consumed: pos,
    })
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
    let mut out = String::new();
    out.push_str(&COLUMNS.join("\t"));
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
            // Replace line breaks with "; " and tabs with spaces so
            // multi-line or tabbed SDF values don't break TSV boundaries.
            let val = record.get(col).unwrap_or("");
            let sanitized = val
                .replace("\r\n", "\n")
                .replace(['\n', '\r'], "; ")
                .replace('\t', " ");
            out.push_str(&sanitized);
        }
        out.push('\n');
        count += 1;
    }

    (out, count)
}

#[cfg(test)]
mod tests;
