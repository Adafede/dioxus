//! MGF parsing and chemical class matching.
//!
//! The parser walks an MGF file block-by-block (`BEGIN IONS` ... `END IONS`),
//! preserving each block's verbatim text so that the filtered MGF is a faithful
//! subset of the input (no re-serialization drift). For every block it extracts
//! SMILES and FORMULA metadata, then matches against user-defined chemical classes.

// Spectrum indices and counts are small, non-negative values sourced from an
// in-memory parse; the `usize -> u32` casts here cannot truncate for realistic
// MGF inputs, so the pedantic cast lint is silenced.
#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use std::collections::HashMap;

use crate::chemical_class::ChemicalClass;
use crate::lipids::{LipidClassification, classify_spectrum, is_acyclic};
use chematic::chem;
use chematic::smiles;

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
        if let Some(smiles_str) = &self.psm_smiles {
            if let Ok(molecule) = smiles::parse(smiles_str.trim()) {
                // Only compute matches for acyclic molecules - lipids must have no rings
                if is_acyclic(&molecule) {
                    for class in classes {
                        matches.insert(class.name.clone(), class.matches(&molecule));
                    }
                }
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
/// For MGF: Each block's [`SpectrumBlock::raw`] preserves the original lines.
/// For SMILES: Each line becomes a SpectrumBlock with SMILES in the psm_smiles field.
///
/// # Panics
///
/// Panics if an `END IONS` marker appears without a matching open `BEGIN IONS`
/// block (the in-flight block is `None`); this only happens for malformed input.
#[must_use]
pub fn extract_blocks(content: &str) -> Vec<SpectrumBlock> {
    // Detect if this is MGF format (look for BEGIN IONS markers)
    let is_mgf = content.lines().any(|line| is_begin_ions(line));

    if is_mgf {
        extract_blocks_mgf(content)
    } else {
        extract_blocks_smiles(content)
    }
}

/// Parse MGF format into spectrum blocks.
fn extract_blocks_mgf(content: &str) -> Vec<SpectrumBlock> {
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

/// Parse SMILES format (one SMILES per line) into spectrum blocks.
fn extract_blocks_smiles(content: &str) -> Vec<SpectrumBlock> {
    let mut blocks = Vec::new();
    let mut index = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        index += 1;
        let mut block = SpectrumBlock::new(index);

        // Parse tab-separated format: support both ID\tSMILES\tDESC or SMILES\tID
        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        // Detect format: if first field contains SMILES-like characters (C, N, O, =, [, ], etc)
        // and second field is alphanumeric/underscores, then it's SMILES\tID format.
        // Otherwise assume ID\tSMILES format.
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
            // SMILES\tID format
            (parts[0], parts[1])
        } else if parts.len() > 1 {
            // ID\tSMILES format (default)
            (parts[1], parts[0])
        } else {
            // Single field - assume it's SMILES
            (parts[0], "")
        };

        block.psm_smiles = Some(smiles_str.to_string());
        if !id_str.is_empty() {
            block.title = Some(id_str.to_string());
        }

        // Preserve the original line
        block.raw = line.to_string();
        block.raw.push('\n');

        blocks.push(block);
    }

    blocks
}

/// Aggregated counts produced by [`summarize`] / [`analyze`].
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub total_items: usize,
    pub lipid_items: usize,
    pub unclassified: usize,
    pub skipped: usize,
}

impl Summary {
    /// Total items that had an annotation (SMILES or FORMULA).
    #[must_use]
    pub const fn annotated_total(&self) -> usize {
        self.total_items.saturating_sub(self.skipped)
    }

    /// Items that had an annotation but were not recognized as lipids.
    #[must_use]
    pub const fn non_lipid_annotated(&self) -> usize {
        self.annotated_total()
            .saturating_sub(self.lipid_items)
            .saturating_sub(self.unclassified)
    }
}

/// Tally items from a parsed block collection.
#[must_use]
pub fn summarize(blocks: &[SpectrumBlock]) -> Summary {
    let mut summary = Summary::default();

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

        // Count based on whether any chemical class matches, not the old classification
        if block.is_lipid() {
            summary.lipid_items += 1;
        } else {
            summary.unclassified += 1;
        }
    }

    summary.total_items = blocks.len();
    summary
}

/// Project a single spectrum block into a gallery card, rendering its
/// 2D structure up-front.
#[must_use]
pub fn gallery_item(block: &SpectrumBlock, classes: &[ChemicalClass]) -> GalleryItem {
    // Compute exact mass from SMILES
    let exact_mass = block
        .psm_smiles
        .as_deref()
        .and_then(|smiles_str| smiles::parse(smiles_str.trim()).ok())
        .map(|mol| chem::exact_mass(&mol))
        .unwrap_or(0.0);

    // Use precomputed class matches from compute_class_matches
    let class_matches = block.gallery_item_matches.clone().unwrap_or_default();

    // Find first matching class color
    let primary_class_color = class_matches
        .iter()
        .find(|(_, matches)| **matches)
        .and_then(|(class_name, _)| {
            classes
                .iter()
                .find(|c| &c.name == class_name)
                .map(|c| c.color.clone())
        })
        .unwrap_or_else(|| "#f1f5f9".to_string());

    let svg = block
        .psm_smiles
        .as_deref()
        .and_then(crate::depict_simple::render_svg)
        .unwrap_or_else(empty_svg);

    GalleryItem {
        block_index: block.index,
        title: block.title.clone(),
        smiles: block.psm_smiles.clone(),
        formula: block.formula.clone().unwrap_or_default(),
        exact_mass,
        precursor_mz: block.precursor_mz,
        charge: block.charge.clone(),
        svg,
        class_matches,
        primary_class_color,
    }
}

/// Build the gallery, rendering a 2D structure for each lipid block.
///
/// `limit` caps how many structures are generated (rendering is intentionally
/// done up-front so the gallery never re-renders diagrams on every frame).
#[must_use]
pub fn build_gallery(
    blocks: &[SpectrumBlock],
    limit: usize,
    classes: &[ChemicalClass],
) -> Vec<GalleryItem> {
    let mut gallery = Vec::new();
    for block in blocks {
        if gallery.len() >= limit {
            break;
        }
        if !block.is_lipid() {
            continue;
        }
        gallery.push(gallery_item(block, classes));
    }
    gallery
}

/// A lightweight, owned view of one spectrum used to render the gallery.
#[derive(Clone, Debug)]
pub struct GalleryItem {
    pub block_index: usize,
    pub title: Option<String>,
    pub smiles: Option<String>,
    pub formula: String,
    pub exact_mass: f64,
    pub precursor_mz: Option<f64>,
    pub charge: Option<String>,
    pub svg: String,
    /// Maps chemical class name -> bool (does this molecule match?)
    pub class_matches: HashMap<String, bool>,
    /// Primary class color (from first matching class)
    pub primary_class_color: String,
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
    /// All available chemical classes (for UI selection)
    pub all_classes: Vec<ChemicalClass>,
}

/// Full pipeline: extract, classify, summarize, build gallery + filtered MGF.
#[must_use]
pub fn build_analysis(mut blocks: Vec<SpectrumBlock>, gallery_limit: usize) -> Analysis {
    let all_classes = ChemicalClass::defaults();

    // Compute class matches for all blocks
    for block in &mut blocks {
        block.compute_class_matches(&all_classes);
    }

    let summary = summarize(&blocks);
    let gallery = build_gallery(&blocks, gallery_limit, &all_classes);
    let filtered_mgf = build_filtered_mgf(&blocks);
    Analysis {
        summary,
        gallery,
        filtered_mgf,
        blocks,
        all_classes,
    }
}

/// Concatenate the verbatim text of all lipid-positive blocks into a filtered MGF.
/// This uses all default chemical classes.
#[must_use]
pub fn build_filtered_mgf(blocks: &[SpectrumBlock]) -> String {
    // Include all default class names
    let all_class_names: Vec<String> = ChemicalClass::defaults()
        .iter()
        .map(|c| c.name.clone())
        .collect();
    build_filtered_mgf_with_classes(blocks, &all_class_names)
}

/// Concatenate the verbatim text of blocks matching selected class names.
#[must_use]
pub fn build_filtered_mgf_with_classes(
    blocks: &[SpectrumBlock],
    selected_class_names: &[String],
) -> String {
    if selected_class_names.is_empty() {
        // If no classes selected, include nothing
        return String::new();
    }

    let mut out = String::new();
    let mut first = true;
    for block in blocks {
        if !block.is_lipid() {
            continue;
        }
        let Some(gallery_item) = block.gallery_item_matches.as_ref() else {
            continue;
        };
        // Include block if it matches at least one selected class
        let matches_any = selected_class_names
            .iter()
            .any(|class_name| gallery_item.get(class_name).copied().unwrap_or(false));
        if !matches_any {
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
    // Also compute class matches for is_lipid() to work correctly
    let all_classes = ChemicalClass::defaults();
    for block in &mut blocks {
        block.compute_class_matches(&all_classes);
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

        let analysis = build_analysis(blocks, 16);
        assert!(
            analysis
                .filtered_mgf
                .contains("BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O")
        );
        assert!(!analysis.filtered_mgf.contains("non_lipid_example"));
        assert!(!analysis.filtered_mgf.contains("missing_annotation"));
    }

    #[test]
    fn summary_counts_lipids() {
        let (blocks, _summary) = analyze(EXAMPLE_MGF);
        let summary = summarize(&blocks);
        assert_eq!(summary.total_items, 3);
        assert_eq!(summary.lipid_items, 1);
        assert_eq!(summary.skipped, 1);
    }

    #[test]
    fn analysis_builds_gallery_and_filtered_mgf() {
        let (blocks, _summary) = analyze(EXAMPLE_MGF);
        let analysis = build_analysis(blocks, 16);
        assert_eq!(analysis.summary.lipid_items, 1);
        assert_eq!(analysis.gallery.len(), 1);
        assert!(!analysis.all_classes.is_empty());
        assert!(analysis.filtered_mgf.contains("palmitic_acid"));
        assert!(!analysis.filtered_mgf.contains("non_lipid_example"));
    }

    #[test]
    fn gallery_items_have_class_matches() {
        let (blocks, _) = analyze(EXAMPLE_MGF);
        let analysis = build_analysis(blocks, 16);

        assert_eq!(analysis.gallery.len(), 1);
        let item = &analysis.gallery[0];

        // All gallery items should have class_matches computed
        assert!(!item.class_matches.is_empty());

        // At least one class should match (since it's a lipid)
        let has_match = item.class_matches.values().any(|&m| m);
        assert!(has_match);
    }

    #[test]
    fn chemical_classes_have_all_required_fields() {
        let classes = ChemicalClass::defaults();

        for class in classes {
            assert!(!class.name.is_empty(), "Class name should not be empty");
            assert!(!class.smarts.is_empty(), "Class SMARTS should not be empty");
            assert!(!class.color.is_empty(), "Class color should not be empty");
            // Colors should start with # or be valid CSS
            assert!(class.color.starts_with('#'), "Color should be hex code");
        }
    }
}
