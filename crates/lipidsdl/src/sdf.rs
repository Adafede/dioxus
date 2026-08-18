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

#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

/// One record extracted from an SDF file — a map of property name → value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    pub properties: HashMap<String, String>,
}

impl Record {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }
}

/// Iterator that yields [`Record`]s from an SDF text blob.
///
/// The iterator is lazy — it scans the text on each `next()` call without
/// pre-allocating a `Vec` of all records. This keeps memory bounded for
/// large files (the LMSD SDF is ~46 MB uncompressed).
#[derive(Debug)]
pub struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
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

            // If "$$$$" (record separator) appears before the next ">",
            // skip past the separator.
            let before_gt = &self.text[self.pos..abs];
            if before_gt.contains("$$$$") {
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

fn parse_one_record(text: &str) -> Option<ParsedRecord> {
    let mut pos = 0;
    let mut properties: HashMap<String, String> = HashMap::new();

    loop {
        let rest = &text[pos..];

        // If "$$$$" (record separator) comes before the next "> ",
        // end of record.
        let dollar_pos = rest.find("$$$$");
        let Some(gt_idx) = rest.find("> ") else {
            break;
        };

        if let Some(d) = dollar_pos
            && d < gt_idx
        {
            break;
        }

        let abs_gt = pos + gt_idx;

        // Check that this is a property block: "> <NAME>" pattern.
        let after_gt = &text[abs_gt..];
        if !after_gt.starts_with("> <") {
            pos = abs_gt + 1;
            continue;
        }

        // Extract property name between < and >.
        let name_start = abs_gt + 3; // skip "> <"
        let name_end = text[name_start..].find('>')?;
        let name = text[name_start..name_start + name_end].to_string();

        // Value starts on the next line.
        let after_name = name_start + name_end + 1; // skip closing '>'
        let newline_pos = text[after_name..].find('\n')? + after_name;
        let value_start = newline_pos + 1; // skip the newline

        // Collect value lines until a blank line, next ">", or "$$$$".
        let mut value_lines: Vec<&str> = Vec::new();
        let mut line_start = value_start;
        loop {
            let line_end = text[line_start..]
                .find('\n')
                .map_or(text.len(), |p| line_start + p);
            let line = &text[line_start..line_end];

            if line.trim().is_empty() || line.starts_with('>') || line.starts_with("$$$$") {
                break;
            }
            value_lines.push(line.trim_end());
            line_start = line_end + 1;
            if line_start >= text.len() {
                break;
            }
        }

        let value = value_lines.join("\n");
        properties.insert(name, value);
        pos = line_start;
    }

    if properties.is_empty() {
        None
    } else {
        Some(ParsedRecord {
            record: Record { properties },
            consumed: pos,
        })
    }
}

/// `LipidMaps` LMSD-specific column definitions and TSV conversion.
pub mod lipidmaps {
    use super::Parser;

    /// The 19 columns the user requested, in order, matching the LMSD SDF
    /// property names exactly.
    pub const COLUMNS: &[&str] = &[
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
    pub fn to_lmsd_tsv(sdf_text: &str) -> (String, usize) {
        let mut out = String::new();
        let header = COLUMNS.join("\t");
        out.push_str(&header);
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
                // Replace newlines with "; " so multi-line SDF values
                // (e.g. SYNONYMS, INCHI) don't break TSV row boundaries.
                let val = record.get(col).unwrap_or("");
                out.push_str(&val.replace('\n', "; "));
            }
            out.push('\n');
            count += 1;
        }

        (out, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SDF: &str = r"  CDK     2DS

  20 21  0  0  0  0  0  0  0  0999 V2000
    2.3120    0.1280    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
M  END
> <LM_ID>
LMFA00000001

> <NAME>
Test fatty acid

> <SMILES>
C(C(C(C(=O)O)O)O)O

$$$$

  CDK     2DS

  15 16  0  0  0  0  0  0  0  0999 V2000
    1.2340    0.5670    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
M  END
> <LM_ID>
LMFA00000002

> <NAME>
Missing SMILES lipid

> <SMILES>

$$$$
";

    #[test]
    fn parse_sdf_extracts_properties() {
        let parser = Parser::new(SAMPLE_SDF);
        let records: Vec<_> = parser.collect();
        assert_eq!(records.len(), 2);

        let rec1 = &records[0];
        assert_eq!(rec1.get("LM_ID"), Some("LMFA00000001"));
        assert_eq!(rec1.get("NAME"), Some("Test fatty acid"));
        assert_eq!(rec1.get("SMILES"), Some("C(C(C(C(=O)O)O)O)O"));
    }

    #[test]
    fn to_lmsd_tsv_filters_empty_smiles() {
        let (tsv, count) = lipidmaps::to_lmsd_tsv(SAMPLE_SDF);
        assert_eq!(count, 1);
        assert!(tsv.contains("LMFA00000001\tTest fatty acid"));
        assert!(!tsv.contains("LMFA00000002"));
    }

    #[test]
    fn to_lmsd_tsv_has_all_columns() {
        let (tsv, _) = lipidmaps::to_lmsd_tsv(SAMPLE_SDF);
        let header_line = tsv.lines().next().unwrap();
        for col in lipidmaps::COLUMNS {
            assert!(
                header_line.contains(col),
                "column {col} missing from header"
            );
        }
        assert_eq!(lipidmaps::COLUMNS.len(), 19);
    }

    #[test]
    fn empty_sdf_yields_empty_tsv() {
        let (tsv, count) = lipidmaps::to_lmsd_tsv("");
        assert_eq!(count, 0);
        assert!(tsv.starts_with("LM_ID\tNAME"));
    }

    #[test]
    fn to_lmsd_tsv_handles_missing_properties() {
        let sdf = "> <LM_ID>\nLMFA00000003\n\n> <SMILES>\nC(=O)O\n\n$$$$\n";
        let (tsv, count) = lipidmaps::to_lmsd_tsv(sdf);
        assert_eq!(count, 1);
        let lines: Vec<&str> = tsv.lines().collect();
        assert_eq!(lines.len(), 2);
        let field_count = lines[1].split('\t').count();
        assert_eq!(field_count, lipidmaps::COLUMNS.len());
    }

    #[test]
    fn to_lmsd_tsv_multi_line_value() {
        let sdf = "> <LM_ID>\nLMFA00000004\n\n> <SYNONYMS>\nSyn1\nSyn2\n\n> <SMILES>\nCC\n\n$$$$\n";
        let (tsv, count) = lipidmaps::to_lmsd_tsv(sdf);
        assert_eq!(count, 1);
        let lines: Vec<&str> = tsv.lines().collect();
        let cols: Vec<&str> = lines[1].split('\t').collect();
        let syn_idx = lipidmaps::COLUMNS
            .iter()
            .position(|&c| c == "SYNONYMS")
            .unwrap();
        assert_eq!(cols[syn_idx], "Syn1; Syn2");
    }
}
