// The `tests` tests, extracted from `analysis.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `analysis` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::super::parsing::extract_blocks;
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
