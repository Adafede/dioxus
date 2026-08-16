//! Aggregate analysis over parsed `SpectrumBlock`s: counts/summary, gallery
// construction, chemical-class matching across the collection, and
// filtered-MGF generation. Already-parsed blocks are pulled in from
// `super::parsing`.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

use super::parsing::{SpectrumBlock, extract_blocks};
use crate::chemical_class::ChemicalClass;
use chematic::{chem, smiles};
use std::collections::HashMap;

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
        .unwrap_or_default();

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
