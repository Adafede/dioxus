//! MGF parsing and lipid selection.
//!
//! The parser walks an MGF file block-by-block (`BEGIN IONS` ... `END IONS`),
//! preserving each block's verbatim text so that the filtered MGF is a faithful
//! subset of the input (no re-serialization drift). For every block it extracts
//! the per-spectrum metadata a lipid classifier needs — chiefly `SMILES=` and the
//! `FORMULA=` fallback — and tags the block with a [`crate::lipids::LipidClass`]
//! when it is recognized as a lipid.

// Spectrum indices and counts are small, non-negative values sourced from an
// in-memory parse; the `usize -> u32` casts here cannot truncate for realistic
// MGF inputs, so the pedantic cast lint is silenced.
#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use std::collections::HashMap;

use crate::lipids::{LipidClass, LipidClassification, classify_spectrum};

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
    /// Verbatim text of the block, from `BEGIN IONS` through `END IONS`.
    pub raw: String,
}

impl SpectrumBlock {
    pub(crate) const fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            psm_smiles: None,
            formula: None,
            charge: None,
            ion_mode: None,
            precursor_mz: None,
            classification: None,
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

    /// `true` when this block was recognized as a lipid.
    #[must_use]
    pub const fn is_lipid(&self) -> bool {
        self.classification.is_some()
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

/// Split raw MGF text into verbatim [`SpectrumBlock`] records.
///
/// Each block's [`SpectrumBlock::raw`] preserves the original lines (normalized
/// to `\n` line endings) so that a filtered MGF can be reconstructed byte for
/// byte.
///
/// # Panics
///
/// Panics if an `END IONS` marker appears without a matching open `BEGIN IONS`
/// block (the in-flight block is `None`); this only happens for malformed input.
#[must_use]
pub fn extract_blocks(content: &str) -> Vec<SpectrumBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<SpectrumBlock> = None;
    let mut index = 0usize;

    for line in content.lines() {
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
        }
    }

    if let Some(block) = current {
        blocks.push(block);
    }

    blocks
}

/// Aggregated counts produced by [`summarize`] / [`analyze`].
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub total_spectra: usize,
    pub lipid_spectra: usize,
    pub unclassified: usize,
    pub skipped: usize,
    pub class_counts: Vec<(LipidClass, usize)>,
}

impl Summary {
    /// Total spectra that had an annotation (SMILES or FORMULA).
    #[must_use]
    pub const fn annotated_total(&self) -> usize {
        self.total_spectra.saturating_sub(self.skipped)
    }

    /// Spectra that had an annotation but were not recognized as lipids.
    #[must_use]
    pub const fn non_lipid_annotated(&self) -> usize {
        self.annotated_total()
            .saturating_sub(self.lipid_spectra)
            .saturating_sub(self.unclassified)
    }
}

/// Tally lipids-vs-not from a parsed block collection.
#[must_use]
pub fn summarize(blocks: &[SpectrumBlock]) -> Summary {
    let mut summary = Summary::default();
    let mut class_counts: HashMap<LipidClass, usize> = HashMap::new();

    for block in blocks {
        let annotated = block
            .psm_smiles
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
            || block
                .formula
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty());
        if !annotated {
            summary.skipped += 1;
            continue;
        }

        if let Some(classification) = &block.classification {
            summary.lipid_spectra += 1;
            *class_counts.entry(classification.class).or_insert(0) += 1;
        } else {
            summary.unclassified += 1;
        }
    }

    summary.total_spectra = blocks.len();
    summary.class_counts = {
        let mut entries: Vec<(LipidClass, usize)> = class_counts.into_iter().collect();
        entries.sort_by_key(|(left_class, _)| *left_class);
        entries
    };
    summary
}

/// Project a single lipid-positive block into a gallery card, rendering its
/// 2D structure up-front.
#[must_use]
pub fn gallery_item(block: &SpectrumBlock) -> GalleryItem {
    let Some(classification) = &block.classification else {
        return GalleryItem {
            block_index: block.index,
            title: block.title.clone(),
            smiles: block.psm_smiles.clone(),
            formula: String::new(),
            class: LipidClass::Other,
            exact_mass: 0.0,
            precursor_mz: block.precursor_mz,
            charge: block.charge.clone(),
            svg: empty_svg(),
        };
    };
    let svg = block
        .psm_smiles
        .as_deref()
     .and_then(crate::depict_simple::render_svg)
        .unwrap_or_else(empty_svg);
    GalleryItem {
        block_index: block.index,
        title: block.title.clone(),
        smiles: block.psm_smiles.clone(),
        formula: classification.formula.clone(),
        class: classification.class,
        exact_mass: classification.exact_mass,
        precursor_mz: block.precursor_mz,
        charge: block.charge.clone(),
        svg,
    }
}

/// Build the gallery, rendering a 2D structure for each lipid block.
///
/// `limit` caps how many structures are generated (rendering is intentionally
/// done up-front so the gallery never re-renders diagrams on every frame).
#[must_use]
pub fn build_gallery(blocks: &[SpectrumBlock], limit: usize) -> Vec<GalleryItem> {
    let mut gallery = Vec::new();
    for block in blocks {
        if gallery.len() >= limit {
            break;
        }
        if !block.is_lipid() {
            continue;
        }
        gallery.push(gallery_item(block));
    }
    gallery
}

/// A lightweight, owned view of one selected lipid used to render the gallery.
#[derive(Clone, Debug)]
pub struct GalleryItem {
    pub block_index: usize,
    pub title: Option<String>,
    pub smiles: Option<String>,
    pub formula: String,
    pub class: LipidClass,
    pub exact_mass: f64,
    pub precursor_mz: Option<f64>,
    pub charge: Option<String>,
    pub svg: String,
}

/// Fallback SVG shown when a structure cannot be rendered.
fn empty_svg() -> String {
    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 40 20\"><text x=\"20\" y=\"12\" fill=\"#94a3b8\" text-anchor=\"middle\" font-family=\"system-ui\" font-size=\"10\">no structure</text></svg>".to_string()
}

/// Aggregate analysis result handed to the UI by the wasm worker / tests.
#[derive(Debug)]
pub struct Analysis {
    pub summary: Summary,
    pub gallery: Vec<GalleryItem>,
    pub filtered_mgf: String,
    pub blocks: Vec<SpectrumBlock>,
}

/// Full pipeline: extract, classify, summarize, build gallery + filtered MGF.
#[must_use]
pub fn build_analysis(blocks: &[SpectrumBlock], gallery_limit: usize) -> Analysis {
    let summary = summarize(blocks);
    let gallery = build_gallery(blocks, gallery_limit);
    let filtered_mgf = build_filtered_mgf(blocks);
    Analysis {
        summary,
        gallery,
        filtered_mgf,
        blocks: blocks.to_vec(),
    }
}

/// Concatenate the verbatim text of lipid-positive blocks into a filtered MGF.
#[must_use]
pub fn build_filtered_mgf(blocks: &[SpectrumBlock]) -> String {
    build_filtered_mgf_with_classes(blocks, &[
        LipidClass::FattyAcyl,
        LipidClass::Glycerolipid,
        LipidClass::Glycerophospholipid,
        LipidClass::Sphingolipid,
        LipidClass::Sterol,
        LipidClass::Other,
    ])
}

/// Concatenate the verbatim text of lipid-positive blocks into a filtered MGF,
/// including only blocks whose class is in the `selected_classes` list.
#[must_use]
pub fn build_filtered_mgf_with_classes(
    blocks: &[SpectrumBlock],
    selected_classes: &[LipidClass],
) -> String {
    let mut out = String::new();
    let mut first = true;
    for block in blocks {
        if !block.is_lipid() {
            continue;
        }
        let Some(classification) = &block.classification else {
            continue;
        };
        if !selected_classes.contains(&classification.class) {
            continue;
        }
        if !first {
            out.push('\n');
        }
        out.push_str(&block.raw);
        first = false;
    }
    out
}

/// Non-wasm convenience: fully parse, classify, summarize in one call.
///
/// The returned blocks keep their [`SpectrumBlock::classification`] populated so
/// callers can inspect or re-filter them; the [`Summary`] is also returned.
#[must_use]
pub fn analyze(content: &str) -> (Vec<SpectrumBlock>, Summary) {
    let mut blocks = extract_blocks(content);
    for block in &mut blocks {
        block.classify();
    }
    let summary = summarize(&blocks);
    (blocks, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_MGF: &str = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2
CHARGE=1-
TITLE=palmitic_acid
END IONS
BEGIN IONS SMILES=CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O
CHARGE=2-
FORMULA=C9H7O4S
TITLE=non_lipid_example
END IONS
BEGIN IONS
PEPMASS=500.0
TITLE=missing_annotation
END IONS
";

    #[test]
    fn extracts_and_preserves_block_text() {
        let blocks = extract_blocks(EXAMPLE_MGF);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].index, 1);
        assert_eq!(
            blocks[0].psm_smiles.as_deref(),
            Some("CCCCCCCCCCCCCCCC(=O)O")
        );
        assert_eq!(blocks[0].title.as_deref(), Some("palmitic_acid"));
        assert!(blocks[0].raw.starts_with("BEGIN IONS"));
        assert!(blocks[0].raw.contains("END IONS"));
    }

    #[test]
    fn detects_lipids_and_keeps_raw_subset() {
        let (blocks, _summary) = analyze(EXAMPLE_MGF);
        assert!(blocks[0].is_lipid());
        assert!(!blocks[1].is_lipid());
        assert!(!blocks[2].is_lipid());

        let filtered = build_filtered_mgf(&blocks);
        assert!(filtered.contains("BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O"));
        assert!(!filtered.contains("non_lipid_example"));
        assert!(!filtered.contains("missing_annotation"));
    }

    #[test]
    fn summary_counts_lipids() {
        let (blocks, _summary) = analyze(EXAMPLE_MGF);
        let summary = summarize(&blocks);
        assert_eq!(summary.total_spectra, 3);
        assert_eq!(summary.lipid_spectra, 1);
        assert_eq!(summary.skipped, 1);
    }

    #[test]
    fn analysis_builds_gallery_and_filtered_mgf() {
        let (blocks, _summary) = analyze(EXAMPLE_MGF);
        let analysis = build_analysis(&blocks, 16);
        assert_eq!(analysis.summary.lipid_spectra, 1);
        assert_eq!(analysis.gallery.len(), 1);
        assert!(analysis.filtered_mgf.contains("palmitic_acid"));
        assert!(!analysis.filtered_mgf.contains("non_lipid_example"));
    }
}
