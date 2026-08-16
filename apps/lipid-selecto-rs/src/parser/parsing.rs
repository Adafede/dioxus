//! MGF `BEGIN IONS ... END IONS` block parsing and the `SpectrumBlock` domain
//! type.
//!
// A `SpectrumBlock` owns both its parsed metadata and the chemical-class
// classification derived from it, so `classify` / `compute_class_matches` live
// alongside the block structure (rather than in the aggregating `analysis`
// module). This keeps block-level parsing and per-block classification in one
// cohesive unit; cross-block aggregation lives in `super::analysis`.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use crate::chemical_class::ChemicalClass;
use crate::lipids::{LipidClassification, classify_spectrum, is_acyclic};
use chematic::smiles;
use std::collections::HashMap;

/// One `BEGIN IONS ... END IONS` record from the source MGF, together with the
/// metadata fields relevant to lipid selection.
#[derive(Clone, Debug)]
pub struct SpectrumBlock {
    /// 1-based position of the block within the original file.
    pub index: usize,
    /// Spectrum title (`TITLE=` / `NAME=`), if present.
    pub title: Option<String>,
    /// `SMILES=` value, if present.
    pub psm_smiles: Option<String>,
    /// `FORMULA=` value, if present.
    pub formula: Option<String>,
    /// `CHARGE=` value, if present.
    pub charge: Option<String>,
    /// `IONMODE=` value (normalized to lower case), if present.
    pub ion_mode: Option<String>,
    /// Observed precursor m/z from `PEPMASS=` / `PRECURSOR_MZ=`.
    pub precursor_mz: Option<f64>,
    /// Result of lipid classification (populated by [`SpectrumBlock::classify`]).
    pub classification: Option<LipidClassification>,
    /// Maps chemical class name -> bool (does this spectrum match?)
    pub gallery_item_matches: Option<HashMap<String, bool>>,
    /// Verbatim text of the block, from `BEGIN IONS` through `END IONS`.
    pub raw: String,
}

impl SpectrumBlock {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            psm_smiles: None,
            formula: None,
            charge: None,
            ion_mode: None,
            precursor_mz: None,
            classification: None,
            gallery_item_matches: None,
            raw: String::new(),
        }
    }

    /// Parse a single metadata line (`KEY=VALUE`) into the appropriate field.
    fn consume_metadata(&mut self, line: &str) {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("SMILES=") {
            self.psm_smiles = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("FORMULA=") {
            self.formula = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("CHARGE=") {
            self.charge = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("IONMODE=") {
            self.ion_mode = Some(value.trim().to_ascii_lowercase());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("PEPMASS=") {
            self.precursor_mz = parse_first_float(value);
            return;
        }
        if let Some(value) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            self.precursor_mz = value.trim().parse().ok();
            return;
        }
        if let Some(value) = trimmed.strip_prefix("TITLE=") {
            self.title = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("NAME=") {
            if self.title.is_none() {
                self.title = Some(value.trim().to_string());
            }
            return;
        }
        if let Some(value) = trimmed.strip_prefix("SCANS=")
            && self.title.is_none()
        {
            self.title = Some(value.trim().to_string());
        }
    }

    /// Parse the inline header tokens emitted on the `BEGIN IONS` line, e.g.
    /// `BEGIN IONS SMILES=... PEPMASS=100.0 CHARGE=1+`.
    fn consume_inline_header(&mut self, header: &str) {
        for token in header.split_whitespace() {
            if token.eq_ignore_ascii_case("BEGIN") || token.eq_ignore_ascii_case("IONS") {
                continue;
            }
            if token.contains('=') {
                self.consume_metadata(token);
            }
        }
    }

    /// Run the lipid classifier over this block's SMILES (with `FORMULA=`
    /// fallback) and store the result in [`SpectrumBlock::classification`].
    pub fn classify(&mut self) {
        self.classification =
            classify_spectrum(self.psm_smiles.as_deref(), self.formula.as_deref());
    }

    /// `true` when this spectrum has at least one matching chemical class.
    #[must_use]
    pub fn is_lipid(&self) -> bool {
        self.gallery_item_matches
            .as_ref()
            .is_some_and(|matches| matches.values().any(|&m| m))
    }

    /// Compute and store class matches for this block's SMILES.
    /// Only matches classes for acyclic molecules (true lipids).
    pub fn compute_class_matches(&mut self, classes: &[ChemicalClass]) {
        let mut matches = HashMap::new();
        if let Some(smiles_str) = &self.psm_smiles
            && let Ok(molecule) = smiles::parse(smiles_str.trim())
            && is_acyclic(&molecule)
        {
            for class in classes {
                matches.insert(class.name.clone(), class.matches(&molecule));
            }
        }
        self.gallery_item_matches = Some(matches);
    }
}

fn parse_first_float(text: &str) -> Option<f64> {
    text.split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
}

/// Detect the `BEGIN IONS` header (case-insensitive, tolerate inline tokens).
fn is_begin_ions(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.eq_ignore_ascii_case("BEGIN IONS")
        || (trimmed.starts_with("BEGIN IONS")
            && trimmed["BEGIN IONS".len()..].starts_with(char::is_whitespace))
}

/// Detect the `END IONS` terminus (case-insensitive).
fn is_end_ions(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.eq_ignore_ascii_case("END IONS")
        || (trimmed.starts_with("END IONS")
            && trimmed["END IONS".len()..].starts_with(char::is_whitespace))
}

/// Split raw MGF or SMILES text into [`SpectrumBlock`] records.
///
/// Accepts any iterator of line-like items so callers can stream from a
/// `upload::BlobLines` reader without buffering the entire file as a single
/// string.
///
/// For MGF: Each block's [`SpectrumBlock::raw`] preserves the original lines.
/// For SMILES: Each line becomes a `SpectrumBlock` with SMILES in the `psm_smiles` field.
///
/// # Panics
///
/// Panics if an `END IONS` marker appears without a matching open `BEGIN IONS`
/// block (the in-flight block is `None`); this only happens for malformed input.
#[must_use]
pub fn extract_blocks_from_lines<'a, I: Iterator<Item = impl AsRef<str> + 'a>>(
    lines: I,
) -> Vec<SpectrumBlock> {
    let mut blocks = Vec::new();
    let mut index = 0usize;
    let mut current: Option<SpectrumBlock> = None;

    for line in lines {
        let line = line.as_ref();

        if is_begin_ions(line) {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
            index += 1;
            let mut block = SpectrumBlock::new(index);
            block.consume_inline_header(line);
            block.raw.push_str(line);
            block.raw.push('\n');
            current = Some(block);
            continue;
        }

        if let Some(block) = current.as_mut() {
            if is_end_ions(line) {
                block.raw.push_str(line);
                block.raw.push('\n');
                blocks.push(current.take().expect("current block exists"));
                continue;
            }
            block.consume_metadata(line);
            block.raw.push_str(line);
            block.raw.push('\n');
        } else {
            // SMILES / flat format: each non-empty, non-comment line is a block.
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            index += 1;
            if let Some(block) = parse_smiles_line(index, trimmed) {
                blocks.push(block);
            }
        }
    }

    if let Some(block) = current {
        blocks.push(block);
    }

    blocks
}

/// Parse a single SMILES-format line into an optional [`SpectrumBlock`].
fn parse_smiles_line(index: usize, trimmed: &str) -> Option<SpectrumBlock> {
    let parts: Vec<&str> = trimmed.split('\t').collect();
    if parts.is_empty() {
        return None;
    }

    let looks_like_smiles = parts[0].contains(|c: char| {
        matches!(
            c,
            'C' | 'N' | 'O' | 'S' | 'P' | '=' | '#' | '[' | ']' | '(' | ')' | '@'
        )
    });
    let looks_like_id = parts.len() > 1
        && parts[1]
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-');

    let (smiles_str, id_str) = if looks_like_smiles && looks_like_id {
        (parts[0], parts[1])
    } else if parts.len() > 1 {
        (parts[1], parts[0])
    } else {
        (parts[0], "")
    };

    let mut block = SpectrumBlock::new(index);
    block.psm_smiles = Some(smiles_str.to_string());
    if !id_str.is_empty() {
        block.title = Some(id_str.to_string());
    }
    block.raw = trimmed.to_string();
    block.raw.push('\n');
    Some(block)
}

/// Split raw MGF or SMILES text into [`SpectrumBlock`] records.
///
/// Convenience wrapper around [`extract_blocks_from_lines`] for callers that
/// already have the full text in memory (tests, native builds).
#[must_use]
pub fn extract_blocks(content: &str) -> Vec<SpectrumBlock> {
    extract_blocks_from_lines(content.lines())
}
